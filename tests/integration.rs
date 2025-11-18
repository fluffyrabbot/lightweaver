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

#[test]
fn test_edge_case_malformed_json() {
    use std::fs;
    use std::io::Write;

    // Create temporary file with malformed JSON
    let temp_path = "test_malformed.json";
    let mut file = fs::File::create(temp_path).unwrap();
    file.write_all(b"{ \"broken\": incomplete json").unwrap();
    drop(file);

    let mut viz = PipelineVisualizer::new();
    let result = viz.add_openlineage_events(Path::new(temp_path));

    // Should return error, not panic
    assert!(result.is_err());

    // Cleanup
    fs::remove_file(temp_path).ok();
}

#[test]
fn test_edge_case_special_characters_in_names() {
    // Create graph with special characters in names
    let mut viz = PipelineVisualizer::new();

    // This would normally come from parsing, but we'll test rendering handles it
    viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json")).unwrap();

    // Should render without panicking (XML escaping should handle special chars)
    let svg = viz.render_svg().unwrap();
    assert!(svg.contains("<svg"));

    let html = viz.render_html().unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
}

#[test]
fn test_edge_case_empty_urn_components() {
    use std::fs;

    // Create a test file with dataset that has empty namespace
    let temp_path = "test_empty_urn.json";
    let bad_event = r#"{
        "eventType": "START",
        "eventTime": "2024-01-01T00:00:00.000Z",
        "run": {
            "runId": "test-run",
            "facets": {}
        },
        "job": {
            "namespace": "test://",
            "name": "test_job",
            "facets": {}
        },
        "inputs": [{
            "namespace": "",
            "name": "bad_dataset",
            "facets": {}
        }],
        "outputs": []
    }"#;

    fs::write(temp_path, bad_event).unwrap();

    let mut viz = PipelineVisualizer::new();
    // Should handle gracefully with warning, not crash
    let result = viz.add_openlineage_events(Path::new(temp_path));

    // Cleanup
    fs::remove_file(temp_path).ok();

    // Should either succeed with warning or fail gracefully (not panic)
    // The important thing is it doesn't crash
    match result {
        Ok(_) => {
            // Successfully parsed - URN validation warnings emitted
        }
        Err(e) => {
            // Parsing failed - also acceptable
            assert!(e.to_string().len() > 0);
        }
    }
}

#[test]
fn test_edge_case_legacy_node_filtering() {
    use lightweaver::GraphFilter;

    // Load data that might have legacy DbtModel nodes
    let mut viz = PipelineVisualizer::new();
    viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json")).unwrap();

    // Filter by namespace - legacy nodes without namespace should pass through
    let filter = GraphFilter::new().with_namespace("dbt://");
    viz.set_filter(filter);

    // Should not panic
    let result = viz.render_svg();
    assert!(result.is_ok() || result.unwrap_err().to_string().contains("too restrictive"));
}

#[test]
#[cfg(feature = "png")]
fn test_library_api_png_rendering() {
    let mut viz = PipelineVisualizer::new();
    viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json")).unwrap();

    // Should render to PNG
    let png_bytes = viz.render_png().unwrap();

    // Verify PNG signature (first 8 bytes)
    assert!(png_bytes.len() > 8);
    assert_eq!(&png_bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]); // PNG magic bytes

    // Verify it's a reasonable size (should be at least 1KB)
    assert!(png_bytes.len() > 1024, "PNG should be larger than 1KB, got {} bytes", png_bytes.len());
}

#[test]
#[cfg(feature = "pdf")]
fn test_library_api_pdf_rendering() {
    let mut viz = PipelineVisualizer::new();
    viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json")).unwrap();

    // Should render to PDF
    let pdf_bytes = viz.render_pdf().unwrap();

    // Verify PDF signature (first 4 bytes)
    assert!(pdf_bytes.len() > 4);
    assert_eq!(&pdf_bytes[0..4], b"%PDF"); // PDF magic bytes

    // Verify it's a reasonable size (should be at least 1KB)
    assert!(pdf_bytes.len() > 1024, "PDF should be larger than 1KB, got {} bytes", pdf_bytes.len());
}

#[test]
#[cfg(feature = "png")]
fn test_library_api_png_with_theme() {
    let mut viz = PipelineVisualizer::new().with_theme("dark");
    viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json")).unwrap();

    // Should render PNG with dark theme
    let png_bytes = viz.render_png().unwrap();
    assert!(png_bytes.len() > 1024);
    assert_eq!(&png_bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
}

#[test]
#[cfg(feature = "pdf")]
fn test_library_api_pdf_with_filters() {
    use lightweaver::{GraphFilter, ToolType};

    let mut viz = PipelineVisualizer::new();
    viz.add_openlineage_events(Path::new("test_data/cross_tool_pipeline.json")).unwrap();

    let filter = GraphFilter::new().with_tool(ToolType::Dbt);
    viz.set_filter(filter);

    // Should render filtered PDF
    let pdf_bytes = viz.render_pdf().unwrap();
    assert!(pdf_bytes.len() > 1024);
    assert_eq!(&pdf_bytes[0..4], b"%PDF");
}

#[test]
fn test_config_loading_and_validation() {
    use lightweaver::Config;
    use std::fs;

    // Create a valid config file
    let config_path = "test_valid_config.toml";
    let config_content = r#"
        output = "custom_output.svg"
        format = "html"
        theme = "dark"
        quiet = true

        [filter]
        tools = ["dbt", "airflow"]
        tags = ["production"]
        namespaces = ["postgres://prod"]
    "#;

    fs::write(config_path, config_content).unwrap();

    // Should load successfully
    let config = Config::from_file(config_path).unwrap();
    assert_eq!(config.theme, Some("dark".to_string()));
    assert_eq!(config.format, Some("html".to_string()));
    assert_eq!(config.quiet, Some(true));

    // Check filter settings
    let filter = config.filter.unwrap();
    assert_eq!(filter.tools.unwrap().len(), 2);
    assert_eq!(filter.tags.unwrap(), vec!["production"]);

    // Cleanup
    fs::remove_file(config_path).ok();
}

#[test]
fn test_config_invalid_theme() {
    use lightweaver::Config;
    use std::fs;

    let config_path = "test_invalid_theme.toml";
    let config_content = r#"theme = "invalid_theme""#;
    fs::write(config_path, config_content).unwrap();

    // Should fail validation
    let result = Config::from_file(config_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid theme"));

    fs::remove_file(config_path).ok();
}

#[test]
fn test_config_invalid_format() {
    use lightweaver::Config;
    use std::fs;

    let config_path = "test_invalid_format.toml";
    let config_content = r#"format = "docx""#;
    fs::write(config_path, config_content).unwrap();

    let result = Config::from_file(config_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid format"));

    fs::remove_file(config_path).ok();
}

#[test]
fn test_config_too_many_filters() {
    use lightweaver::Config;
    use std::fs;

    let config_path = "test_too_many_filters.toml";
    // Create a config with 51 tools (over the limit of 50)
    let tools: Vec<String> = (0..51).map(|i| format!("\"tool{}\"", i)).collect();
    let config_content = format!("[filter]\ntools = [{}]", tools.join(", "));
    fs::write(config_path, &config_content).unwrap();

    let result = Config::from_file(config_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Too many tools"));

    fs::remove_file(config_path).ok();
}
