//! Example: Creating a custom parser plugin for Lightweaver
//!
//! This example shows how to create a custom parser plugin that can read
//! your own data format and convert it into a pipeline graph.
//!
//! Run with: cargo run --example custom_parser

use lightweaver::{
    graph::{Edge, EdgeKind, Node, NodeKind, NodeMetadata, PipelineGraph, ToolType},
    parsers::ParseError,
    Parser, PipelineVisualizer,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Example: CSV parser for simple pipeline definitions
///
/// Expected format:
/// ```csv
/// job_name,depends_on,tool
/// extract_data,,airflow
/// transform_data,extract_data,dbt
/// load_data,transform_data,airflow
/// ```
struct CsvPipelineParser;

impl Parser for CsvPipelineParser {
    fn name(&self) -> &str {
        "csv_pipeline_parser"
    }

    fn parse(&self, path: &Path) -> Result<PipelineGraph, ParseError> {
        let content = fs::read_to_string(path)?;
        let mut graph = PipelineGraph::new();

        // Skip header line
        for (idx, line) in content.lines().skip(1).enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 3 {
                return Err(ParseError::InvalidFormat(format!(
                    "Invalid CSV line {}: expected 3 columns (job_name,depends_on,tool)",
                    idx + 2
                )));
            }

            let job_name = parts[0].trim();
            let depends_on = parts[1].trim();
            let tool_str = parts[2].trim();

            let tool = match tool_str.to_lowercase().as_str() {
                "dbt" => ToolType::Dbt,
                "airflow" => ToolType::Airflow,
                "spark" => ToolType::Spark,
                "fivetran" => ToolType::Fivetran,
                "dagster" => ToolType::Dagster,
                "prefect" => ToolType::Prefect,
                _ => ToolType::Custom,
            };

            // Add job node
            let job_id = format!("job_{}", job_name);
            graph.add_node(Node {
                id: job_id.clone(),
                kind: NodeKind::Job {
                    namespace: format!("{}://pipeline", tool_str),
                    name: job_name.to_string(),
                    tool,
                    facets: HashMap::new(),
                },
                metadata: NodeMetadata {
                    name: job_name.to_string(),
                    description: Some(format!("Job from CSV: {}", job_name)),
                    tags: vec!["csv-imported".to_string()],
                    owner: None,
                },
            });

            // Add dependency edge if specified
            if !depends_on.is_empty() {
                let parent_id = format!("job_{}", depends_on);

                // Make sure parent exists
                if !graph.nodes.contains_key(&parent_id) {
                    return Err(ParseError::InvalidFormat(format!(
                        "Job '{}' depends on '{}', but '{}' is not defined",
                        job_name, depends_on, depends_on
                    )));
                }

                graph.add_edge(Edge {
                    from: parent_id,
                    to: job_id,
                    kind: EdgeKind::DependsOn,
                });
            }
        }

        Ok(graph)
    }

    fn supported_extensions(&self) -> Vec<&str> {
        vec!["csv", "pipeline"]
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Parser Plugin Example ===\n");

    // Create sample CSV file
    let sample_csv = "\
job_name,depends_on,tool
extract_salesforce,,airflow
extract_postgres,,airflow
staging_customers,extract_postgres,dbt
staging_orders,extract_postgres,dbt
dim_customers,staging_customers,dbt
fct_orders,staging_orders,dbt
export_to_s3,fct_orders,airflow
";

    fs::write("/tmp/pipeline.csv", sample_csv)?;
    println!("Created sample CSV file at /tmp/pipeline.csv");
    println!("Content:\n{}\n", sample_csv);

    // Create visualizer and register custom parser
    let mut viz = PipelineVisualizer::new();

    println!("Registering custom CSV parser plugin...");
    viz.register_parser(Box::new(CsvPipelineParser));

    // Load the CSV file using the plugin
    println!("Loading pipeline from CSV...\n");
    viz.add_source(Path::new("/tmp/pipeline.csv"))?;

    // Get statistics
    let stats = viz.stats()?;
    println!("Pipeline loaded successfully!");
    println!("  Jobs: {}", stats.job_nodes);
    println!("  Edges: {}", stats.total_edges);
    println!("  Total nodes: {}\n", stats.total_nodes);

    // Generate SVG visualization
    println!("Generating SVG visualization...");
    let svg = viz.render_svg()?;
    let output_path = "/tmp/custom_pipeline.svg";
    fs::write(output_path, svg)?;
    println!("✓ Saved to {}", output_path);

    // Also generate interactive HTML
    println!("Generating interactive HTML...");
    let html = viz.render_html()?;
    let html_path = "/tmp/custom_pipeline.html";
    fs::write(html_path, html)?;
    println!("✓ Saved to {}", html_path);

    println!("\n=== Success! ===");
    println!("Your custom CSV parser successfully converted the pipeline definition");
    println!("into a visual graph. Open {} in your browser to explore!", html_path);

    Ok(())
}
