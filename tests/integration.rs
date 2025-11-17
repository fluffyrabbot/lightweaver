//! Integration tests for Lightweaver library API

use lightweaver::PipelineVisualizer;
use std::path::Path;

#[test]
fn test_library_api_basic() {
    let mut viz = PipelineVisualizer::new();

    // Add test data
    let result = viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json"));
    assert!(result.is_ok());

    // Get stats
    let stats = viz.stats();
    assert!(stats.is_ok());
    let stats = stats.unwrap();
    assert_eq!(stats.source_count, 1);
    assert!(stats.total_nodes > 0);

    // Render should work
    let svg = viz.render_svg();
    assert!(svg.is_ok());
    assert!(svg.unwrap().contains("<svg"));
}

#[test]
fn test_library_api_multi_source() {
    let mut viz = PipelineVisualizer::new();

    // Add multiple sources
    viz.add_openlineage_events(Path::new("test_data/dbt_events.json")).unwrap();
    viz.add_openlineage_events(Path::new("test_data/airflow_events.json")).unwrap();

    // Should merge
    let stats = viz.stats().unwrap();
    assert_eq!(stats.source_count, 2);

    // Should render merged graph
    let svg = viz.render_svg().unwrap();
    assert!(svg.contains("</svg>"));
}

#[test]
fn test_library_api_with_theme() {
    let mut viz = PipelineVisualizer::new()
        .with_theme("dark");

    viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json")).unwrap();

    let svg = viz.render_svg().unwrap();
    // Should successfully render SVG with theme
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn test_library_api_error_handling() {
    let mut viz = PipelineVisualizer::new();

    // Non-existent file should error
    let result = viz.add_openlineage_events(Path::new("nonexistent.json"));
    assert!(result.is_err());

    // Empty visualizer should error on render
    let empty_viz = PipelineVisualizer::new();
    let result = empty_viz.render_svg();
    assert!(result.is_err());
}

#[test]
fn test_library_api_filtering() {
    use lightweaver::{GraphFilter, ToolType};

    // Load multi-tool pipeline
    let mut viz = PipelineVisualizer::new();
    viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json")).unwrap();

    // Get unfiltered stats
    let stats = viz.stats().unwrap();
    let total_nodes = stats.total_nodes;
    assert!(total_nodes > 0);

    // Apply filter to show only dbt
    let filter = GraphFilter::new().with_tool(ToolType::Dbt);
    viz.set_filter(filter);

    // Should have fewer nodes after filtering
    let filtered_stats = viz.stats().unwrap();
    assert!(filtered_stats.total_nodes <= total_nodes);

    // Should be able to render filtered graph
    let svg = viz.render_svg().unwrap();
    assert!(svg.contains("<svg"));

    // Clear filter
    viz.clear_filter();
    let cleared_stats = viz.stats().unwrap();
    assert_eq!(cleared_stats.total_nodes, total_nodes);
}

#[test]
fn test_library_api_combined_filters() {
    use lightweaver::{GraphFilter, ToolType};

    // Load test data
    let mut viz = PipelineVisualizer::new();
    viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json")).unwrap();

    // Test combining multiple tools (OR logic)
    let multi_tool_filter = GraphFilter::new()
        .with_tool(ToolType::Dbt)
        .with_tool(ToolType::Airflow);
    viz.set_filter(multi_tool_filter);

    let stats = viz.stats().unwrap();
    // Should include both dbt and airflow nodes
    assert!(stats.total_nodes > 0);
    let svg = viz.render_svg().unwrap();
    assert!(svg.contains("<svg"));

    // Test tool + namespace combination
    let combined_filter = GraphFilter::new()
        .with_tool(ToolType::Dbt)
        .with_namespace("dbt://");
    viz.set_filter(combined_filter);

    let combined_stats = viz.stats().unwrap();
    assert!(combined_stats.total_nodes > 0);
    assert!(combined_stats.total_nodes <= stats.total_nodes);
}

#[test]
fn test_filter_empty_result_error() {
    use lightweaver::{GraphFilter, ToolType};

    let mut viz = PipelineVisualizer::new();
    viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json")).unwrap();

    // Apply filter that matches nothing
    let impossible_filter = GraphFilter::new()
        .with_tool(ToolType::Kafka)  // No Kafka nodes in test data
        .with_tag("nonexistent");
    viz.set_filter(impossible_filter);

    // Should error with filter-specific message
    let result = viz.render_svg();
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("after applying filters"));
    assert!(err_msg.contains("too restrictive"));
}

#[test]
fn test_library_api_html_rendering() {
    let mut viz = PipelineVisualizer::new();
    viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json")).unwrap();

    // Should render to HTML
    let html = viz.render_html().unwrap();

    // Verify HTML structure
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<svg"));
    assert!(html.contains("svg-pan-zoom"));
    assert!(html.contains("const nodes = ["));
    assert!(html.contains("</html>"));

    // Should contain node data
    assert!(html.contains("\"type\""));
    assert!(html.contains("\"tool\""));
}

#[test]
fn test_library_api_html_with_dark_theme() {
    let mut viz = PipelineVisualizer::new()
        .with_theme("dark");
    viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json")).unwrap();

    let html = viz.render_html().unwrap();

    // Should have dark theme class
    assert!(html.contains(r#"class="dark""#) || html.contains("body.dark"));
}

#[test]
fn test_library_api_html_with_filters() {
    use lightweaver::{GraphFilter, ToolType};

    let mut viz = PipelineVisualizer::new();
    viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json")).unwrap();

    let filter = GraphFilter::new().with_tool(ToolType::Dbt);
    viz.set_filter(filter);

    // Should render HTML with filtered nodes
    let html = viz.render_html().unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("const nodes = ["));
}
