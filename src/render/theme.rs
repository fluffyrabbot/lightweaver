// Visual themes for rendering

#[derive(Debug, Clone)]
pub struct Theme {
    pub background: String,
    pub node_fill: NodeColors,
    pub node_stroke: String,
    pub edge_stroke: String,
    pub text_color: String,
    pub font_family: String,
    pub font_size: f64,
}

#[derive(Debug, Clone)]
pub struct NodeColors {
    pub dbt_model_table: String,
    pub dbt_model_view: String,
    pub dbt_model_incremental: String,
    pub dbt_source: String,
    pub airflow_task: String,
    pub table: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: "#ffffff".to_string(),
            node_fill: NodeColors::default(),
            node_stroke: "#1f2937".to_string(),
            edge_stroke: "#6b7280".to_string(),
            text_color: "#111827".to_string(),
            font_family: "Inter, system-ui, sans-serif".to_string(),
            font_size: 14.0,
        }
    }
}

impl Default for NodeColors {
    fn default() -> Self {
        Self {
            dbt_model_table: "#3b82f6".to_string(),      // Blue
            dbt_model_view: "#10b981".to_string(),       // Green
            dbt_model_incremental: "#8b5cf6".to_string(), // Purple
            dbt_source: "#f59e0b".to_string(),           // Amber
            airflow_task: "#ef4444".to_string(),         // Red
            table: "#6366f1".to_string(),                // Indigo
        }
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            background: "#111827".to_string(),
            node_fill: NodeColors {
                dbt_model_table: "#60a5fa".to_string(),
                dbt_model_view: "#34d399".to_string(),
                dbt_model_incremental: "#a78bfa".to_string(),
                dbt_source: "#fbbf24".to_string(),
                airflow_task: "#f87171".to_string(),
                table: "#818cf8".to_string(),
            },
            node_stroke: "#f9fafb".to_string(),
            edge_stroke: "#9ca3af".to_string(),
            text_color: "#f9fafb".to_string(),
            font_family: "Inter, system-ui, sans-serif".to_string(),
            font_size: 14.0,
        }
    }
}
