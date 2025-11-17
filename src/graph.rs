// Core data model for pipeline graphs
use std::collections::HashMap;

pub type NodeId = String;

#[derive(Debug, Clone)]
pub struct PipelineGraph {
    pub nodes: HashMap<NodeId, Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    DbtModel {
        materialization: String,
        rows: Option<u64>,
    },
    DbtSource {
        database: String,
        schema: String,
    },
    AirflowTask {
        operator: String,
        schedule: Option<String>,
    },
    Table {
        schema: String,
        name: String,
    },
}

#[derive(Debug, Clone)]
pub struct NodeMetadata {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone)]
pub enum EdgeKind {
    DependsOn,   // dbt model dependency
    ReadsFrom,   // query reads table
    WritesTo,    // task writes to table
}

impl PipelineGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    pub fn get_node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let graph = PipelineGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_node() {
        let mut graph = PipelineGraph::new();

        graph.add_node(Node {
            id: "model1".to_string(),
            kind: NodeKind::DbtModel {
                materialization: "table".to_string(),
                rows: Some(1000),
            },
            metadata: NodeMetadata {
                name: "dim_customers".to_string(),
                description: Some("Customer dimension".to_string()),
                tags: vec!["core".to_string()],
                owner: None,
            },
        });

        assert_eq!(graph.node_count(), 1);
        assert!(graph.get_node(&"model1".to_string()).is_some());
    }

    #[test]
    fn test_add_edge() {
        let mut graph = PipelineGraph::new();

        graph.add_edge(Edge {
            from: "model1".to_string(),
            to: "model2".to_string(),
            kind: EdgeKind::DependsOn,
        });

        assert_eq!(graph.edge_count(), 1);
    }
}
