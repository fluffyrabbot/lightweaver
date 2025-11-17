// SVG renderer for pipeline graphs

use crate::graph::{NodeKind, PipelineGraph, ToolType};
use crate::layout::{LayoutResult, Position};
use crate::render::Theme;

pub struct SvgRenderer {
    pub theme: Theme,
    pub node_width: f64,
    pub node_height: f64,
}

impl Default for SvgRenderer {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            node_width: 160.0,  // Match layout
            node_height: 80.0,  // Match layout
        }
    }
}

impl SvgRenderer {
    pub fn render(&self, graph: &PipelineGraph, layout: &LayoutResult) -> String {
        let mut svg = String::new();

        // SVG header
        svg.push_str(&format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
            layout.width, layout.height
        ));
        svg.push('\n');

        // Define arrowhead marker
        svg.push_str(&format!(
            r#"  <defs>
    <marker id="arrowhead" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto">
      <polygon points="0 0, 10 3, 0 6" fill="{}"/>
    </marker>
  </defs>
"#,
            self.theme.edge_stroke
        ));

        // Background
        svg.push_str(&format!(
            r#"  <rect width="100%" height="100%" fill="{}"/>"#,
            self.theme.background
        ));
        svg.push('\n');

        // Render edges first (so nodes appear on top)
        svg.push_str("  <g id=\"edges\">\n");
        for edge in &graph.edges {
            if let (Some(from_pos), Some(to_pos)) = (
                layout.positions.get(&edge.from),
                layout.positions.get(&edge.to),
            ) {
                svg.push_str(&self.render_edge(from_pos, to_pos));
            }
        }
        svg.push_str("  </g>\n");

        // Render nodes
        svg.push_str("  <g id=\"nodes\">\n");
        for (node_id, node) in &graph.nodes {
            if let Some(pos) = layout.positions.get(node_id) {
                svg.push_str(&self.render_node(node, pos));
            }
        }
        svg.push_str("  </g>\n");

        // Add legend with graph awareness
        svg.push_str(&self.render_legend_with_graph(graph, layout));

        svg.push_str("</svg>");
        svg
    }

    fn render_legend(&self, layout: &LayoutResult) -> String {
        // Intentionally empty - will be replaced with graph-aware legend
        String::new()
    }

    fn render_legend_with_graph(&self, graph: &PipelineGraph, layout: &LayoutResult) -> String {
        use std::collections::HashMap;

        let legend_x = layout.width - 220.0;
        let legend_y = 20.0;
        let mut svg = String::new();

        // Count nodes by tool
        let mut tool_counts: HashMap<ToolType, usize> = HashMap::new();
        let mut has_datasets = false;

        for node in graph.nodes.values() {
            match &node.kind {
                NodeKind::Job { tool, .. } => {
                    *tool_counts.entry(*tool).or_insert(0) += 1;
                }
                NodeKind::Dataset { .. } => {
                    has_datasets = true;
                }
                _ => {}
            }
        }

        // Don't render legend if only one tool and no datasets
        if tool_counts.len() == 0 && !has_datasets {
            return String::new();
        }

        svg.push_str(&format!(
            "  <g id=\"legend\">\n    <rect x=\"{}\" y=\"{}\" width=\"200\" height=\"{}\" rx=\"8\" fill=\"#ffffff\" stroke=\"#e5e7eb\" stroke-width=\"1\" opacity=\"0.95\"/>\n    <text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"12\" font-weight=\"bold\" fill=\"{}\">Pipeline Tools</text>\n",
            legend_x - 10.0,
            legend_y - 10.0,
            30.0 + ((tool_counts.len() + if has_datasets { 1 } else { 0 }) as f64 * 25.0),
            legend_x,
            legend_y + 5.0,
            self.theme.font_family,
            self.theme.text_color
        ));

        let mut idx = 0;

        // Sort tools by count (descending)
        let mut tools: Vec<_> = tool_counts.iter().collect();
        tools.sort_by(|a, b| b.1.cmp(a.1));

        for (tool, count) in tools {
            let y = legend_y + 20.0 + (idx as f64 * 25.0);
            let color = tool.color();
            let name = tool.as_str();

            svg.push_str(&format!(
                r#"    <rect x="{}" y="{}" width="16" height="16" rx="3" fill="{}"/>
    <text x="{}" y="{}" font-family="{}" font-size="11" fill="{}">{} ({})</text>
"#,
                legend_x,
                y,
                color,
                legend_x + 22.0,
                y + 12.0,
                self.theme.font_family,
                self.theme.text_color,
                name,
                count
            ));
            idx += 1;
        }

        // Add datasets if present
        if has_datasets {
            let y = legend_y + 20.0 + (idx as f64 * 25.0);
            svg.push_str(&format!(
                "    <rect x=\"{}\" y=\"{}\" width=\"16\" height=\"16\" rx=\"3\" fill=\"#e5e7eb\" stroke=\"#9ca3af\" stroke-width=\"2\" stroke-dasharray=\"3,2\"/>\n    <text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"11\" fill=\"{}\">datasets</text>\n",
                legend_x,
                y,
                legend_x + 22.0,
                y + 12.0,
                self.theme.font_family,
                self.theme.text_color
            ));
        }

        svg.push_str("  </g>\n");
        svg
    }

    fn render_edge(&self, from: &Position, to: &Position) -> String {
        // Calculate center points of nodes
        let from_x = from.x + self.node_width / 2.0;
        let from_y = from.y + self.node_height / 2.0;
        let to_x = to.x + self.node_width / 2.0;
        let to_y = to.y + self.node_height / 2.0;

        // Simple straight line for now (could add bezier curves later)
        format!(
            r#"    <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="2" marker-end="url(#arrowhead)"/>
"#,
            from_x, from_y, to_x, to_y, self.theme.edge_stroke
        )
    }

    fn render_node(&self, node: &crate::graph::Node, pos: &Position) -> String {
        let fill_color = self.get_node_color(&node.kind);
        let is_dataset = matches!(&node.kind, NodeKind::Dataset { .. });

        let mut svg = String::new();

        // Drop shadow
        svg.push_str(&format!(
            "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"8\" fill=\"#000000\" opacity=\"0.1\"/>\n",
            pos.x + 2.0,
            pos.y + 2.0,
            self.node_width,
            self.node_height
        ));

        // Node rectangle (different style for datasets)
        if is_dataset {
            // Dataset: dashed border, lighter fill
            svg.push_str(&format!(
                "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"8\" fill=\"#f3f4f6\" stroke=\"#9ca3af\" stroke-width=\"2\" stroke-dasharray=\"5,3\"/>\n",
                pos.x, pos.y, self.node_width, self.node_height
            ));
        } else {
            // Job: solid border, tool-specific color
            svg.push_str(&format!(
                "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"8\" fill=\"{}\" stroke=\"{}\" stroke-width=\"2\"/>\n",
                pos.x, pos.y, self.node_width, self.node_height, fill_color, self.theme.node_stroke
            ));
        }

        // Tool name (for Job nodes)
        if let NodeKind::Job { tool, .. } = &node.kind {
            let tool_name = tool.as_str();
            svg.push_str(&format!(
                r#"    <text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="8" font-weight="600" fill="{}" opacity="0.5">{}</text>
"#,
                pos.x + self.node_width / 2.0,
                pos.y + 12.0,
                self.theme.font_family,
                self.theme.text_color,
                tool_name.to_uppercase()
            ));
        }

        // Node type badge
        let badge_text = self.get_node_badge(&node.kind);
        let badge_y = if matches!(&node.kind, NodeKind::Job { .. }) {
            pos.y + 26.0  // Lower for jobs (to make room for tool name)
        } else {
            pos.y + 16.0  // Higher for datasets
        };
        svg.push_str(&format!(
            r#"    <text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="9" font-weight="bold" fill="{}" opacity="0.7">{}</text>
"#,
            pos.x + self.node_width / 2.0,
            badge_y,
            self.theme.font_family,
            self.theme.text_color,
            badge_text
        ));

        // Node label (name)
        let text_x = pos.x + self.node_width / 2.0;
        let text_y = pos.y + self.node_height / 2.0 + 4.0;

        let name = truncate_with_ellipsis(&node.metadata.name, 20);

        svg.push_str(&format!(
            r#"    <text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="{}" font-weight="600" fill="{}">{}</text>
"#,
            text_x,
            text_y,
            self.theme.font_family,
            self.theme.font_size,
            self.theme.text_color,
            escape_xml(&name)
        ));

        // Tags (if any)
        if !node.metadata.tags.is_empty() {
            let tags_text = node.metadata.tags.iter()
                .take(2)
                .map(|t| t.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let tags_display = truncate_with_ellipsis(&tags_text, 25);

            svg.push_str(&format!(
                r#"    <text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="9" fill="{}" opacity="0.6">{}</text>
"#,
                text_x,
                pos.y + self.node_height - 12.0,
                self.theme.font_family,
                self.theme.text_color,
                escape_xml(&tags_display)
            ));
        }

        svg
    }

    fn get_node_badge(&self, kind: &NodeKind) -> &str {
        match kind {
            NodeKind::Job { tool, facets, .. } => {
                match tool {
                    ToolType::Dbt => {
                        // Extract materialization from dbt facet
                        if let Some(dbt_facet) = facets.get("dbt") {
                            if let Some(mat) = dbt_facet.get("materialization") {
                                if let Some(mat_str) = mat.as_str() {
                                    return match mat_str {
                                        "table" => "TABLE",
                                        "view" => "VIEW",
                                        "incremental" => "INCREMENTAL",
                                        "ephemeral" => "EPHEMERAL",
                                        _ => "MODEL",
                                    };
                                }
                            }
                        }
                        "MODEL"
                    }
                    ToolType::Airflow => "TASK",
                    ToolType::Spark => "JOB",
                    ToolType::Fivetran => "SYNC",
                    ToolType::Dagster => "ASSET",
                    ToolType::Prefect => "FLOW",
                    _ => "JOB",
                }
            }
            NodeKind::Dataset { .. } => "DATASET",

            // Legacy support
            NodeKind::DbtModel { materialization, .. } => match materialization.as_str() {
                "table" => "TABLE",
                "view" => "VIEW",
                "incremental" => "INCREMENTAL",
                "ephemeral" => "EPHEMERAL",
                _ => "MODEL",
            },
            NodeKind::DbtSource { .. } => "SOURCE",
        }
    }

    fn get_node_color(&self, kind: &NodeKind) -> &str {
        match kind {
            NodeKind::Job { tool, .. } => {
                // Use official brand colors for all tools!
                tool.color()
            }
            NodeKind::Dataset { .. } => {
                // Datasets use neutral gray
                "#f3f4f6"
            }

            // Legacy support
            NodeKind::DbtModel { materialization, .. } => match materialization.as_str() {
                "table" => &self.theme.node_fill.dbt_model_table,
                "view" => &self.theme.node_fill.dbt_model_view,
                "incremental" => &self.theme.node_fill.dbt_model_incremental,
                _ => &self.theme.node_fill.dbt_model_view,
            },
            NodeKind::DbtSource { .. } => &self.theme.node_fill.dbt_source,
        }
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn truncate_with_ellipsis(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Node, NodeKind, NodeMetadata, PipelineGraph};
    use crate::layout::Position;
    use std::collections::HashMap;

    #[test]
    fn test_render_simple_graph() {
        let mut graph = PipelineGraph::new();
        graph.add_node(Node {
            id: "test".to_string(),
            kind: NodeKind::DbtModel {
                materialization: "table".to_string(),
                rows: None,
            },
            metadata: NodeMetadata {
                name: "test_model".to_string(),
                description: None,
                tags: Vec::new(),
                owner: None,
            },
        });

        let mut positions = HashMap::new();
        positions.insert("test".to_string(), Position { x: 0.0, y: 0.0 });

        let layout = LayoutResult {
            positions,
            width: 500.0,
            height: 300.0,
        };

        let renderer = SvgRenderer::default();
        let svg = renderer.render(&graph, &layout);

        assert!(svg.contains("<svg"));
        assert!(svg.contains("test_model"));
        assert!(svg.contains("</svg>"));
    }
}
