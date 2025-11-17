//! Simple example of using Lightweaver as a library
//!
//! Run with: cargo run --example simple

use lightweaver::PipelineVisualizer;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating pipeline visualization...\n");

    // Create a visualizer
    let mut viz = PipelineVisualizer::new()
        .with_theme("light");

    // Add a dbt manifest
    let manifest_path = Path::new("test_data/jaffle_manifest.json");
    if manifest_path.exists() {
        println!("Adding dbt manifest...");
        viz.add_dbt_manifest(manifest_path)?;
    }

    // Add OpenLineage events
    let events_path = Path::new("test_data/cross_tool_pipeline.json");
    if events_path.exists() {
        println!("Adding OpenLineage events...");
        viz.add_openlineage_events(events_path)?;
    }

    // Get statistics
    let stats = viz.stats()?;
    println!("\nStatistics:");
    println!("  {}", stats);

    // Render to SVG
    println!("\nRendering SVG...");
    let svg = viz.render_svg()?;

    // Save to file
    let output_path = "example_output.svg";
    std::fs::write(output_path, svg)?;

    println!("✨ Generated {}", output_path);

    Ok(())
}
