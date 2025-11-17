// Core data model for pipeline graphs
// OpenLineage-aligned universal representation

use serde::{Deserialize, Serialize};
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

/// Universal node types aligned with OpenLineage
#[derive(Debug, Clone)]
pub enum NodeKind {
    /// A job/task/transformation (processes data)
    Job {
        namespace: String,     // e.g., "dbt://my_project", "airflow://prod"
        name: String,          // e.g., "models.dim_customers", "etl_dag.load_task"
        tool: ToolType,        // Which tool this job belongs to
        facets: HashMap<String, serde_json::Value>,  // Tool-specific metadata
    },

    /// A dataset/table/file (data asset)
    Dataset {
        namespace: String,     // e.g., "postgres://prod-db:5432", "s3://bucket"
        name: String,          // e.g., "public.customers", "data/orders/"
        facets: HashMap<String, serde_json::Value>,  // Schema, stats, etc.
    },

    // Legacy types for backward compatibility with current dbt parser
    // TODO: Remove these once dbt parser is refactored to use Job/Dataset
    DbtModel {
        materialization: String,
        rows: Option<u64>,
    },
    DbtSource {
        database: String,
        schema: String,
    },
}

/// Tool taxonomy - all data pipeline tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolType {
    Dbt,
    Airflow,
    Spark,
    Fivetran,
    Dagster,
    Prefect,
    Flink,
    Kafka,
    GreatExpectations,
    Custom,
}

impl ToolType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolType::Dbt => "dbt",
            ToolType::Airflow => "airflow",
            ToolType::Spark => "spark",
            ToolType::Fivetran => "fivetran",
            ToolType::Dagster => "dagster",
            ToolType::Prefect => "prefect",
            ToolType::Flink => "flink",
            ToolType::Kafka => "kafka",
            ToolType::GreatExpectations => "great_expectations",
            ToolType::Custom => "custom",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            ToolType::Dbt => "#FF694B",           // dbt orange
            ToolType::Airflow => "#017CEE",       // airflow blue
            ToolType::Spark => "#E25A1C",         // spark orange
            ToolType::Fivetran => "#0073E6",      // fivetran blue
            ToolType::Dagster => "#2C3E50",       // dagster dark
            ToolType::Prefect => "#2D4B73",       // prefect blue
            ToolType::Flink => "#E6526F",         // flink pink
            ToolType::Kafka => "#231F20",         // kafka black
            ToolType::GreatExpectations => "#FF6F61",  // coral
            ToolType::Custom => "#95A5A6",        // gray
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeMetadata {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EdgeKind,
}

impl Edge {
    /// Create a new edge
    pub fn new(from: impl Into<NodeId>, to: impl Into<NodeId>, kind: EdgeKind) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            kind,
        }
    }

    /// Check if this is a data flow edge
    pub fn is_data_flow(&self) -> bool {
        matches!(self.kind, EdgeKind::ReadsFrom | EdgeKind::WritesTo | EdgeKind::DataDependency)
    }
}

/// Universal edge types for data lineage
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// Data dependency: upstream job writes to dataset, downstream job reads from it
    /// Used for dbt model → model, Spark job → job, etc.
    DataDependency,

    /// Job reads from Dataset
    ReadsFrom,

    /// Job writes to Dataset
    WritesTo,

    /// Control dependency: one job triggers/waits for another
    /// Used for Airflow task dependencies
    ControlDependency,

    /// Legacy: generic dependency (backward compat)
    DependsOn,
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

    /// Get all nodes of a specific tool type
    pub fn nodes_by_tool(&self, tool: ToolType) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|node| match &node.kind {
                NodeKind::Job { tool: t, .. } => *t == tool,
                _ => false,
            })
            .collect()
    }

    /// Get dataset URN (namespace:name)
    pub fn dataset_urn(node: &Node) -> Option<String> {
        match &node.kind {
            NodeKind::Dataset { namespace, name, .. } => Some(format!("{}:{}", namespace, name)),
            _ => None,
        }
    }

    /// Count of job nodes
    pub fn job_count(&self) -> usize {
        self.nodes.values().filter(|n| matches!(n.kind, NodeKind::Job { .. })).count()
    }

    /// Count of dataset nodes
    pub fn dataset_count(&self) -> usize {
        self.nodes.values().filter(|n| matches!(n.kind, NodeKind::Dataset { .. })).count()
    }

    /// Iterator over job nodes
    pub fn jobs(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values().filter(|n| matches!(n.kind, NodeKind::Job { .. }))
    }

    /// Iterator over dataset nodes
    pub fn datasets(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values().filter(|n| matches!(n.kind, NodeKind::Dataset { .. }))
    }
}

// Helper constructors for common node types
impl Node {
    /// Check if this node is a job
    pub fn is_job(&self) -> bool {
        matches!(self.kind, NodeKind::Job { .. })
    }

    /// Check if this node is a dataset
    pub fn is_dataset(&self) -> bool {
        matches!(self.kind, NodeKind::Dataset { .. })
    }

    /// Get the tool type if this is a job node
    pub fn tool(&self) -> Option<ToolType> {
        match &self.kind {
            NodeKind::Job { tool, .. } => Some(*tool),
            _ => None,
        }
    }

    /// Get the URN if this is a dataset node
    pub fn urn(&self) -> Option<String> {
        match &self.kind {
            NodeKind::Dataset { namespace, name, .. } => Some(format!("{}:{}", namespace, name)),
            _ => None,
        }
    }

    pub fn dbt_model(
        id: impl Into<String>,
        name: impl Into<String>,
        materialization: impl Into<String>,
        tags: Vec<String>,
    ) -> Self {
        let name_str = name.into();
        let mat_str = materialization.into();

        let mut facets = HashMap::new();
        facets.insert(
            "dbt".to_string(),
            serde_json::json!({
                "materialization": mat_str,
                "tags": tags.clone(),
            }),
        );

        Self {
            id: id.into(),
            kind: NodeKind::Job {
                namespace: "dbt://project".to_string(),
                name: name_str.clone(),
                tool: ToolType::Dbt,
                facets,
            },
            metadata: NodeMetadata {
                name: name_str,
                description: None,
                tags,
                owner: None,
            },
        }
    }

    pub fn dbt_source(
        id: impl Into<String>,
        name: impl Into<String>,
        database: impl Into<String>,
        schema: impl Into<String>,
    ) -> Self {
        let db = database.into();
        let sch = schema.into();
        let nm = name.into();

        let mut facets = HashMap::new();
        facets.insert(
            "dataSource".to_string(),
            serde_json::json!({
                "name": "postgres",
                "uri": format!("{}://{}", db, sch),
            }),
        );

        Self {
            id: id.into(),
            kind: NodeKind::Dataset {
                namespace: format!("postgres://{}", db),
                name: format!("{}.{}", sch, nm),
                facets,
            },
            metadata: NodeMetadata {
                name: nm.clone(),
                description: None,
                tags: Vec::new(),
                owner: None,
            },
        }
    }

    pub fn airflow_task(
        id: impl Into<String>,
        dag_id: impl Into<String>,
        task_id: impl Into<String>,
        operator: impl Into<String>,
    ) -> Self {
        let dag = dag_id.into();
        let task = task_id.into();
        let op = operator.into();

        let mut facets = HashMap::new();
        facets.insert(
            "airflow".to_string(),
            serde_json::json!({
                "dagId": dag,
                "taskId": task,
                "operator": op,
            }),
        );

        Self {
            id: id.into(),
            kind: NodeKind::Job {
                namespace: "airflow://prod".to_string(),
                name: format!("{}.{}", dag, task),
                tool: ToolType::Airflow,
                facets,
            },
            metadata: NodeMetadata {
                name: task.clone(),
                description: None,
                tags: Vec::new(),
                owner: None,
            },
        }
    }
}

// Display trait implementations for better debugging and logging
impl std::fmt::Display for PipelineGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PipelineGraph({} nodes, {} edges)", self.node_count(), self.edge_count())
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Node({}, {})", self.id, self.metadata.name)
    }
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} -> {} ({:?})", self.from, self.to, self.kind)
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
    fn test_add_dbt_model() {
        let mut graph = PipelineGraph::new();

        let node = Node::dbt_model(
            "model1",
            "dim_customers",
            "table",
            vec!["core".to_string()],
        );

        graph.add_node(node);

        assert_eq!(graph.node_count(), 1);
        assert!(graph.get_node(&"model1".to_string()).is_some());
    }

    #[test]
    fn test_add_edge() {
        let mut graph = PipelineGraph::new();

        graph.add_edge(Edge {
            from: "model1".to_string(),
            to: "model2".to_string(),
            kind: EdgeKind::DataDependency,
        });

        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_nodes_by_tool() {
        let mut graph = PipelineGraph::new();

        graph.add_node(Node::dbt_model("m1", "customers", "table", vec![]));
        graph.add_node(Node::dbt_model("m2", "orders", "view", vec![]));
        graph.add_node(Node::airflow_task("t1", "etl_dag", "load_data", "BashOperator"));

        let dbt_nodes = graph.nodes_by_tool(ToolType::Dbt);
        assert_eq!(dbt_nodes.len(), 2);

        let airflow_nodes = graph.nodes_by_tool(ToolType::Airflow);
        assert_eq!(airflow_nodes.len(), 1);
    }

    #[test]
    fn test_tool_type_colors() {
        assert_eq!(ToolType::Dbt.color(), "#FF694B");
        assert_eq!(ToolType::Airflow.color(), "#017CEE");
        assert_eq!(ToolType::Spark.color(), "#E25A1C");
    }

    #[test]
    fn test_dataset_urn() {
        let node = Node {
            id: "ds1".to_string(),
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
        };

        let urn = PipelineGraph::dataset_urn(&node);
        assert_eq!(urn, Some("postgres://prod:public.customers".to_string()));
    }
}
