//! Lightweaver - Universal Data Pipeline Visualizer
//!
//! Lightweaver provides a library and CLI tool for visualizing data pipelines
//! from multiple sources (dbt, Airflow, Spark, etc.) using the OpenLineage standard.
//!
//! # Examples
//!
//! ```no_run
//! use lightweaver::{PipelineVisualizer, Theme};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a visualizer
//! let mut viz = PipelineVisualizer::new();
//!
//! // Add data sources
//! viz.add_dbt_manifest(Path::new("target/manifest.json"))?;
//! viz.add_openlineage_events(Path::new("events.json"))?;
//!
//! // Render to SVG
//! let svg = viz.render_svg()?;
//! std::fs::write("pipeline.svg", svg)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Multi-Source Lineage Merging
//!
//! Lightweaver's killer feature is cross-tool lineage stitching:
//!
//! ```no_run
//! use lightweaver::PipelineVisualizer;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut viz = PipelineVisualizer::new();
//!
//! // Merge lineage from multiple tools
//! viz.add_dbt_manifest(Path::new("dbt/manifest.json"))?;
//! viz.add_openlineage_events(Path::new("airflow/events.json"))?;
//! viz.add_openlineage_events(Path::new("spark/events.json"))?;
//!
//! // Automatically stitches cross-tool dependencies via dataset URNs
//! let svg = viz.render_svg()?;
//! # Ok(())
//! # }
//! ```

// Core modules
pub mod graph;
pub mod layout;
pub mod lineage;
pub mod openlineage;
pub mod parsers;
pub mod render;

// Re-exports for convenience
pub use graph::{Edge, EdgeKind, Node, NodeId, NodeKind, PipelineGraph, ToolType};
pub use layout::{hierarchical::HierarchicalLayout, Layout, LayoutError};
pub use parsers::ParseError;
pub use render::{SvgRenderer, Theme};

use std::path::Path;

/// High-level API for building and rendering pipeline visualizations
///
/// This is the main entry point for library usage. It handles:
/// - Parsing multiple data sources
/// - Merging graphs with cross-tool stitching
/// - Layout computation
/// - SVG rendering
///
/// # Examples
///
/// ```no_run
/// use lightweaver::PipelineVisualizer;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut viz = PipelineVisualizer::new()
///     .with_theme("dark");
///
/// viz.add_dbt_manifest(Path::new("manifest.json"))?
///     .add_openlineage_events(Path::new("events.json"))?;
///
/// let svg = viz.render_svg()?;
/// std::fs::write("output.svg", svg)?;
/// # Ok(())
/// # }
/// ```
pub struct PipelineVisualizer {
    graphs: Vec<PipelineGraph>,
    theme: Theme,
}

impl PipelineVisualizer {
    /// Create a new visualizer with default settings
    pub fn new() -> Self {
        Self {
            graphs: Vec::new(),
            theme: Theme::default(),
        }
    }

    /// Set the color theme (light or dark)
    pub fn with_theme(mut self, theme: impl AsRef<str>) -> Self {
        self.theme = match theme.as_ref() {
            "dark" => Theme::dark(),
            _ => Theme::default(),
        };
        self
    }

    /// Add a dbt manifest.json to the visualization
    ///
    /// Parses the manifest and extracts models, seeds, and snapshots.
    /// Tests and other non-model nodes are filtered out.
    pub fn add_dbt_manifest(&mut self, path: &Path) -> Result<&mut Self, ParseError> {
        let graph = parsers::dbt::parse_manifest(path)?;
        self.graphs.push(graph);
        Ok(self)
    }

    /// Add OpenLineage events (JSON or NDJSON format)
    ///
    /// Supports events from any OpenLineage-compatible tool:
    /// dbt, Airflow, Spark, Fivetran, Dagster, Prefect, etc.
    pub fn add_openlineage_events(&mut self, path: &Path) -> Result<&mut Self, ParseError> {
        let graph = parsers::openlineage::parse_events(path)?;
        self.graphs.push(graph);
        Ok(self)
    }

    /// Get the current merged graph
    ///
    /// If multiple sources have been added, this merges them with
    /// cross-tool stitching based on dataset URNs.
    pub fn graph(&self) -> Result<PipelineGraph, Box<dyn std::error::Error>> {
        if self.graphs.is_empty() {
            return Err("No data sources added. Use add_dbt_manifest() or add_openlineage_events()".into());
        }

        if self.graphs.len() == 1 {
            Ok(self.graphs[0].clone())
        } else {
            let mut merged = lineage::merger::merge_graphs(self.graphs.clone());
            lineage::merger::deduplicate_datasets(&mut merged);
            Ok(merged)
        }
    }

    /// Render the visualization to SVG
    ///
    /// This is the main method for library usage. It:
    /// 1. Merges all added sources
    /// 2. Computes hierarchical layout
    /// 3. Renders to SVG with the configured theme
    pub fn render_svg(&self) -> Result<String, Box<dyn std::error::Error>> {
        let graph = self.graph()?;

        // Compute layout
        let layout_engine = HierarchicalLayout::default();
        let layout = layout_engine.compute(&graph)?;

        // Render SVG
        let renderer = SvgRenderer {
            theme: self.theme.clone(),
            ..Default::default()
        };

        Ok(renderer.render(&graph, &layout))
    }

    /// Get statistics about the current graph
    pub fn stats(&self) -> Result<VisualizationStats, Box<dyn std::error::Error>> {
        let graph = self.graph()?;

        Ok(VisualizationStats {
            total_nodes: graph.node_count(),
            job_nodes: graph.job_count(),
            dataset_nodes: graph.dataset_count(),
            total_edges: graph.edge_count(),
            source_count: self.graphs.len(),
        })
    }
}

impl Default for PipelineVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about a visualization
#[derive(Debug, Clone)]
pub struct VisualizationStats {
    /// Total number of nodes
    pub total_nodes: usize,
    /// Number of job nodes
    pub job_nodes: usize,
    /// Number of dataset nodes
    pub dataset_nodes: usize,
    /// Total number of edges
    pub total_edges: usize,
    /// Number of data sources merged
    pub source_count: usize,
}

impl std::fmt::Display for VisualizationStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} nodes ({} jobs, {} datasets), {} edges from {} source(s)",
            self.total_nodes,
            self.job_nodes,
            self.dataset_nodes,
            self.total_edges,
            self.source_count
        )
    }
}
