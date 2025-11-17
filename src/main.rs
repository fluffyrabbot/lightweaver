//! Lightweaver CLI - Thin wrapper around the library API

mod cli;

use cli::{parse_args, print_help, Command};
use lightweaver::PipelineVisualizer;
use std::fs;
use std::process;
use std::time::Instant;

fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!();
            print_help();
            process::exit(1);
        }
    };

    match args.command {
        Command::Generate(gen_args) => {
            if let Err(e) = generate(gen_args) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Command::Help => {
            print_help();
        }
        Command::Version => {
            println!("lightweaver {}", env!("CARGO_PKG_VERSION"));
            println!("Universal Data Pipeline Visualizer");
        }
    }
}

fn generate(args: cli::GenerateArgs) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    let quiet = args.quiet;

    // Create visualizer with theme
    let mut viz = PipelineVisualizer::new().with_theme(&args.theme);

    // Build filter if any filter flags provided
    if !args.filter_tools.is_empty() || !args.filter_tags.is_empty() || !args.filter_namespaces.is_empty() {
        use lightweaver::{GraphFilter, ToolType};

        let mut filter = GraphFilter::new();

        // Parse tool types
        for tool_str in &args.filter_tools {
            let tool = match tool_str.to_lowercase().as_str() {
                "dbt" => ToolType::Dbt,
                "airflow" => ToolType::Airflow,
                "spark" => ToolType::Spark,
                "fivetran" => ToolType::Fivetran,
                "dagster" => ToolType::Dagster,
                "prefect" => ToolType::Prefect,
                "flink" => ToolType::Flink,
                "kafka" => ToolType::Kafka,
                "great_expectations" | "greatexpectations" => ToolType::GreatExpectations,
                "custom" => ToolType::Custom,
                _ => {
                    return Err(format!(
                        "Unknown tool type '{}'. Valid options: dbt, airflow, spark, fivetran, dagster, prefect, flink, kafka, great_expectations, custom",
                        tool_str
                    ).into());
                }
            };
            filter = filter.with_tool(tool);
        }

        // Add tag filters
        for tag in &args.filter_tags {
            filter = filter.with_tag(tag.clone());
        }

        // Add namespace filters
        for ns in &args.filter_namespaces {
            filter = filter.with_namespace(ns.clone());
        }

        viz.set_filter(filter);

        if !quiet {
            println!("🔍 Applying filters:");
            if !args.filter_tools.is_empty() {
                println!("   Tools: {}", args.filter_tools.join(", "));
            }
            if !args.filter_tags.is_empty() {
                println!("   Tags: {}", args.filter_tags.join(", "));
            }
            if !args.filter_namespaces.is_empty() {
                println!("   Namespaces: {}", args.filter_namespaces.join(", "));
            }
        }
    }

    // Add dbt manifest if provided
    if let Some(dbt_path) = args.dbt_manifest {
        if !quiet {
            println!("📖 Reading dbt manifest: {}", dbt_path.display());
        }
        viz.add_dbt_manifest(&dbt_path)?;
        let stats = viz.stats()?;
        if !quiet {
            println!("   Found {} nodes, {} edges from dbt",
                stats.total_nodes, stats.total_edges);
        }
    }

    // Add OpenLineage events if provided (can have multiple files)
    for openlineage_path in &args.openlineage {
        if !quiet {
            println!("📖 Reading OpenLineage events: {}", openlineage_path.display());
        }
        let before = viz.stats().ok().map(|s| (s.total_nodes, s.total_edges));
        viz.add_openlineage_events(openlineage_path)?;
        let after = viz.stats()?;

        if !quiet {
            if let Some((before_nodes, before_edges)) = before {
                let new_nodes = after.total_nodes - before_nodes;
                let new_edges = after.total_edges - before_edges;
                println!("   Found {} nodes, {} edges from OpenLineage", new_nodes, new_edges);
            } else {
                println!("   Found {} nodes, {} edges from OpenLineage",
                    after.total_nodes, after.total_edges);
            }
        }
    }

    // Get final stats (after filtering if filters were applied)
    let stats = viz.stats()?;

    // Show merge info if multiple sources
    if !quiet && stats.source_count > 1 {
        println!("🔗 Merging {} sources...", stats.source_count);
        println!("   Merged graph: {} nodes, {} edges", stats.total_nodes, stats.total_edges);
    }

    // Show filtering results if filters were applied
    if !quiet && (!args.filter_tools.is_empty() || !args.filter_tags.is_empty() || !args.filter_namespaces.is_empty()) {
        println!("✂️  After filtering: {} nodes, {} edges", stats.total_nodes, stats.total_edges);
    }

    // Warn about large graphs (always show, even in quiet mode)
    if stats.total_nodes > 500 {
        eprintln!("⚠️  Large graph warning: {} nodes detected", stats.total_nodes);
        eprintln!("   Rendering may take longer and produce a large SVG file.");
        eprintln!("   Consider filtering your input data or splitting into multiple visualizations.");
    }

    // Determine output format (explicit --format or auto-detect from extension)
    let format = args.format.as_deref().unwrap_or_else(|| {
        match args.output.extension().and_then(|s| s.to_str()) {
            Some("html") => "html",
            Some("png") => "png",
            Some("pdf") => "pdf",
            _ => "svg",
        }
    });

    // Render to appropriate format
    let output_bytes = match format {
        "html" => {
            if !quiet {
                println!("🎨 Rendering interactive HTML...");
            }
            viz.render_html()?.into_bytes()
        }
        #[cfg(feature = "png")]
        "png" => {
            if !quiet {
                println!("🎨 Rendering PNG image...");
            }
            viz.render_png()?
        }
        #[cfg(not(feature = "png"))]
        "png" => {
            return Err("PNG support not enabled. Rebuild with --features png".into());
        }
        #[cfg(feature = "pdf")]
        "pdf" => {
            if !quiet {
                println!("🎨 Rendering PDF document...");
            }
            viz.render_pdf()?
        }
        #[cfg(not(feature = "pdf"))]
        "pdf" => {
            return Err("PDF support not enabled. Rebuild with --features pdf".into());
        }
        _ => {
            if !quiet {
                println!("🎨 Rendering SVG...");
            }
            viz.render_svg()?.into_bytes()
        }
    };

    let size_kb = output_bytes.len() / 1024;
    if !quiet {
        println!("💾 Writing to: {} ({} KB)", args.output.display(), size_kb);
    }
    fs::write(&args.output, &output_bytes)?;

    // Warn about very large SVG files (always show)
    if format == "svg" && size_kb > 5000 {
        eprintln!("⚠️  Large SVG warning: {} KB generated", size_kb);
        eprintln!("   This may be slow to open in browsers or editors.");
        eprintln!("   Consider using a dedicated SVG viewer for best performance.");
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    if !quiet {
        println!("✨ Done! Generated {} in {:.2}s", args.output.display(), elapsed);
        // Output statistics summary
        println!("\n📊 Summary:");
        println!("   {}", stats);
    }

    Ok(())
}
