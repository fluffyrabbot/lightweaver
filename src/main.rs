// Lightweaver - Data Pipeline Visualizer

mod graph;
mod layout;
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
    }
}

fn generate(args: cli::GenerateArgs) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = args.dbt_manifest.ok_or("--dbt is required")?;

    println!("📖 Reading dbt manifest: {}", manifest_path.display());
    let graph = parsers::dbt::parse_manifest(&manifest_path)?;
    println!("   Found {} nodes, {} edges", graph.node_count(), graph.edge_count());

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

    println!("💾 Writing to: {}", args.output.display());
    fs::write(&args.output, svg)?;

    println!("✨ Done! Generated {}", args.output.display());

    Ok(())
}
