// Graph merger: combines multiple PipelineGraphs into one unified graph
// Key insight: Use dataset URNs to stitch together cross-tool lineage

use crate::graph::{Edge, EdgeKind, NodeId, NodeKind, PipelineGraph};
use std::collections::HashMap;

/// Merge multiple PipelineGraphs into a single unified graph
///
/// Strategy:
/// 1. Collect all nodes from all graphs
/// 2. Build dataset URN registry (namespace:name -> node_id)
/// 3. For each Job node, connect to datasets via URNs (cross-tool stitching!)
/// 4. Preserve all original edges
pub fn merge_graphs(graphs: Vec<PipelineGraph>) -> PipelineGraph {
    let mut merged = PipelineGraph::new();
    let mut dataset_urn_to_id: HashMap<String, NodeId> = HashMap::new();

    // Phase 1: Collect all nodes and build dataset URN registry
    for (graph_idx, graph) in graphs.iter().enumerate() {
        for (node_id, node) in &graph.nodes {
            // Make node IDs unique across graphs by prefixing with graph index
            let unique_id = format!("g{}:{}", graph_idx, node_id);

            // Add node to merged graph
            merged.nodes.insert(unique_id.clone(), node.clone());

            // If it's a dataset, register its URN (keep LAST occurrence for deduplication)
            if let NodeKind::Dataset { namespace, name, .. } = &node.kind {
                let urn = format!("{}:{}", namespace, name);
                dataset_urn_to_id.insert(urn, unique_id.clone());
            }
        }
    }

    // Phase 2: Add all original edges (with prefixed IDs)
    for (graph_idx, graph) in graphs.iter().enumerate() {
        for edge in &graph.edges {
            let prefixed_edge = Edge {
                from: format!("g{}:{}", graph_idx, edge.from),
                to: format!("g{}:{}", graph_idx, edge.to),
                kind: edge.kind.clone(),
            };
            merged.edges.push(prefixed_edge);
        }
    }

    // Phase 3: Cross-tool stitching (the magic!)
    // Collect new edges to add (can't modify while iterating)
    use std::collections::HashSet;
    let mut new_edges = HashSet::new();
    let existing_edges: HashSet<_> = merged.edges.iter().collect();

    // Build edge index for O(1) lookups (optimization: prevents O(n*m) nested loops)
    let mut edges_by_source: HashMap<NodeId, Vec<&Edge>> = HashMap::new();
    for edge in &merged.edges {
        edges_by_source.entry(edge.from.clone()).or_default().push(edge);
    }

    // For each Job node, check if its inputs/outputs match dataset URNs from other sources
    let jobs: Vec<_> = merged
        .nodes
        .iter()
        .filter(|(_, node)| matches!(node.kind, NodeKind::Job { .. }))
        .map(|(id, _)| id.clone())
        .collect();

    for job_id in jobs {
        // Look for edges where this job reads from or writes to datasets
        let job_edges: Vec<_> = merged
            .edges
            .iter()
            .filter(|e| e.from == job_id || e.to == job_id)
            .cloned()
            .collect();

        for edge in job_edges {
            // If job writes to a dataset, check if another job reads from same dataset URN
            if edge.kind == EdgeKind::WritesTo {
                let dataset_id = &edge.to;
                if let Some(dataset_node) = merged.nodes.get(dataset_id) {
                    if let NodeKind::Dataset { namespace, name, .. } = &dataset_node.kind {
                        let urn = format!("{}:{}", namespace, name);

                        // Find other datasets with same URN (different node_id)
                        for (other_id, other_node) in &merged.nodes {
                            if other_id == dataset_id {
                                continue;
                            }
                            if let NodeKind::Dataset { namespace: ns2, name: name2, .. } = &other_node.kind {
                                let other_urn = format!("{}:{}", ns2, name2);
                                if urn == other_urn {
                                    // Found matching dataset! Connect all readers of other_id to this one
                                    // Use edge index for O(1) lookup instead of O(n) filter
                                    let readers: Vec<_> = edges_by_source
                                        .get(other_id)
                                        .map(|edges| edges.iter()
                                            .filter(|e| e.kind == EdgeKind::ReadsFrom)
                                            .map(|e| e.to.clone())
                                            .collect())
                                        .unwrap_or_default();

                                    for reader_job in readers {
                                        // Create cross-tool edge: job_id -> dataset_id <- reader_job
                                        // This is the lineage connection!
                                        let new_edge = Edge {
                                            from: dataset_id.clone(),
                                            to: reader_job,
                                            kind: EdgeKind::ReadsFrom,
                                        };

                                        // Only add if edge doesn't already exist (O(1) lookup with HashSet)
                                        if !existing_edges.contains(&new_edge) {
                                            new_edges.insert(new_edge);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Add all collected edges
    for edge in new_edges {
        merged.add_edge(edge);
    }

    merged
}

/// Deduplicate dataset nodes with identical URNs
/// Keeps the first occurrence, redirects all edges to it
pub fn deduplicate_datasets(graph: &mut PipelineGraph) {
    let mut urn_to_canonical: HashMap<String, NodeId> = HashMap::new();
    let mut nodes_to_remove: Vec<NodeId> = Vec::new();
    let mut edge_redirects: HashMap<NodeId, NodeId> = HashMap::new();

    // Find canonical dataset for each URN
    for (node_id, node) in &graph.nodes {
        if let NodeKind::Dataset { namespace, name, .. } = &node.kind {
            let urn = format!("{}:{}", namespace, name);

            if let Some(canonical_id) = urn_to_canonical.get(&urn) {
                // Duplicate found - mark for removal and redirect edges
                nodes_to_remove.push(node_id.clone());
                edge_redirects.insert(node_id.clone(), canonical_id.clone());
            } else {
                // First occurrence - make it canonical
                urn_to_canonical.insert(urn, node_id.clone());
            }
        }
    }

    // Remove duplicate nodes
    for node_id in nodes_to_remove {
        graph.nodes.remove(&node_id);
    }

    // Redirect edges
    for edge in &mut graph.edges {
        if let Some(canonical_from) = edge_redirects.get(&edge.from) {
            edge.from = canonical_from.clone();
        }
        if let Some(canonical_to) = edge_redirects.get(&edge.to) {
            edge.to = canonical_to.clone();
        }
    }

    // Remove duplicate edges (using HashSet for O(n) instead of O(n²))
    use std::collections::HashSet;
    let unique_edges: HashSet<Edge> = graph.edges.drain(..).collect();
    graph.edges = unique_edges.into_iter().collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Node, NodeMetadata, ToolType};

    #[test]
    fn test_merge_two_graphs() {
        let mut graph1 = PipelineGraph::new();
        let mut graph2 = PipelineGraph::new();

        // Graph 1: dbt job writes to postgres dataset
        graph1.add_node(Node {
            id: "dbt_job".to_string(),
            kind: NodeKind::Job {
                namespace: "dbt://project".to_string(),
                name: "models.customers".to_string(),
                tool: ToolType::Dbt,
                facets: HashMap::new(),
            },
            metadata: NodeMetadata {
                name: "customers".to_string(),
                description: None,
                tags: vec![],
                owner: None,
            },
        });

        graph1.add_node(Node {
            id: "dataset1".to_string(),
            kind: NodeKind::Dataset {
                namespace: "postgres://prod".to_string(),
                name: "public.customers".to_string(),
                facets: HashMap::new(),
            },
            metadata: NodeMetadata {
                name: "customers".to_string(),
                description: None,
                tags: vec![],
                owner: None,
            },
        });

        graph1.add_edge(Edge {
            from: "dbt_job".to_string(),
            to: "dataset1".to_string(),
            kind: EdgeKind::WritesTo,
        });

        // Graph 2: airflow job reads from postgres dataset (same URN!)
        graph2.add_node(Node {
            id: "airflow_job".to_string(),
            kind: NodeKind::Job {
                namespace: "airflow://prod".to_string(),
                name: "export_task".to_string(),
                tool: ToolType::Airflow,
                facets: HashMap::new(),
            },
            metadata: NodeMetadata {
                name: "export_task".to_string(),
                description: None,
                tags: vec![],
                owner: None,
            },
        });

        graph2.add_node(Node {
            id: "dataset2".to_string(),
            kind: NodeKind::Dataset {
                namespace: "postgres://prod".to_string(),
                name: "public.customers".to_string(),  // Same URN!
                facets: HashMap::new(),
            },
            metadata: NodeMetadata {
                name: "customers".to_string(),
                description: None,
                tags: vec![],
                owner: None,
            },
        });

        graph2.add_edge(Edge {
            from: "dataset2".to_string(),
            to: "airflow_job".to_string(),
            kind: EdgeKind::ReadsFrom,
        });

        let merged = merge_graphs(vec![graph1, graph2]);

        // Should have 4 nodes: 2 jobs + 2 datasets
        assert_eq!(merged.node_count(), 4);

        // Should have original edges
        assert!(merged.edges.len() >= 2);
    }

    #[test]
    fn test_deduplicate_datasets() {
        let mut graph = PipelineGraph::new();

        // Add two datasets with same URN
        graph.add_node(Node {
            id: "dataset1".to_string(),
            kind: NodeKind::Dataset {
                namespace: "postgres://prod".to_string(),
                name: "public.customers".to_string(),
                facets: HashMap::new(),
            },
            metadata: NodeMetadata {
                name: "customers".to_string(),
                description: None,
                tags: vec![],
                owner: None,
            },
        });

        graph.add_node(Node {
            id: "dataset2".to_string(),
            kind: NodeKind::Dataset {
                namespace: "postgres://prod".to_string(),
                name: "public.customers".to_string(),  // Duplicate!
                facets: HashMap::new(),
            },
            metadata: NodeMetadata {
                name: "customers".to_string(),
                description: None,
                tags: vec![],
                owner: None,
            },
        });

        deduplicate_datasets(&mut graph);

        // Should only have 1 dataset node now
        assert_eq!(graph.node_count(), 1);
    }
}
