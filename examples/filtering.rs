//! Example: Filtering pipeline graphs
//!
//! Demonstrates how to filter graphs by tool type, tag, or namespace.

use lightweaver::{GraphFilter, PipelineVisualizer, ToolType};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a multi-tool pipeline
    let mut viz = PipelineVisualizer::new();
    viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json"))?;

    println!("Full graph statistics:");
    let full_stats = viz.stats()?;
    println!("  {}\n", full_stats);

    // Example 1: Filter by tool type
    println!("Example 1: Show only dbt nodes");
    let dbt_filter = GraphFilter::new().with_tool(ToolType::Dbt);
    viz.set_filter(dbt_filter);

    let dbt_stats = viz.stats()?;
    println!("  {}", dbt_stats);
    let svg = viz.render_svg()?;
    std::fs::write("filtered_dbt_only.svg", svg)?;
    println!("  → Saved to filtered_dbt_only.svg\n");

    // Example 2: Filter by multiple tools
    println!("Example 2: Show dbt and Airflow nodes");
    let multi_tool_filter = GraphFilter::new()
        .with_tool(ToolType::Dbt)
        .with_tool(ToolType::Airflow);
    viz.set_filter(multi_tool_filter);

    let multi_stats = viz.stats()?;
    println!("  {}", multi_stats);
    let svg = viz.render_svg()?;
    std::fs::write("filtered_dbt_airflow.svg", svg)?;
    println!("  → Saved to filtered_dbt_airflow.svg\n");

    // Example 3: Filter by tag
    println!("Example 3: Show only nodes with 'production' tag");
    let tag_filter = GraphFilter::new().with_tag("production");
    viz.set_filter(tag_filter);

    // May have no matching nodes if test data doesn't have tags
    if let Ok(tag_stats) = viz.stats() {
        println!("  {}", tag_stats);
        if tag_stats.total_nodes > 0 {
            let svg = viz.render_svg()?;
            std::fs::write("filtered_production.svg", svg)?;
            println!("  → Saved to filtered_production.svg\n");
        } else {
            println!("  (No nodes match this filter in test data)\n");
        }
    }

    // Example 4: Filter by namespace
    println!("Example 4: Show only dbt:// namespace");
    let ns_filter = GraphFilter::new().with_namespace("dbt://");
    viz.set_filter(ns_filter);

    let ns_stats = viz.stats()?;
    println!("  {}", ns_stats);
    let svg = viz.render_svg()?;
    std::fs::write("filtered_dbt_namespace.svg", svg)?;
    println!("  → Saved to filtered_dbt_namespace.svg\n");

    // Example 5: Combine multiple filters
    println!("Example 5: dbt tool + production tag + dbt:// namespace");
    let combined_filter = GraphFilter::new()
        .with_tool(ToolType::Dbt)
        .with_tag("production")
        .with_namespace("dbt://");
    viz.set_filter(combined_filter);

    if let Ok(combined_stats) = viz.stats() {
        println!("  {}", combined_stats);
        if combined_stats.total_nodes > 0 {
            let svg = viz.render_svg()?;
            std::fs::write("filtered_combined.svg", svg)?;
            println!("  → Saved to filtered_combined.svg");
        } else {
            println!("  (No nodes match all filters)");
        }
    }

    Ok(())
}
