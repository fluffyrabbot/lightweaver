# Universal Data Pipeline Visualizer Architecture

**Date**: 2025-01-17
**Goal**: Cover the entire market space (dbt, Airflow, Spark, Fivetran, Dagster, Prefect, etc.)

## The Key Insight: OpenLineage as Foundation

After researching cross-tool lineage, the answer is clear: **OpenLineage is the universal standard**.

- **LF AI & Data project** with broad industry adoption
- **100+ integrations** out of the box (dbt, Airflow, Spark, Flink, Dagster, Great Expectations, etc.)
- **JsonSchema specification** with well-defined core model
- **Extensible via facets** for tool-specific metadata

### Why OpenLineage?

1. **Universal abstraction**: Every data tool = Jobs + Datasets + Runs
2. **Already integrated**: Most modern tools emit OpenLineage events
3. **Standardized naming**: Dataset URNs enable cross-tool stitching
4. **Future-proof**: New tools add OpenLineage support, we get them for free

## Three-Layer Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    Input Formats                         │
├──────────────────────────────────────────────────────────┤
│ • dbt manifest.json                                      │
│ • Airflow metadata DB / REST API                         │
│ • Spark event logs / history server API                  │
│ • Fivetran connector configs / API                       │
│ • Dagster asset metadata                                 │
│ • Prefect flow definitions                               │
│ • Native OpenLineage JSON events                         │
│ • Custom YAML/JSON schemas                               │
└──────────────────────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────┐
│              Parser Layer (Tool → OpenLineage)           │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐        │
│  │    dbt     │  │  Airflow   │  │   Spark    │        │
│  │  Parser    │  │  Parser    │  │  Parser    │  ...   │
│  └────────────┘  └────────────┘  └────────────┘        │
│         │                │               │              │
│         └────────────────┴───────────────┘              │
│                         │                                │
│                         ▼                                │
│            ┌────────────────────────┐                    │
│            │  OpenLineage Events    │                    │
│            │  - RunEvent            │                    │
│            │  - Job                 │                    │
│            │  - Dataset (input/out) │                    │
│            │  - Facets (metadata)   │                    │
│            └────────────────────────┘                    │
└──────────────────────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────┐
│        Lineage Graph Builder (OpenLineage → IR)          │
├──────────────────────────────────────────────────────────┤
│ • Aggregate events into complete lineage graph           │
│ • Merge multi-source graphs by dataset URNs              │
│ • Detect cycles, compute levels                          │
│ • Preserve tool-specific facets as metadata              │
│                                                          │
│             → PipelineGraph (our IR)                     │
└──────────────────────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────┐
│           Layout & Rendering                             │
├──────────────────────────────────────────────────────────┤
│ • Hierarchical layout (Sugiyama)                         │
│ • Force-directed (for complex graphs)                    │
│ • SVG generation (current)                               │
│ • Interactive HTML (future)                              │
└──────────────────────────────────────────────────────────┘
```

## Core Data Model Refactor

### Current model (tool-specific):
```rust
enum NodeKind {
    DbtModel { materialization: String, rows: Option<u64> },
    DbtSource { database: String, schema: String },
    AirflowTask { operator: String, schedule: Option<String> },
    Table { schema: String, name: String },
}
```

### New model (OpenLineage-aligned):
```rust
// Universal node types
enum NodeKind {
    Job {
        namespace: String,      // e.g., "dbt://my_project"
        name: String,           // e.g., "models.dim_customers"
        tool: ToolType,         // dbt, airflow, spark, etc.
        facets: JobFacets,      // Tool-specific metadata
    },
    Dataset {
        namespace: String,      // e.g., "postgres://prod"
        name: String,           // e.g., "public.dim_customers"
        facets: DatasetFacets,  // Schema, stats, etc.
    },
}

enum ToolType {
    Dbt,
    Airflow,
    Spark,
    Fivetran,
    Dagster,
    Prefect,
    Custom(String),
}

// OpenLineage facets (extensible metadata)
struct JobFacets {
    // dbt-specific
    dbt_materialization: Option<String>,
    dbt_tags: Vec<String>,

    // airflow-specific
    airflow_operator: Option<String>,
    airflow_dag_id: Option<String>,

    // spark-specific
    spark_application_id: Option<String>,
    spark_job_id: Option<String>,

    // Common across tools
    sql: Option<String>,
    source_code_location: Option<String>,
    documentation: Option<String>,
    owner: Option<String>,
}

struct DatasetFacets {
    schema: Option<SchemaFacet>,
    data_source: Option<DataSourceFacet>,
    statistics: Option<StatisticsFacet>,
}
```

### Edge types (data lineage):
```rust
enum EdgeKind {
    // Job reads from Dataset
    ReadsFrom {
        job: NodeId,
        dataset: NodeId,
        transformation_type: Option<String>,  // "SELECT", "UNION", etc.
    },

    // Job writes to Dataset
    WritesTo {
        job: NodeId,
        dataset: NodeId,
        output_type: Option<String>,  // "CREATE", "INSERT", "MERGE"
    },

    // Job depends on Job (orchestration)
    DependsOn {
        upstream: NodeId,
        downstream: NodeId,
        dependency_type: DependencyType,  // Data, Control, Trigger
    },
}

enum DependencyType {
    Data,      // dbt model depends on another model
    Control,   // Airflow task waits for another task
    Trigger,   // Spark job triggered by Airflow
}
```

## Parser Implementation Strategy

### Phase 1: OpenLineage native support
```rust
// src/parsers/openlineage.rs
pub fn parse_events(path: &Path) -> Result<PipelineGraph, ParseError> {
    // Read OpenLineage JSON events (newline-delimited or array)
    // Build graph from Job/Dataset/Run events
    // This gives us immediate compatibility with 100+ tools
}
```

### Phase 2: Tool-specific parsers (emit OpenLineage internally)
```rust
// src/parsers/dbt.rs - refactored
pub fn parse_manifest(path: &Path) -> Result<Vec<OpenLineageEvent>, ParseError> {
    // Parse manifest.json
    // Emit OpenLineage RunEvent for each model
    // Include dbt-specific facets (materialization, tags, etc.)
}

// src/parsers/airflow.rs - new
pub fn parse_metadata_db(conn_str: &str) -> Result<Vec<OpenLineageEvent>, ParseError> {
    // Query Airflow metadata DB (dag, dag_run, task_instance tables)
    // Or call Airflow REST API
    // Emit OpenLineage events for each DAG/Task
}

// src/parsers/spark.rs - new
pub fn parse_event_log(path: &Path) -> Result<Vec<OpenLineageEvent>, ParseError> {
    // Parse Spark event log JSON
    // Extract SparkListenerJobStart/End, StageSubmitted, etc.
    // Emit OpenLineage events with Spark facets
}
```

### Phase 3: Multi-source graph merging
```rust
// src/lineage/merger.rs
pub fn merge_graphs(graphs: Vec<PipelineGraph>) -> PipelineGraph {
    // Merge by dataset URNs
    // Example: dbt model "dim_customers" writes to "postgres://prod/public.dim_customers"
    //          Airflow task reads from same URN
    // → Connect dbt Job → Dataset ← Airflow Job
}
```

## Dataset URN Standardization

OpenLineage uses URNs to uniquely identify datasets across tools:

```
Format: {scheme}://{authority}/{path}

Examples:
- postgres://prod-db:5432/public.dim_customers
- s3://my-bucket/data/orders/year=2025/month=01/
- kafka://prod-cluster/topic.user_events
- bigquery://project-id/dataset.table
- file:///opt/data/analytics/customers.parquet
```

This enables automatic stitching:
- dbt model writes to `postgres://prod/public.dim_customers`
- Airflow task reads from same URN → edge created automatically

## Implementation Plan

### Phase 1: OpenLineage Foundation (Week 1)
- [ ] Add OpenLineage types (`src/openlineage.rs`)
- [ ] Refactor `PipelineGraph` to be OpenLineage-aligned
- [ ] Implement native OpenLineage JSON parser
- [ ] Update dbt parser to emit OpenLineage internally
- [ ] Test with dbt manifest → OpenLineage → PipelineGraph

### Phase 2: Multi-Tool Support (Week 2)
- [ ] Implement Airflow metadata DB/API parser
- [ ] Implement Spark event log parser
- [ ] Implement Fivetran config parser (if schemas available)
- [ ] Add CLI flags: `--dbt`, `--airflow`, `--spark`, `--openlineage`
- [ ] Test with real Airflow + dbt pipeline

### Phase 3: Cross-Tool Lineage (Week 3)
- [ ] Implement dataset URN resolution
- [ ] Build multi-source graph merger
- [ ] Add conflict resolution (same dataset, different metadata)
- [ ] Test: dbt → Airflow → Spark end-to-end lineage

### Phase 4: Visual Polish for Multi-Tool (Week 4)
- [ ] Color-code by tool (dbt = blue, Airflow = green, Spark = orange)
- [ ] Show tool logos/icons on nodes
- [ ] Legend with tool breakdown
- [ ] Interactive filters: "Show only dbt", "Hide upstream sources"

## Example: End-to-End Multi-Tool Pipeline

```
Input:
1. dbt manifest.json (15 models, 3 sources)
2. Airflow DAG metadata (5 tasks including "run_dbt", "load_to_warehouse")
3. Spark event log (2 jobs: "aggregate_metrics", "train_model")

Processing:
1. Parse dbt → OpenLineage events
   - Job: "dbt://jaffle_shop/models.customers"
   - Writes to: "postgres://prod/analytics.customers"

2. Parse Airflow → OpenLineage events
   - Job: "airflow://etl_dag/run_dbt"
   - Triggers: "dbt://jaffle_shop/models.customers"

   - Job: "airflow://etl_dag/load_to_warehouse"
   - Reads: "postgres://prod/analytics.customers"
   - Writes: "s3://data-lake/analytics/customers/"

3. Parse Spark → OpenLineage events
   - Job: "spark://analytics/aggregate_metrics"
   - Reads: "s3://data-lake/analytics/customers/"
   - Writes: "s3://data-lake/metrics/daily_rollup/"

4. Merge by dataset URNs:
   dbt:customers → postgres://prod/analytics.customers ← airflow:load_to_warehouse
                   → s3://data-lake/analytics/customers/ ← spark:aggregate_metrics

Output: Single unified lineage graph showing:
- 3 tools (color-coded: dbt=blue, airflow=green, spark=orange)
- 4 datasets (postgres, s3 × 2)
- 4 jobs (1 dbt, 2 airflow, 1 spark)
- 6 edges (2 dbt dependencies, 2 airflow reads/writes, 2 spark reads/writes)
```

## Why This Covers the Entire Market

1. **OpenLineage native tools** (immediate support):
   - dbt, Airflow, Spark, Flink, Dagster, Great Expectations
   - Any tool emitting OpenLineage → we visualize it

2. **Tool-specific parsers** (custom integrations):
   - Fivetran, Prefect, Luigi, Argo Workflows
   - Parse their configs → emit OpenLineage → visualize

3. **Future tools** (zero effort):
   - New tools add OpenLineage support → we get them for free
   - No code changes needed on our side

4. **Custom pipelines** (user-defined):
   - Users can write simple YAML/JSON in OpenLineage format
   - Visualize custom scripts, cron jobs, etc.

## Competitive Moat

Unlike existing tools that are:
- **Tool-specific**: dbt docs (dbt only), Airflow UI (Airflow only)
- **Heavyweight**: Marquez, DataHub, Atlan (require infrastructure)
- **Cloud-only**: Monte Carlo, Datafold (SaaS, expensive)

We are:
- **Universal**: All tools via OpenLineage standard
- **Lightweight**: Single binary, reads JSON files, outputs SVG
- **Local-first**: No servers, no databases, no SaaS
- **Open-source**: Community-driven, free forever

## Next Steps

1. **Tonight**: Implement OpenLineage types and refactor graph model
2. **Tomorrow**: Build Airflow + Spark parsers
3. **Weekend**: Multi-source merger + end-to-end demo
4. **Ship v0.1**: Universal data pipeline visualizer supporting dbt, Airflow, Spark

---

**Bottom line**: By building on OpenLineage, we cover 100+ tools immediately and future-proof against new tools. This is the platonic ideal architecture.
