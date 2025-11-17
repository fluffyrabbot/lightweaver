// OpenLineage specification types
// Spec: https://github.com/OpenLineage/OpenLineage/blob/main/spec/OpenLineage.md
//
// NOTE: Many types here are not yet used but are part of the OpenLineage standard
// and will be needed for full facet support in future versions.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RunEvent is the top-level event describing an observed state of a job run
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunEvent {
    pub event_type: EventType,
    pub event_time: String,  // ISO 8601 timestamp
    pub run: Run,
    pub job: Job,
    #[serde(default)]
    pub inputs: Vec<Dataset>,
    #[serde(default)]
    pub outputs: Vec<Dataset>,
    pub producer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    Start,
    Complete,
    Fail,
    Abort,
    #[serde(other)]
    Other,
}

/// Job is a process definition that consumes and produces datasets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub namespace: String,  // e.g., "dbt://my_project" or "airflow://prod"
    pub name: String,       // e.g., "models.dim_customers" or "etl_dag.load_data"
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub facets: HashMap<String, serde_json::Value>,
}

/// Dataset is an abstract representation of data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub namespace: String,  // e.g., "postgres://prod-db:5432"
    pub name: String,       // e.g., "public.dim_customers"
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub facets: HashMap<String, serde_json::Value>,
}

/// Run is an instance of a running job
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub run_id: String,  // UUID
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub facets: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Common Facets (strongly-typed for convenience)
// ============================================================================

/// Job facets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlJobFacet {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceCodeLocationJobFacet {
    #[serde(rename = "type")]
    pub location_type: String,  // "git", "file", etc.
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipJobFacet {
    pub owners: Vec<Owner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,  // "user", "team", etc.
}

/// Dataset facets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDatasetFacet {
    pub fields: Vec<SchemaField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceDatasetFacet {
    pub name: String,  // e.g., "postgres", "s3", "bigquery"
    pub uri: String,   // e.g., "postgres://host:port/database"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputStatisticsOutputDatasetFacet {
    pub row_count: Option<u64>,
    pub size: Option<u64>,  // bytes
}

/// Run facets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NominalTimeRunFacet {
    pub nominal_start_time: String,  // ISO 8601
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nominal_end_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParentRunFacet {
    pub run: RunReference,
    pub job: JobReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunReference {
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobReference {
    pub namespace: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorMessageRunFacet {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub programming_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stacktrace: Option<String>,
}

// ============================================================================
// Tool-Specific Facets (for dbt, Airflow, Spark, etc.)
// ============================================================================

/// dbt-specific job facet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbtJobFacet {
    pub materialization: String,  // "table", "view", "incremental", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// Airflow-specific job facet
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirflowJobFacet {
    pub dag_id: String,
    pub task_id: String,
    pub operator: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool: Option<String>,
}

/// Spark-specific job facet
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SparkJobFacet {
    pub application_id: String,
    pub job_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_ids: Option<Vec<u32>>,
}

// ============================================================================
// Helper functions for creating facets
// ============================================================================

impl Job {
    pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
            facets: HashMap::new(),
        }
    }

    pub fn with_facet(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.facets.insert(key.into(), value);
        self
    }

    pub fn with_sql(mut self, query: impl Into<String>) -> Self {
        self.facets.insert(
            "sql".to_string(),
            serde_json::json!({ "query": query.into() }),
        );
        self
    }

    pub fn with_dbt(
        mut self,
        materialization: impl Into<String>,
        tags: Vec<String>,
    ) -> Self {
        self.facets.insert(
            "dbt".to_string(),
            serde_json::json!({
                "materialization": materialization.into(),
                "tags": tags,
                "meta": serde_json::Value::Null
            }),
        );
        self
    }

    pub fn with_airflow(
        mut self,
        dag_id: impl Into<String>,
        task_id: impl Into<String>,
        operator: impl Into<String>,
    ) -> Self {
        self.facets.insert(
            "airflow".to_string(),
            serde_json::json!({
                "dag_id": dag_id.into(),
                "task_id": task_id.into(),
                "operator": operator.into(),
                "pool": serde_json::Value::Null
            }),
        );
        self
    }
}

impl Dataset {
    pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
            facets: HashMap::new(),
        }
    }

    pub fn with_facet(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.facets.insert(key.into(), value);
        self
    }

    pub fn with_schema(mut self, fields: Vec<SchemaField>) -> Self {
        self.facets.insert(
            "schema".to_string(),
            serde_json::json!({ "fields": fields }),
        );
        self
    }

    pub fn with_data_source(mut self, name: impl Into<String>, uri: impl Into<String>) -> Self {
        self.facets.insert(
            "dataSource".to_string(),
            serde_json::json!({
                "name": name.into(),
                "uri": uri.into()
            }),
        );
        self
    }

    /// Construct a dataset URN from namespace + name
    pub fn urn(&self) -> String {
        format!("{}:{}", self.namespace, self.name)
    }
}

impl Run {
    pub fn new(run_id: impl Into<String>) -> Self {
        Self {
            run_id: run_id.into(),
            facets: HashMap::new(),
        }
    }

    pub fn with_facet(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.facets.insert(key.into(), value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_dbt_job() {
        let job = Job::new("dbt://jaffle_shop", "models.customers")
            .with_dbt("table", vec!["core".to_string()]);

        assert_eq!(job.namespace, "dbt://jaffle_shop");
        assert_eq!(job.name, "models.customers");
        assert!(job.facets.contains_key("dbt"));
    }

    #[test]
    fn test_create_dataset() {
        let dataset = Dataset::new("postgres://prod:5432", "public.customers")
            .with_data_source("postgres", "postgres://prod:5432/analytics");

        assert_eq!(dataset.namespace, "postgres://prod:5432");
        assert_eq!(dataset.name, "public.customers");
        assert!(dataset.facets.contains_key("dataSource"));
    }

    #[test]
    fn test_dataset_urn() {
        let dataset = Dataset::new("s3://bucket", "data/orders/");
        assert_eq!(dataset.urn(), "s3://bucket:data/orders/");
    }

    #[test]
    fn test_serialize_run_event() {
        let event = RunEvent {
            event_type: EventType::Start,
            event_time: "2025-01-17T12:00:00Z".to_string(),
            run: Run::new("550e8400-e29b-41d4-a716-446655440000"),
            job: Job::new("dbt://jaffle", "models.customers"),
            inputs: vec![Dataset::new("postgres://prod", "raw.customers")],
            outputs: vec![Dataset::new("postgres://prod", "analytics.customers")],
            producer: "lightweaver/0.1.0".to_string(),
            schema_url: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("START"));
        assert!(json.contains("dbt://jaffle"));
    }
}
