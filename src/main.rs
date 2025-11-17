// Lightweaver - Data Pipeline Visualizer

mod graph;
mod layout;
mod lineage;
mod openlineage;
mod parsers;
mod render;
mod cli;

use cli::{Command, parse_args, print_help};
use layout::{Layout, hierarchical::HierarchicalLayout};
use render::{SvgRenderer, Theme};
use std::fs;
use std::process;

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
    use std::time::Instant;
    let start_time = Instant::now();

    let mut graphs = Vec::new();

    // Parse dbt manifest if provided
    if let Some(dbt_path) = args.dbt_manifest {
        println!("📖 Reading dbt manifest: {}", dbt_path.display());
        let g = parsers::dbt::parse_manifest(&dbt_path)?;
        println!("   Found {} nodes, {} edges from dbt", g.node_count(), g.edge_count());
        graphs.push(g);
    }

    // Parse OpenLineage events if provided (can have multiple files)
    for openlineage_path in &args.openlineage {
        println!("📖 Reading OpenLineage events: {}", openlineage_path.display());
        let g = parsers::openlineage::parse_events(openlineage_path)?;
        println!("   Found {} nodes, {} edges from OpenLineage", g.node_count(), g.edge_count());
        graphs.push(g);
    }

    if graphs.is_empty() {
        return Err("At least one input source is required".into());
    }

    // Merge graphs if multiple sources
    let graph = if graphs.len() > 1 {
        println!("🔗 Merging {} sources...", graphs.len());
        let mut merged = lineage::merger::merge_graphs(graphs);
        lineage::merger::deduplicate_datasets(&mut merged);
        println!("   Merged graph: {} nodes, {} edges", merged.node_count(), merged.edge_count());
        merged
    } else {
        graphs.into_iter().next().unwrap()
    };

    println!("📐 Computing layout...");
    let layout_engine = HierarchicalLayout::default();
    let layout = layout_engine.compute(&graph)?;
    println!("   Layout complete: {}x{}", layout.width as u32, layout.height as u32);

    println!("🎨 Rendering SVG...");
    let theme = match args.theme.as_str() {
        "dark" => Theme::dark(),
        _ => Theme::default(),
    };

    let renderer = SvgRenderer {
        theme,
        ..Default::default()
    };

    let svg = renderer.render(&graph, &layout);
    let svg_bytes = svg.as_bytes();

    let size_kb = svg_bytes.len() / 1024;
    println!("💾 Writing to: {} ({} KB)", args.output.display(), size_kb);
    fs::write(&args.output, svg_bytes)?;

    let elapsed = start_time.elapsed().as_secs_f64();
    println!("✨ Done! Generated {} in {:.2}s", args.output.display(), elapsed);

    // Output statistics summary
    println!("\n📊 Summary:");
    println!("   {} nodes ({} jobs, {} datasets)",
        graph.node_count(), graph.job_count(), graph.dataset_count());
    println!("   {} edges", graph.edge_count());

    Ok(())
}
