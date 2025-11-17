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
pub use graph::{Edge, EdgeKind, GraphFilter, Node, NodeId, NodeKind, PipelineGraph, ToolType};
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
    filter: Option<GraphFilter>,
}

impl PipelineVisualizer {
    /// Create a new visualizer with default settings
    pub fn new() -> Self {
        Self {
            graphs: Vec::new(),
            theme: Theme::default(),
            filter: None,
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

    /// Set a graph filter to apply during rendering
    ///
    /// Filters allow you to show only specific tools, tags, or namespaces.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lightweaver::{PipelineVisualizer, GraphFilter, ToolType};
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut viz = PipelineVisualizer::new();
    /// viz.add_openlineage_events(Path::new("events.json"))?;
    ///
    /// // Show only dbt nodes
    /// let filter = GraphFilter::new().with_tool(ToolType::Dbt);
    /// viz.set_filter(filter);
    ///
    /// let svg = viz.render_svg()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_filter(&mut self, filter: GraphFilter) -> &mut Self {
        self.filter = Some(filter);
        self
    }

    /// Clear any applied filter
    pub fn clear_filter(&mut self) -> &mut Self {
        self.filter = None;
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
    /// If a filter is set, applies it to the merged graph.
    pub fn graph(&self) -> Result<PipelineGraph, Box<dyn std::error::Error>> {
        if self.graphs.is_empty() {
            return Err(
                "No pipeline nodes found. No data sources were added.\n\n\
                 Possible reasons:\n\
                 • You need to call add_dbt_manifest() or add_openlineage_events() first\n\
                 • The input files were empty or contained no valid pipeline nodes\n\n\
                 Example usage:\n\
                 • viz.add_dbt_manifest(Path::new(\"target/manifest.json\"))?;\n\
                 • viz.add_openlineage_events(Path::new(\"events.json\"))?;".into()
            );
        }

        let mut graph = if self.graphs.len() == 1 {
            self.graphs[0].clone()
        } else {
            let mut merged = lineage::merger::merge_graphs(self.graphs.clone());
            lineage::merger::deduplicate_datasets(&mut merged);
            merged
        };

        // Apply filter if set
        if let Some(ref filter) = self.filter {
            graph = graph.filter(filter);
        }

        Ok(graph)
    }

    /// Render the visualization to SVG
    ///
    /// This is the main method for library usage. It:
    /// 1. Merges all added sources
    /// 2. Computes hierarchical layout
    /// 3. Renders to SVG with the configured theme
    pub fn render_svg(&self) -> Result<String, Box<dyn std::error::Error>> {
        let graph = self.graph()?;

        // Check for empty graph
        if graph.node_count() == 0 {
            // Provide filter-aware error message
            if self.filter.is_some() {
                return Err(
                    "No pipeline nodes found to visualize after applying filters.\n\n\
                     Your filters may be too restrictive. Try:\n\
                     • Removing or relaxing some filters\n\
                     • Checking filter values for typos\n\
                     • Using viz.clear_filter() to see the unfiltered graph\n\n\
                     Example: If filtering by tag, ensure your nodes have those tags set.".into()
                );
            } else {
                return Err(
                    "No pipeline nodes found to visualize.\n\n\
                     Possible reasons:\n\
                     • Your input files were parsed but contained no valid pipeline nodes\n\
                     • dbt manifest has no models/seeds/snapshots (tests are filtered out)\n\
                     • OpenLineage events file was empty or had no job/dataset information\n\n\
                     Troubleshooting:\n\
                     • Check that your input files are not empty\n\
                     • Verify the file format matches the expected schema\n\
                     • For dbt: ensure manifest.json contains models\n\
                     • For OpenLineage: ensure events have 'job' and 'inputs'/'outputs' fields".into()
                );
            }
        }

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

    /// Render the visualization to interactive HTML
    ///
    /// Creates a self-contained HTML file with:
    /// - Pan and zoom controls (mouse wheel, drag)
    /// - Click nodes to view metadata
    /// - Search and highlight
    /// - Keyboard shortcuts
    /// - Dark mode toggle
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lightweaver::PipelineVisualizer;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut viz = PipelineVisualizer::new();
    /// viz.add_openlineage_events(Path::new("events.json"))?;
    ///
    /// let html = viz.render_html()?;
    /// std::fs::write("pipeline.html", html)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn render_html(&self) -> Result<String, Box<dyn std::error::Error>> {
        let svg_content = self.render_svg()?;
        let graph = self.graph()?;

        // Generate node metadata JSON for JavaScript using proper serialization
        use serde::Serialize;

        #[derive(Serialize)]
        struct NodeData<'a> {
            id: &'a str,
            name: &'a str,
            #[serde(rename = "type")]
            node_type: &'static str,
            tool: &'a str,
            namespace: &'a str,
            tags: &'a [String],
            description: Option<&'a str>,
        }

        let node_data: Vec<NodeData> = graph.nodes.values().map(|node| {
            let (node_type, tool, namespace) = match &node.kind {
                NodeKind::Job { tool, namespace, .. } => ("job", tool.as_str(), namespace.as_str()),
                NodeKind::Dataset { namespace, .. } => ("dataset", "dataset", namespace.as_str()),
                NodeKind::DbtModel { .. } => ("job", "dbt", "dbt://"),
                NodeKind::DbtSource { .. } => ("dataset", "source", ""),
            };

            NodeData {
                id: &node.id,
                name: &node.metadata.name,
                node_type,
                tool,
                namespace,
                tags: &node.metadata.tags,
                description: node.metadata.description.as_deref(),
            }
        }).collect();

        let nodes_json = serde_json::to_string(&node_data)?;

        let theme_name = if self.theme.background == "#0d1117" {
            "dark"
        } else {
            "light"
        };

        Ok(HTML_TEMPLATE
            .replace("{{SVG_CONTENT}}", &svg_content)
            .replace("{{NODES_JSON}}", &nodes_json)
            .replace("{{INITIAL_THEME}}", theme_name))
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

/// HTML template for interactive visualization
const HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Lightweaver - Pipeline Visualization</title>
    <script src="https://cdn.jsdelivr.net/npm/svg-pan-zoom@3.6.1/dist/svg-pan-zoom.min.js"
            integrity="sha384-yc/c2Lk1s2V2ir1rxvjo8YyVD9PlOlYTqpNr3Wm1WIuAA30GlDYNx6U5104OiavY"
            crossorigin="anonymous"></script>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; overflow: hidden; }
        body.dark { background: #0d1117; color: #c9d1d9; }
        body.light { background: #ffffff; color: #24292f; }

        #container { display: flex; height: 100vh; }
        #svg-container { flex: 1; position: relative; overflow: hidden; }
        #sidebar { width: 350px; border-left: 1px solid; padding: 20px; overflow-y: auto; transition: transform 0.3s; }
        body.light #sidebar { border-color: #d0d7de; background: #f6f8fa; }
        body.dark #sidebar { border-color: #30363d; background: #161b22; }
        #sidebar.hidden { transform: translateX(100%); }

        #toolbar { position: absolute; top: 10px; left: 10px; z-index: 10; display: flex; gap: 8px; }
        button { padding: 8px 16px; border: 1px solid; border-radius: 6px; cursor: pointer; font-size: 14px; transition: all 0.2s; }
        body.light button { background: white; color: #24292f; border-color: #d0d7de; }
        body.dark button { background: #21262d; color: #c9d1d9; border-color: #30363d; }
        button:hover { opacity: 0.8; }

        #search { padding: 8px 12px; border: 1px solid; border-radius: 6px; font-size: 14px; width: 200px; }
        body.light #search { background: white; color: #24292f; border-color: #d0d7de; }
        body.dark #search { background: #0d1117; color: #c9d1d9; border-color: #30363d; }

        #sidebar h2 { font-size: 18px; margin-bottom: 16px; }
        #sidebar .field { margin-bottom: 12px; }
        #sidebar .label { font-size: 12px; font-weight: 600; opacity: 0.7; margin-bottom: 4px; }
        #sidebar .value { font-size: 14px; }
        #sidebar .tags { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 4px; }
        #sidebar .tag { padding: 2px 8px; border-radius: 12px; font-size: 12px; }
        body.light .tag { background: #ddf4ff; color: #0969da; }
        body.dark .tag { background: #1f6feb; color: #cdd9e5; }

        .highlight { stroke: #fbbf24 !important; stroke-width: 3px !important; filter: drop-shadow(0 0 8px #fbbf24); }

        #help { position: absolute; bottom: 10px; right: 10px; padding: 12px; border-radius: 6px; font-size: 12px; opacity: 0.7; }
        body.light #help { background: rgba(255,255,255,0.9); }
        body.dark #help { background: rgba(22,27,34,0.9); }
    </style>
</head>
<body class="{{INITIAL_THEME}}">
    <div id="container">
        <div id="svg-container">
            <div id="toolbar">
                <button id="reset-zoom">Reset Zoom</button>
                <input type="text" id="search" placeholder="Search nodes..." />
                <button id="theme-toggle">Toggle Dark Mode</button>
            </div>
            <div id="svg-wrapper">{{SVG_CONTENT}}</div>
            <div id="help">
                <div><kbd>Click</kbd> node to inspect</div>
                <div><kbd>Scroll</kbd> to zoom</div>
                <div><kbd>Drag</kbd> to pan</div>
                <div><kbd>Esc</kbd> to close sidebar</div>
            </div>
        </div>
        <div id="sidebar" class="hidden">
            <h2>Node Details</h2>
            <div id="node-details"></div>
        </div>
    </div>

    <script>
        const nodes = {{NODES_JSON}};
        const panZoom = svgPanZoom('#svg-wrapper svg', {
            zoomEnabled: true,
            controlIconsEnabled: false,
            fit: true,
            center: true,
            minZoom: 0.1,
            maxZoom: 10
        });

        document.getElementById('reset-zoom').addEventListener('click', () => {
            panZoom.reset();
        });

        document.getElementById('theme-toggle').addEventListener('click', () => {
            document.body.classList.toggle('dark');
            document.body.classList.toggle('light');
        });

        const sidebar = document.getElementById('sidebar');
        const nodeDetails = document.getElementById('node-details');
        const searchInput = document.getElementById('search');

        // Node click handlers
        document.querySelectorAll('g[id]').forEach(el => {
            el.style.cursor = 'pointer';
            el.addEventListener('click', (e) => {
                e.stopPropagation();
                const nodeId = el.id;
                const node = nodes.find(n => n.id === nodeId);
                if (node) showNodeDetails(node);
            });
        });

        function showNodeDetails(node) {
            sidebar.classList.remove('hidden');
            nodeDetails.innerHTML = `
                <div class="field">
                    <div class="label">NAME</div>
                    <div class="value">${escapeHtml(node.name)}</div>
                </div>
                <div class="field">
                    <div class="label">ID</div>
                    <div class="value" style="font-family: monospace; font-size: 12px;">${escapeHtml(node.id)}</div>
                </div>
                <div class="field">
                    <div class="label">TYPE</div>
                    <div class="value">${node.type === 'job' ? '⚙️ Job' : '📊 Dataset'}</div>
                </div>
                <div class="field">
                    <div class="label">TOOL</div>
                    <div class="value">${escapeHtml(node.tool)}</div>
                </div>
                <div class="field">
                    <div class="label">NAMESPACE</div>
                    <div class="value" style="font-family: monospace; font-size: 12px;">${escapeHtml(node.namespace)}</div>
                </div>
                ${node.description ? `
                <div class="field">
                    <div class="label">DESCRIPTION</div>
                    <div class="value">${escapeHtml(node.description)}</div>
                </div>
                ` : ''}
                ${node.tags && node.tags.length > 0 ? `
                <div class="field">
                    <div class="label">TAGS</div>
                    <div class="tags">
                        ${node.tags.map(tag => `<span class="tag">${escapeHtml(tag)}</span>`).join('')}
                    </div>
                </div>
                ` : ''}
            `;
        }

        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }

        // Search functionality
        let searchTimeout;
        searchInput.addEventListener('input', (e) => {
            clearTimeout(searchTimeout);
            searchTimeout = setTimeout(() => {
                const query = e.target.value.toLowerCase();
                document.querySelectorAll('.highlight').forEach(el => el.classList.remove('highlight'));

                if (query) {
                    nodes.filter(node =>
                        node.name.toLowerCase().includes(query) ||
                        node.id.toLowerCase().includes(query) ||
                        node.tags.some(tag => tag.toLowerCase().includes(query))
                    ).forEach(node => {
                        const el = document.getElementById(node.id);
                        if (el) el.classList.add('highlight');
                    });
                }
            }, 300);
        });

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                sidebar.classList.add('hidden');
                document.querySelectorAll('.highlight').forEach(el => el.classList.remove('highlight'));
                searchInput.value = '';
            }
            if (e.key === '/' && e.target.tagName !== 'INPUT') {
                e.preventDefault();
                searchInput.focus();
            }
        });

        // Click outside sidebar to close
        document.getElementById('svg-container').addEventListener('click', () => {
            sidebar.classList.add('hidden');
        });
    </script>
</body>
</html>"#;
