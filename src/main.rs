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

    // Create visualizer with theme
    let mut viz = PipelineVisualizer::new().with_theme(&args.theme);

    // Add dbt manifest if provided
    if let Some(dbt_path) = args.dbt_manifest {
        println!("📖 Reading dbt manifest: {}", dbt_path.display());
        viz.add_dbt_manifest(&dbt_path)?;
        let stats = viz.stats()?;
        println!("   Found {} nodes, {} edges from dbt",
            stats.total_nodes, stats.total_edges);
    }

    // Add OpenLineage events if provided (can have multiple files)
    for openlineage_path in &args.openlineage {
        println!("📖 Reading OpenLineage events: {}", openlineage_path.display());
        let before = viz.stats().ok().map(|s| (s.total_nodes, s.total_edges));
        viz.add_openlineage_events(openlineage_path)?;
        let after = viz.stats()?;

        if let Some((before_nodes, before_edges)) = before {
            let new_nodes = after.total_nodes - before_nodes;
            let new_edges = after.total_edges - before_edges;
            println!("   Found {} nodes, {} edges from OpenLineage", new_nodes, new_edges);
        } else {
            println!("   Found {} nodes, {} edges from OpenLineage",
                after.total_nodes, after.total_edges);
        }
    }

    // Get final stats
    let stats = viz.stats()?;

    // Show merge info if multiple sources
    if stats.source_count > 1 {
        println!("🔗 Merging {} sources...", stats.source_count);
        println!("   Merged graph: {} nodes, {} edges", stats.total_nodes, stats.total_edges);
    }

    // Render to SVG
    println!("🎨 Rendering SVG...");
    let svg = viz.render_svg()?;
    let svg_bytes = svg.as_bytes();

    let size_kb = svg_bytes.len() / 1024;
    println!("💾 Writing to: {} ({} KB)", args.output.display(), size_kb);
    fs::write(&args.output, svg_bytes)?;

    let elapsed = start_time.elapsed().as_secs_f64();
    println!("✨ Done! Generated {} in {:.2}s", args.output.display(), elapsed);

    // Output statistics summary
    println!("\n📊 Summary:");
    println!("   {}", stats);

    Ok(())
}
