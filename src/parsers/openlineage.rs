// Parser for OpenLineage JSON events
// Spec: https://github.com/OpenLineage/OpenLineage/blob/main/spec/OpenLineage.md

use crate::graph::{Edge, EdgeKind, Node, NodeKind, NodeMetadata, PipelineGraph, ToolType};
use crate::openlineage::RunEvent;
use crate::parsers::ParseError;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Parse OpenLineage events from a JSON file
/// Supports both newline-delimited JSON (NDJSON) and JSON arrays
pub fn parse_events(path: &Path) -> Result<PipelineGraph, ParseError> {
    // Validate file size to prevent OOM (100MB limit)
    const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;
    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(ParseError::InvalidFormat(
            format!("File too large: {} bytes (max: {} MB). Consider splitting your events file.",
                    metadata.len(), MAX_FILE_SIZE / 1024 / 1024)
        ));
    }

    let content = fs::read_to_string(path)?;

    // Try to parse as JSON array first
    if let Ok(events) = serde_json::from_str::<Vec<RunEvent>>(&content) {
        return build_graph_from_events(events);
    }

    // Try parsing as newline-delimited JSON
    let mut events = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let event: RunEvent = serde_json::from_str(line)?;
        events.push(event);
    }

    build_graph_from_events(events)
}

/// Build a PipelineGraph from OpenLineage RunEvents
fn build_graph_from_events(events: Vec<RunEvent>) -> Result<PipelineGraph, ParseError> {
    let mut graph = PipelineGraph::new();
    let mut dataset_registry: HashMap<String, String> = HashMap::new(); // URN -> node_id
    let mut job_registry: HashMap<String, String> = HashMap::new(); // job_key -> node_id

    for event in events {
        // Extract job information
        let job_key = format!("{}:{}", event.job.namespace, event.job.name);
        let job_idx = job_registry.len();
        let job_node_id = job_registry
            .entry(job_key.clone())
            .or_insert_with(|| format!("job_{}", job_idx));

        // Determine tool type from namespace
        let tool = detect_tool_type(&event.job.namespace);

        // Only add job node once (on first event)
        if !graph.nodes.contains_key(job_node_id) {
            let job_node = Node {
                id: job_node_id.clone(),
                kind: NodeKind::Job {
                    namespace: event.job.namespace.clone(),
                    name: event.job.name.clone(),
                    tool,
                    facets: event.job.facets.clone(),
                },
                metadata: NodeMetadata {
                    name: extract_display_name(&event.job.name),
                    description: extract_description(&event.job.facets),
                    tags: extract_tags(&event.job.facets),
                    owner: extract_owner(&event.job.facets),
                },
            };
            graph.add_node(job_node);
        }

        // Process input datasets
        for input in &event.inputs {
            let dataset_urn = format!("{}:{}", input.namespace, input.name);
            let dataset_idx = dataset_registry.len();
            let dataset_node_id = dataset_registry
                .entry(dataset_urn.clone())
                .or_insert_with(|| format!("dataset_{}", dataset_idx));

            // Add dataset node if not exists
            if !graph.nodes.contains_key(dataset_node_id) {
                let dataset_node = Node {
                    id: dataset_node_id.clone(),
                    kind: NodeKind::Dataset {
                        namespace: input.namespace.clone(),
                        name: input.name.clone(),
                        facets: input.facets.clone(),
                    },
                    metadata: NodeMetadata {
                        name: extract_display_name(&input.name),
                        description: extract_description(&input.facets),
                        tags: Vec::new(),
                        owner: None,
                    },
                };
                graph.add_node(dataset_node);
            }

            // Add edge: Dataset -> Job (job reads from dataset)
            graph.add_edge(Edge {
                from: dataset_node_id.clone(),
                to: job_node_id.clone(),
                kind: EdgeKind::ReadsFrom,
            });
        }

        // Process output datasets
        for output in &event.outputs {
            let dataset_urn = format!("{}:{}", output.namespace, output.name);
            let dataset_idx = dataset_registry.len();
            let dataset_node_id = dataset_registry
                .entry(dataset_urn.clone())
                .or_insert_with(|| format!("dataset_{}", dataset_idx));

            // Add dataset node if not exists
            if !graph.nodes.contains_key(dataset_node_id) {
                let dataset_node = Node {
                    id: dataset_node_id.clone(),
                    kind: NodeKind::Dataset {
                        namespace: output.namespace.clone(),
                        name: output.name.clone(),
                        facets: output.facets.clone(),
                    },
                    metadata: NodeMetadata {
                        name: extract_display_name(&output.name),
                        description: extract_description(&output.facets),
                        tags: Vec::new(),
                        owner: None,
                    },
                };
                graph.add_node(dataset_node);
            }

            // Add edge: Job -> Dataset (job writes to dataset)
            graph.add_edge(Edge {
                from: job_node_id.clone(),
                to: dataset_node_id.clone(),
                kind: EdgeKind::WritesTo,
            });
        }
    }

    Ok(graph)
}

/// Detect tool type from OpenLineage namespace
fn detect_tool_type(namespace: &str) -> ToolType {
    if namespace.starts_with("dbt://") {
        ToolType::Dbt
    } else if namespace.starts_with("airflow://") {
        ToolType::Airflow
    } else if namespace.starts_with("spark://") {
        ToolType::Spark
    } else if namespace.starts_with("fivetran://") {
        ToolType::Fivetran
    } else if namespace.starts_with("dagster://") {
        ToolType::Dagster
    } else if namespace.starts_with("prefect://") {
        ToolType::Prefect
    } else if namespace.starts_with("flink://") {
        ToolType::Flink
    } else if namespace.starts_with("kafka://") {
        ToolType::Kafka
    } else {
        ToolType::Custom
    }
}

/// Extract display name from full path
/// e.g., "models.staging.stg_customers" -> "stg_customers"
/// e.g., "public.dim_customers" -> "dim_customers"
fn extract_display_name(full_name: &str) -> String {
    full_name
        .split('.')
        .next_back()
        .unwrap_or(full_name)
        .to_string()
}

/// Extract description from facets
fn extract_description(facets: &HashMap<String, Value>) -> Option<String> {
    facets
        .get("documentation")
        .and_then(|v| v.get("description"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Extract tags from facets (dbt-specific)
fn extract_tags(facets: &HashMap<String, Value>) -> Vec<String> {
    facets
        .get("dbt")
        .and_then(|v| v.get("tags"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Extract owner from facets
fn extract_owner(facets: &HashMap<String, Value>) -> Option<String> {
    facets
        .get("ownership")
        .and_then(|v| v.get("owners"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openlineage::{Dataset, Job, Run};

    #[test]
    fn test_build_simple_graph() {
        let events = vec![RunEvent {
            event_type: crate::openlineage::EventType::Complete,
            event_time: "2025-01-17T12:00:00Z".to_string(),
            run: Run::new("test-run-id"),
            job: Job::new("dbt://jaffle_shop", "models.customers"),
            inputs: vec![Dataset::new("postgres://prod", "raw.customers")],
            outputs: vec![Dataset::new("postgres://prod", "analytics.customers")],
            producer: "test".to_string(),
            schema_url: None,
        }];

        let graph = build_graph_from_events(events).unwrap();

        // Should have 1 job + 2 datasets = 3 nodes
        assert_eq!(graph.node_count(), 3);

        // Should have 2 edges (input->job, job->output)
        assert_eq!(graph.edge_count(), 2);
    }

    #[test]
    fn test_detect_tool_type() {
        assert_eq!(detect_tool_type("dbt://project"), ToolType::Dbt);
        assert_eq!(detect_tool_type("airflow://prod"), ToolType::Airflow);
        assert_eq!(detect_tool_type("spark://cluster"), ToolType::Spark);
        assert_eq!(detect_tool_type("custom://foo"), ToolType::Custom);
    }

    #[test]
    fn test_extract_display_name() {
        assert_eq!(extract_display_name("models.staging.stg_customers"), "stg_customers");
        assert_eq!(extract_display_name("public.dim_customers"), "dim_customers");
        assert_eq!(extract_display_name("customers"), "customers");
    }

    #[test]
    fn test_multi_tool_graph() {
        let events = vec![
            RunEvent {
                event_type: crate::openlineage::EventType::Complete,
                event_time: "2025-01-17T12:00:00Z".to_string(),
                run: Run::new("run-1"),
                job: Job::new("dbt://jaffle", "models.customers"),
                inputs: vec![],
                outputs: vec![Dataset::new("postgres://prod", "analytics.customers")],
                producer: "test".to_string(),
                schema_url: None,
            },
            RunEvent {
                event_type: crate::openlineage::EventType::Complete,
                event_time: "2025-01-17T12:01:00Z".to_string(),
                run: Run::new("run-2"),
                job: Job::new("airflow://prod", "etl_dag.load_customers"),
                inputs: vec![Dataset::new("postgres://prod", "analytics.customers")],
                outputs: vec![Dataset::new("s3://bucket", "data/customers.parquet")],
                producer: "test".to_string(),
                schema_url: None,
            },
        ];

        let graph = build_graph_from_events(events).unwrap();

        // 2 jobs + 2 unique datasets = 4 nodes
        assert_eq!(graph.node_count(), 4);

        // dbt: 0 in, 1 out = 1 edge
        // airflow: 1 in, 1 out = 2 edges
        // Total = 3 edges
        assert_eq!(graph.edge_count(), 3);

        // Verify tools
        let dbt_nodes = graph.nodes_by_tool(ToolType::Dbt);
        assert_eq!(dbt_nodes.len(), 1);

        let airflow_nodes = graph.nodes_by_tool(ToolType::Airflow);
        assert_eq!(airflow_nodes.len(), 1);
    }
}
