// Layout algorithms for pipeline graphs

pub mod algorithms;
pub mod hierarchical;

use crate::graph::{NodeId, PipelineGraph};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub positions: HashMap<NodeId, Position>,
    pub width: f64,
    pub height: f64,
}

pub trait Layout {
    fn compute(&self, graph: &PipelineGraph) -> Result<LayoutResult, LayoutError>;
}

#[derive(Debug)]
pub enum LayoutError {
    CycleDetected(Vec<NodeId>),
    EmptyGraph,
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LayoutError::CycleDetected(nodes) => {
                write!(f, "Cycle detected: {}", nodes.join(" -> "))
            }
            LayoutError::EmptyGraph => write!(f, "Cannot layout empty graph"),
        }
    }
}

impl std::error::Error for LayoutError {}
