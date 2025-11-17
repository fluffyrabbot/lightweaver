//! Example: Generating interactive HTML visualizations
//!
//! This example demonstrates creating interactive HTML output with:
//! - Pan and zoom controls
//! - Click nodes to view details
//! - Search functionality
//! - Dark mode toggle

use lightweaver::PipelineVisualizer;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating interactive HTML visualizations...\n");

    // Example 1: Basic HTML output
    println!("Example 1: Basic interactive HTML");
    let mut viz = PipelineVisualizer::new();
    viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json"))?;

    let html = viz.render_html()?;
    std::fs::write("basic_interactive.html", html)?;
    println!("  → Saved to basic_interactive.html");
    println!("  → Open in browser to interact!\n");

    // Example 2: Dark theme HTML
    println!("Example 2: Dark theme interactive HTML");
    let mut dark_viz = PipelineVisualizer::new()
        .with_theme("dark");
    dark_viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json"))?;

    let dark_html = dark_viz.render_html()?;
    std::fs::write("dark_interactive.html", dark_html)?;
    println!("  → Saved to dark_interactive.html\n");

    // Example 3: Multi-source with filters
    println!("Example 3: Multi-source pipeline (filtered)");
    use lightweaver::{GraphFilter, ToolType};

    let mut multi_viz = PipelineVisualizer::new();
    multi_viz.add_openlineage_events(Path::new("test_data/dbt_events.json"))?;
    multi_viz.add_openlineage_events(Path::new("test_data/airflow_events.json"))?;

    // Filter to show only dbt nodes
    let filter = GraphFilter::new().with_tool(ToolType::Dbt);
    multi_viz.set_filter(filter);

    let multi_html = multi_viz.render_html()?;
    std::fs::write("multi_source_interactive.html", multi_html)?;
    println!("  → Saved to multi_source_interactive.html\n");

    println!("✨ All examples generated!");
    println!("\nInteractive features:");
    println!("  • Mouse wheel or trackpad to zoom");
    println!("  • Click and drag to pan");
    println!("  • Click nodes to view metadata sidebar");
    println!("  • Search box to highlight matching nodes");
    println!("  • Press '/' to focus search");
    println!("  • Press Esc to close sidebar");
    println!("  • Toggle dark mode button in toolbar");

    Ok(())
}
