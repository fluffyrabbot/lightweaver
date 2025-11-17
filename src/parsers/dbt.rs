// Parser for dbt manifest.json

use crate::graph::{Edge, EdgeKind, Node, NodeKind, NodeMetadata, PipelineGraph};
use crate::parsers::ParseError;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn parse_manifest(path: &Path) -> Result<PipelineGraph, ParseError> {
    let content = fs::read_to_string(path)?;
    let manifest: Value = serde_json::from_str(&content)?;

    let mut graph = PipelineGraph::new();

    // Parse nodes (models, seeds, snapshots, etc.)
    if let Some(nodes) = manifest["nodes"].as_object() {
        for (node_id, node_data) in nodes {
            // Skip tests and other non-model nodes
            let resource_type = node_data["resource_type"]
                .as_str()
                .ok_or_else(|| ParseError::MissingField("resource_type".to_string()))?;

            if !matches!(resource_type, "model" | "seed" | "snapshot") {
                continue;
            }

            let name = node_data["name"]
                .as_str()
                .ok_or_else(|| ParseError::MissingField("name".to_string()))?
                .to_string();

            let materialization = node_data["config"]["materialized"]
                .as_str()
                .unwrap_or("view")
                .to_string();

            let description = node_data["description"].as_str().map(|s| s.to_string());

            let tags: Vec<String> = node_data["tags"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let node = Node {
                id: node_id.clone(),
                kind: NodeKind::DbtModel {
                    materialization,
                    rows: None, // Can be enriched from catalog later
                },
                metadata: NodeMetadata {
                    name,
                    description,
                    tags,
                    owner: None,
                },
            };

            graph.add_node(node);
        }
    }

    // Parse sources
    if let Some(sources) = manifest["sources"].as_object() {
        for (source_id, source_data) in sources {
            let name = source_data["name"]
                .as_str()
                .ok_or_else(|| ParseError::MissingField("name".to_string()))?
                .to_string();

            let database = source_data["database"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();

            let schema = source_data["schema"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();

            let description = source_data["description"].as_str().map(|s| s.to_string());

            let tags: Vec<String> = source_data["tags"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let node = Node {
                id: source_id.clone(),
                kind: NodeKind::DbtSource { database, schema },
                metadata: NodeMetadata {
                    name,
                    description,
                    tags,
                    owner: None,
                },
            };

            graph.add_node(node);
        }
    }

    // Parse dependencies
    if let Some(nodes) = manifest["nodes"].as_object() {
        for (node_id, node_data) in nodes {
            if let Some(depends_on) = node_data["depends_on"]["nodes"].as_array() {
                for dep in depends_on {
                    if let Some(dep_id) = dep.as_str() {
                        graph.add_edge(Edge {
                            from: dep_id.to_string(),
                            to: node_id.clone(),
                            kind: EdgeKind::DependsOn,
                        });
                    }
                }
            }
        }
    }

    Ok(graph)
}

// Tests will be added after we have real dbt projects to test against
