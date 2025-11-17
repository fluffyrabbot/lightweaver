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
            layer_spacing: 250.0,  // More horizontal space between layers
            node_spacing: 100.0,   // More vertical space between nodes
            node_width: 160.0,     // Wider nodes for longer names
            node_height: 80.0,     // Taller nodes for badges
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

        // Find max nodes in any layer for centering
        let max_nodes_in_layer = layers.iter().map(|l| l.len()).max().unwrap_or(1);
        let total_height = max_nodes_in_layer as f64 * self.node_spacing;

        // Assign positions with vertical centering
        let mut positions = HashMap::new();
        let mut max_width: f64 = 0.0;
        let padding = 50.0;

        for (layer_idx, layer) in layers.iter().enumerate() {
            let layer_x = padding + (layer_idx as f64 * self.layer_spacing);

            // Calculate vertical offset to center this layer
            let layer_height = layer.len() as f64 * self.node_spacing;
            let vertical_offset = (total_height - layer_height) / 2.0 + padding;

            for (node_idx, node_id) in layer.iter().enumerate() {
                let node_y = vertical_offset + (node_idx as f64 * self.node_spacing);

                let pos = match self.direction {
                    Direction::LeftToRight => Position {
                        x: layer_x,
                        y: node_y,
                    },
                    Direction::TopToBottom => Position {
                        x: node_y,
                        y: layer_x,
                    },
                };

                max_width = max_width.max(pos.x + self.node_width);

                positions.insert(node_id.clone(), pos);
            }
        }

        Ok(LayoutResult {
            positions,
            width: max_width + padding,
            height: total_height + self.node_height + (2.0 * padding),
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
