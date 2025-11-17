//! Example of multi-source lineage merging
//!
//! This demonstrates Lightweaver's killer feature: automatic cross-tool
//! lineage stitching based on dataset URNs.
//!
//! Run with: cargo run --example multi_source

use lightweaver::PipelineVisualizer;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Multi-Source Lineage Merging Example\n");
    println!("=====================================\n");

    // Create a visualizer with dark theme
    let mut viz = PipelineVisualizer::new()
        .with_theme("dark");

    // Add multiple sources
    println!("Adding data sources:");

    let dbt_events = Path::new("test_data/dbt_events.json");
    if dbt_events.exists() {
        println!("  ✓ dbt events");
        viz.add_openlineage_events(dbt_events)?;
    }

    let airflow_events = Path::new("test_data/airflow_events.json");
    if airflow_events.exists() {
        println!("  ✓ Airflow events");
        viz.add_openlineage_events(airflow_events)?;
    }

    // Show what we're merging
    let stats = viz.stats()?;
    println!("\nMerging {} sources:", stats.source_count);
    println!("  Total nodes: {}", stats.total_nodes);
    println!("  Jobs: {}", stats.job_nodes);
    println!("  Datasets: {}", stats.dataset_nodes);
    println!("  Edges: {}", stats.total_edges);

    // The magic happens here: datasets with matching URNs are deduplicated
    // and cross-tool edges are automatically created
    println!("\n🔗 Cross-tool stitching happening automatically...");

    // Render
    println!("\nRendering visualization...");
    let svg = viz.render_svg()?;

    // Save
    let output = "multi_source_example.svg";
    std::fs::write(output, svg)?;

    println!("✨ Generated {}", output);
    println!("\nOpen it in your browser to see the cross-tool lineage!");

    Ok(())
}
