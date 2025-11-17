// Core graph algorithms for layout

use crate::graph::{NodeId, PipelineGraph};
use std::collections::{HashMap, HashSet, VecDeque};

/// Compute topological layers using Kahn's algorithm
/// Returns nodes grouped by layer (layer 0 = sources, layer N = sinks)
pub fn topological_layers(graph: &PipelineGraph) -> Result<Vec<Vec<NodeId>>, Vec<NodeId>> {
    if graph.node_count() == 0 {
        return Ok(Vec::new());
    }

    // Build adjacency list and compute in-degrees
    let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
    let mut out_edges: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

    // Initialize all nodes with 0 in-degree
    for node_id in graph.nodes.keys() {
        in_degree.insert(node_id.clone(), 0);
        out_edges.insert(node_id.clone(), Vec::new());
    }

    // Count in-degrees and build adjacency list
    // Skip edges that reference nodes not in the graph (e.g., tests, other filtered nodes)
    for edge in &graph.edges {
        // Warn if edge references non-existent nodes (helps catch data quality issues)
        let from_exists = in_degree.contains_key(&edge.from);
        let to_exists = in_degree.contains_key(&edge.to);

        if !from_exists || !to_exists {
            eprintln!("⚠️  Warning: Edge references non-existent node(s): {} -> {} (from_exists: {}, to_exists: {})",
                      edge.from, edge.to, from_exists, to_exists);
        }

        if let Some(to_degree) = in_degree.get_mut(&edge.to) {
            *to_degree += 1;
        }
        if let Some(from_edges) = out_edges.get_mut(&edge.from) {
            // Only add edge if target node exists
            if in_degree.contains_key(&edge.to) {
                from_edges.push(edge.to.clone());
            }
        }
    }

    let mut layers: Vec<Vec<NodeId>> = Vec::new();
    let mut current_layer: VecDeque<NodeId> = VecDeque::new();
    let mut processed = 0;

    // Start with nodes that have no incoming edges
    for (node_id, &degree) in &in_degree {
        if degree == 0 {
            current_layer.push_back(node_id.clone());
        }
    }

    while !current_layer.is_empty() {
        let layer: Vec<NodeId> = current_layer.drain(..).collect();
        processed += layer.len();

        let mut next_layer: Vec<NodeId> = Vec::new();

        for node_id in &layer {
            if let Some(neighbors) = out_edges.get(node_id) {
                for neighbor in neighbors {
                    // Safe: only process edges where neighbor exists in the graph
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            next_layer.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        layers.push(layer);

        for node in next_layer {
            current_layer.push_back(node);
        }
    }

    // If we didn't process all nodes, there's a cycle
    if processed != graph.node_count() {
        let cycle_nodes = detect_cycle_nodes(graph, &in_degree);
        return Err(cycle_nodes);
    }

    Ok(layers)
}

/// Detect which nodes are part of cycles
fn detect_cycle_nodes(_graph: &PipelineGraph, in_degree: &HashMap<NodeId, usize>) -> Vec<NodeId> {
    in_degree
        .iter()
        .filter(|(_, &degree)| degree > 0)
        .map(|(node_id, _)| node_id.clone())
        .collect()
}

/// Detect cycles in the graph using DFS
pub fn detect_cycles(graph: &PipelineGraph) -> Vec<Vec<NodeId>> {
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut cycles = Vec::new();

    // Build adjacency list
    let mut adj_list: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for node_id in graph.nodes.keys() {
        adj_list.insert(node_id.clone(), Vec::new());
    }
    for edge in &graph.edges {
        // Only add edges where both nodes exist
        if let Some(adj) = adj_list.get_mut(&edge.from) {
            adj.push(edge.to.clone());
        }
    }

    for node_id in graph.nodes.keys() {
        if !visited.contains(node_id) {
            let mut path = Vec::new();
            if dfs_cycle(
                node_id,
                &adj_list,
                &mut visited,
                &mut rec_stack,
                &mut path,
                &mut cycles,
            ) {
                // Cycle detected, already added to cycles
            }
        }
    }

    cycles
}

fn dfs_cycle(
    node: &NodeId,
    adj_list: &HashMap<NodeId, Vec<NodeId>>,
    visited: &mut HashSet<NodeId>,
    rec_stack: &mut HashSet<NodeId>,
    path: &mut Vec<NodeId>,
    cycles: &mut Vec<Vec<NodeId>>,
) -> bool {
    visited.insert(node.clone());
    rec_stack.insert(node.clone());
    path.push(node.clone());

    if let Some(neighbors) = adj_list.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                if dfs_cycle(neighbor, adj_list, visited, rec_stack, path, cycles) {
                    return true;
                }
            } else if rec_stack.contains(neighbor) {
                // Found cycle - extract it from path
                let cycle_start = path.iter().position(|n| n == neighbor).unwrap();
                let cycle = path[cycle_start..].to_vec();
                cycles.push(cycle);
                return true;
            }
        }
    }

    path.pop();
    rec_stack.remove(node);
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Edge, EdgeKind, Node, NodeKind, NodeMetadata};

    fn make_test_node(id: &str) -> Node {
        Node {
            id: id.to_string(),
            kind: NodeKind::DbtModel {
                materialization: "table".to_string(),
                rows: None,
            },
            metadata: NodeMetadata {
                name: id.to_string(),
                description: None,
                tags: Vec::new(),
                owner: None,
            },
        }
    }

    #[test]
    fn test_simple_dag() {
        let mut graph = PipelineGraph::new();
        graph.add_node(make_test_node("a"));
        graph.add_node(make_test_node("b"));
        graph.add_node(make_test_node("c"));

        graph.add_edge(Edge {
            from: "a".to_string(),
            to: "b".to_string(),
            kind: EdgeKind::DependsOn,
        });
        graph.add_edge(Edge {
            from: "b".to_string(),
            to: "c".to_string(),
            kind: EdgeKind::DependsOn,
        });

        let layers = topological_layers(&graph).unwrap();
        assert_eq!(layers.len(), 3);
        assert_eq!(layers[0], vec!["a"]);
        assert_eq!(layers[1], vec!["b"]);
        assert_eq!(layers[2], vec!["c"]);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = PipelineGraph::new();
        graph.add_node(make_test_node("a"));
        graph.add_node(make_test_node("b"));
        graph.add_node(make_test_node("c"));

        graph.add_edge(Edge {
            from: "a".to_string(),
            to: "b".to_string(),
            kind: EdgeKind::DependsOn,
        });
        graph.add_edge(Edge {
            from: "b".to_string(),
            to: "c".to_string(),
            kind: EdgeKind::DependsOn,
        });
        graph.add_edge(Edge {
            from: "c".to_string(),
            to: "a".to_string(),
            kind: EdgeKind::DependsOn,
        });

        let result = topological_layers(&graph);
        assert!(result.is_err());

        let cycles = detect_cycles(&graph);
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_diamond_dag() {
        let mut graph = PipelineGraph::new();
        graph.add_node(make_test_node("a"));
        graph.add_node(make_test_node("b"));
        graph.add_node(make_test_node("c"));
        graph.add_node(make_test_node("d"));

        // Diamond: a -> b,c -> d
        graph.add_edge(Edge {
            from: "a".to_string(),
            to: "b".to_string(),
            kind: EdgeKind::DependsOn,
        });
        graph.add_edge(Edge {
            from: "a".to_string(),
            to: "c".to_string(),
            kind: EdgeKind::DependsOn,
        });
        graph.add_edge(Edge {
            from: "b".to_string(),
            to: "d".to_string(),
            kind: EdgeKind::DependsOn,
        });
        graph.add_edge(Edge {
            from: "c".to_string(),
            to: "d".to_string(),
            kind: EdgeKind::DependsOn,
        });

        let layers = topological_layers(&graph).unwrap();
        assert_eq!(layers.len(), 3);
        assert_eq!(layers[0], vec!["a"]);
        assert_eq!(layers[1].len(), 2); // b and c (order doesn't matter)
        assert_eq!(layers[2], vec!["d"]);
    }
}
