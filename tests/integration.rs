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
    let mut empty_viz = PipelineVisualizer::new();
    let result = empty_viz.render_svg();
    assert!(result.is_err());
}
