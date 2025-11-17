// Hierarchical (layered) layout for DAGs

use super::{Layout, LayoutError, LayoutResult, Position};
use crate::graph::PipelineGraph;
use crate::layout::algorithms::topological_layers;
use std::collections::HashMap;

pub struct HierarchicalLayout {
    pub direction: Direction,
    pub layer_spacing: f64,
    pub node_spacing: f64,
    pub node_width: f64,
    pub node_height: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    LeftToRight,
    TopToBottom,
}

impl Default for HierarchicalLayout {
    fn default() -> Self {
        Self {
            direction: Direction::LeftToRight,
            layer_spacing: 200.0,
            node_spacing: 80.0,
            node_width: 120.0,
            node_height: 60.0,
        }
    }
}

impl Layout for HierarchicalLayout {
    fn compute(&self, graph: &PipelineGraph) -> Result<LayoutResult, LayoutError> {
        if graph.node_count() == 0 {
            return Err(LayoutError::EmptyGraph);
        }

        // Get topological layers
        let layers =
            topological_layers(graph).map_err(|cycle_nodes| LayoutError::CycleDetected(cycle_nodes))?;

        // Assign positions
        let mut positions = HashMap::new();
        let mut max_width: f64 = 0.0;
        let mut max_height: f64 = 0.0;

        for (layer_idx, layer) in layers.iter().enumerate() {
            let layer_pos = layer_idx as f64 * self.layer_spacing;

            for (node_idx, node_id) in layer.iter().enumerate() {
                let node_pos = node_idx as f64 * self.node_spacing;

                let pos = match self.direction {
                    Direction::LeftToRight => Position {
                        x: layer_pos,
                        y: node_pos,
                    },
                    Direction::TopToBottom => Position {
                        x: node_pos,
                        y: layer_pos,
                    },
                };

                max_width = max_width.max(pos.x + self.node_width);
                max_height = max_height.max(pos.y + self.node_height);

                positions.insert(node_id.clone(), pos);
            }
        }

        Ok(LayoutResult {
            positions,
            width: max_width + 50.0,  // Add padding
            height: max_height + 50.0,
        })
    }
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
    fn test_simple_layout() {
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

        let layout = HierarchicalLayout::default();
        let result = layout.compute(&graph).unwrap();

        assert_eq!(result.positions.len(), 3);
        assert!(result.positions.contains_key("a"));
        assert!(result.positions.contains_key("b"));
        assert!(result.positions.contains_key("c"));

        // Verify left-to-right ordering
        let pos_a = &result.positions["a"];
        let pos_b = &result.positions["b"];
        let pos_c = &result.positions["c"];

        assert!(pos_a.x < pos_b.x);
        assert!(pos_b.x < pos_c.x);
    }

    #[test]
    fn test_empty_graph() {
        let graph = PipelineGraph::new();
        let layout = HierarchicalLayout::default();
        let result = layout.compute(&graph);

        assert!(matches!(result, Err(LayoutError::EmptyGraph)));
    }
}
