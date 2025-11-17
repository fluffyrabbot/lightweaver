// SVG renderer for pipeline graphs

use crate::graph::{NodeKind, PipelineGraph};
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
            node_width: 120.0,
            node_height: 60.0,
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

        svg.push_str("</svg>");
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

        let mut svg = String::new();

        // Node rectangle
        svg.push_str(&format!(
            r#"    <rect x="{}" y="{}" width="{}" height="{}" rx="8" fill="{}" stroke="{}" stroke-width="2"/>
"#,
            pos.x,
            pos.y,
            self.node_width,
            self.node_height,
            fill_color,
            self.theme.node_stroke
        ));

        // Node label
        let text_x = pos.x + self.node_width / 2.0;
        let text_y = pos.y + self.node_height / 2.0 + self.theme.font_size / 3.0;

        svg.push_str(&format!(
            r#"    <text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="{}" fill="{}">{}</text>
"#,
            text_x,
            text_y,
            self.theme.font_family,
            self.theme.font_size,
            self.theme.text_color,
            escape_xml(&node.metadata.name)
        ));

        svg
    }

    fn get_node_color(&self, kind: &NodeKind) -> &str {
        match kind {
            NodeKind::DbtModel { materialization, .. } => match materialization.as_str() {
                "table" => &self.theme.node_fill.dbt_model_table,
                "view" => &self.theme.node_fill.dbt_model_view,
                "incremental" => &self.theme.node_fill.dbt_model_incremental,
                _ => &self.theme.node_fill.dbt_model_view,
            },
            NodeKind::DbtSource { .. } => &self.theme.node_fill.dbt_source,
            NodeKind::AirflowTask { .. } => &self.theme.node_fill.airflow_task,
            NodeKind::Table { .. } => &self.theme.node_fill.table,
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
