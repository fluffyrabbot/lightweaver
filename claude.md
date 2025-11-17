# Lightweaver: Data Pipeline Visualizer

**Project Redefinition Document**
**Date:** November 17, 2025
**Status:** Fork & Pivot from general-purpose diagram tool to specialized data pipeline visualizer

---

## Current State Assessment

### What Exists (As of 282171e)

**Core Infrastructure:**
- ✅ **Bytecode VM**: Full Lox-based scripting language with bytecode compiler
  - Scanner, parser, AST (851 LOC in parser.rs)
  - Stack-based VM with call frames (596 LOC in vm/mod.rs)
  - Compiler with scope tracking (691 LOC in compiler.rs)
  - Object system with classes and instances (WIP)
- ✅ **Rendering Engine**: Software rasterization with anti-aliased lines
  - Distance-field based line rendering
  - PNG output capability
  - Basic shapes (Rect, Line, Triangle)
- ✅ **Graph Data Structures**: Tree-based graph representation (unused)
- ✅ **Type Safety**: Rust with proper newtype wrappers (ObjectId, BytecodeIndex, etc.)

**What's Missing:**
- ❌ No layout algorithms
- ❌ No connection between scripting language and diagram generation
- ❌ No domain-specific primitives
- ❌ No real-world data ingestion
- ❌ No user-facing CLI or UI

**Code Quality:**
- Compiles cleanly (warnings only)
- ~5,911 lines of Rust
- Architecturally sound but incomplete
- No tests, no documentation

---

## The Vision: Platonic Ideal of Data Pipeline Visualization

### The Problem We're Solving

**Data engineers are drowning in complexity:**
- Airflow DAGs with hundreds of tasks rendered as spaghetti
- dbt lineage graphs that are technically correct but visually useless at scale
- SQL query dependencies buried in code, impossible to understand holistically
- Data flow across Kafka → Spark → S3 → Redshift → dbt → BI tools: completely opaque
- Impact analysis: "If I change this table, what breaks?" - takes hours of manual tracing
- Onboarding: New engineers spend weeks understanding data flow

**Existing tools fall short:**
- **Airflow UI**: Functional but ugly, can't handle >50 tasks well
- **dbt docs**: Static lineage, no interactivity, chokes on large projects
- **Mermaid/D2**: General-purpose, no data engineering primitives
- **Commercial tools** (Monte Carlo, Datafold): Expensive, coupled to observability platforms
- **GraphViz**: Can't express data-specific semantics (SCD Type 2, fanout patterns, etc.)

### What Makes Lightweaver Different

**1. Domain-Specific Language**

Not just nodes and edges - first-class primitives for data engineering:

```javascript
// Define a dbt model with lineage
model("dim_customers") {
  .type = "dimension"
  .materialization = "table"
  .rows = 1_200_000
  .freshness = "4h"

  depends_on("stg_customers", "stg_orders")

  // Automatic SCD Type 2 pattern recognition
  .scd_type = 2
  .natural_key = ["customer_id"]
}

// Airflow task with context
task("extract_salesforce") {
  .operator = "HttpOperator"
  .schedule = "0 */4 * * *"  // Every 4 hours
  .avg_duration = "12m"
  .failure_rate = 0.02  // 2% failure rate - visualize as warning

  writes_to("s3://raw/salesforce/")
}

// Data flow with transformations
pipeline("customer_360") {
  source("salesforce_api")
    -> transform("deduplicate", type: "spark")
    -> transform("enrich_demographics", type: "python")
    -> sink("redshift.analytics.dim_customers")

  // Branch for real-time path
  source("kafka.user_events")
    -> transform("sessionize", type: "flink")
    -> sink("clickhouse.events")
}

// Query visualization
query("dashboard_sales_by_region") {
  reads_from([
    "dim_customers",
    "fct_orders",
    "dim_geography"
  ])

  .used_by = "looker_dashboard:sales_overview"
  .query_cost = "$0.43/run"  // BigQuery cost
}
```

**2. Intelligent Layout**

Not just force-directed graphs - purpose-built for data pipelines:

- **Temporal awareness**: Left-to-right flow matching data direction (source → transformation → sink)
- **Layered DAG layout**: Topological sorting with horizontal lanes for parallel processing
- **Swimlanes**: Automatic grouping by:
  - Infrastructure (Bronze/Silver/Gold layers)
  - Team ownership
  - Criticality (production vs exploratory)
  - Schedule (hourly, daily, batch)
- **Minimap for large graphs**: Overview + detail view
- **Collapse/expand**: Hide implementation details, focus on architecture
- **Highlight paths**: "Show me all upstream dependencies of this dashboard"

**3. Live Data Integration**

Not static diagrams - living, breathing visualizations:

```javascript
// Connect to live systems
connect_airflow("https://airflow.company.com", api_token)
connect_dbt_cloud(account_id: 12345)
connect_bigquery(project: "analytics-prod")

// Auto-generate from reality
auto_discover() {
  .include_dbt_models = true
  .include_airflow_dags = true
  .infer_from_query_logs = true  // Parse BigQuery logs
}

// Augment with runtime data
enrich_with_metrics() {
  .show_row_counts = true
  .show_last_run_time = true
  .show_data_freshness = true
  .color_by_health = true  // Red = failing, Yellow = SLA breach, Green = healthy
}
```

**4. Interactivity**

Not just pretty pictures - actionable intelligence:

- **Click to drill down**: Node → view query/code → see sample data
- **Impact analysis**: Click a table → highlight all downstream consumers (lit up in red)
- **Time travel**: Slider to see pipeline state at any point in history
- **Search**: "Find all models using PII" → highlights in yellow
- **Filter by**:
  - Owner/team
  - Tags (pii, deprecated, critical)
  - Health status
  - Last modified date
- **Export**: PNG, SVG, PDF, interactive HTML

**5. Programmability**

Use the scripting language for power users:

```javascript
// Find circular dependencies
let cycles = find_cycles()
for cycle in cycles {
  highlight(cycle, color: "red")
  annotate(cycle[0], "⚠️ Circular dependency detected")
}

// Detect data quality issues
for model in dbt_models {
  if model.test_coverage < 0.5 {
    model.color = "orange"
    model.label += " (low test coverage)"
  }
}

// Custom metrics
fun critical_path_length(node) {
  // Longest path to this node
  return max(node.dependencies.map(d => critical_path_length(d))) + 1
}

for model in all_models() {
  model.criticality = critical_path_length(model)
  model.size = model.criticality * 10  // Size by importance
}

// Generate documentation
export_markdown("pipeline_docs.md") {
  .include_descriptions = true
  .include_owners = true
  .group_by = "team"
}
```

---

## The Platonic Ideal

### User Experience

**First-time user (data engineer joining new company):**

```bash
$ lightweaver init
? Connect to dbt? (y/n) y
? dbt project path: /home/user/analytics
? Connect to Airflow? (y/n) y
? Airflow URL: https://airflow.company.com
? API token: ****

Discovering data pipelines...
  ✓ Found 347 dbt models
  ✓ Found 89 Airflow DAGs
  ✓ Found 1,247 BigQuery tables
  ✓ Inferred 3,891 dependencies

Generating visualization...
  ✓ Laid out 347 nodes in 8 layers
  ✓ Detected 12 swimlanes (teams)
  ✓ Identified 3 circular dependencies ⚠️

$ lightweaver serve
🚀 Lightweaver running at http://localhost:3000
📊 Visualizing 347 models, 89 DAGs, 1,247 tables

# Opens browser to interactive visualization
# Can immediately:
#   - See entire pipeline architecture
#   - Click on any node to see code/docs
#   - Filter by team/tag/health
#   - Export to PNG for wiki
#   - Share URL with teammates
```

**Power user (data platform lead):**

```javascript
// custom_views/critical_pipeline.lw
import dbt from "./connections/dbt"
import airflow from "./connections/airflow"

// Define what's critical
let critical_tables = [
  "fct_revenue",
  "dim_customers",
  "fct_orders"
]

// Build subgraph
let critical_pipeline = subgraph()

for table in critical_tables {
  critical_pipeline.add(table)
  critical_pipeline.add_upstream(table, depth: 10)  // All deps
}

// Enrich with SLAs
for node in critical_pipeline.nodes() {
  if node.type == "dbt_model" {
    let sla = lookup_sla(node.name)
    node.sla = sla

    if node.last_run_duration > sla.max_duration {
      node.status = "warning"
      alert("SLA breach: " + node.name)
    }
  }
}

// Layout
layout(critical_pipeline, {
  algorithm: "hierarchical",
  direction: "left_to_right",
  swimlanes: "team",
  color_by: "health"
})

// Export
render("critical_pipeline.svg", {
  width: 3840,
  height: 2160,
  theme: "dark",
  show_metrics: true
})

// Also create interactive HTML
serve(critical_pipeline, {
  port: 8080,
  enable_live_updates: true,  // Poll Airflow every 5min
  enable_alerts: true
})
```

### Technical Features

**Must Have (MVP):**
- [ ] Parse dbt `manifest.json` → generate graph
- [ ] Parse Airflow DAG files → generate graph
- [ ] Merge dbt + Airflow into unified view
- [ ] Hierarchical (layered DAG) layout algorithm
- [ ] SVG rendering (not just PNG)
- [ ] Interactive HTML output with pan/zoom
- [ ] Click node → show metadata panel
- [ ] CLI: `lightweaver generate --input manifest.json --output pipeline.svg`
- [ ] Hot reload: watch files, regenerate on change

**Should Have (V1):**
- [ ] Live integration: poll Airflow API for task status
- [ ] Color by health: green/yellow/red based on task success
- [ ] Filter UI: show/hide by tag, owner, status
- [ ] Search: find nodes by name/description
- [ ] Impact analysis: click node → highlight all downstream
- [ ] Export to PNG, PDF, interactive HTML
- [ ] Swimlanes: group by team/layer
- [ ] Performance: handle 1000+ node graphs smoothly

**Could Have (V2):**
- [ ] Time travel: see pipeline state at historical point
- [ ] Diff view: compare two versions of pipeline
- [ ] Query log analysis: infer dependencies from BigQuery logs
- [ ] Cost attribution: show compute cost per pipeline step
- [ ] Data lineage: trace field-level lineage (column X comes from Y.Z)
- [ ] Multiplayer: collaborative editing with CRDTs
- [ ] VS Code extension: visualize dbt project in IDE
- [ ] Slack integration: post pipeline health summaries

**Won't Have (Out of Scope):**
- ❌ Generic diagram tool features (we're specialized)
- ❌ Workflow orchestration (use Airflow/Prefect for that)
- ❌ Data quality testing (use dbt tests, Great Expectations)
- ❌ Full ETL execution (visualization only)

---

## Technical Strategy

### Leverage Existing Assets

**The VM is overkill for basic viz, but perfect for:**
- Custom metrics and transformations
- User-defined layout algorithms
- Scripted graph manipulation
- Plugin system (users extend with .lw scripts)

**The rendering engine needs:**
- Switch from pixel buffer to vector graphics (SVG)
- Add text rendering (nodes need labels)
- Add GPU acceleration for large graphs (thousands of nodes)
- Interactive mode (pan, zoom, click handlers)

**New components needed:**
- **Parsers**: dbt manifest.json, Airflow DAG files, SQL query parsers
- **Graph algorithms**: Topological sort, cycle detection, longest path
- **Layout engine**: Hierarchical layout, force-directed fallback
- **API clients**: Airflow REST API, dbt Cloud API, BigQuery API
- **Frontend**: Lightweight web UI (or terminal UI for v0)

### Architecture Evolution

**Phase 1: Proof of Concept (2-4 weeks)**
```
[dbt manifest.json] → Parser → Graph → Layout → SVG
```
- Parse manifest.json into internal graph representation
- Implement basic hierarchical layout
- Render to SVG with node labels
- Success metric: Visualize real dbt project with 50+ models

**Phase 2: CLI Tool (4-6 weeks)**
```
lightweaver generate \
  --dbt manifest.json \
  --airflow dags/ \
  --output pipeline.svg \
  --config layout.toml
```
- Merge multiple data sources
- Configurable layout options
- Multiple output formats (SVG, PNG, HTML)
- Success metric: Usable by data engineers without coding

**Phase 3: Interactive Mode (6-8 weeks)**
```
lightweaver serve --watch
```
- Web UI with pan/zoom
- Click node → metadata panel
- Filter/search
- Hot reload
- Success metric: Demo-able, shareable, delightful

**Phase 4: Live Integration (8-12 weeks)**
```
lightweaver connect \
  --airflow-url https://... \
  --dbt-cloud-account 12345 \
  --bigquery-project analytics
```
- Poll live systems
- Show runtime data (row counts, last run time)
- Alert on failures/SLA breaches
- Success metric: Used in production by data teams

---

## Development Philosophy

### Core Tenets

**1. Tear Out and Replace Suboptimal Code Instantly**

No incremental improvements to wrong directions. No sunk cost fallacy. If code doesn't serve the mission, delete it ruthlessly.

- **60% of current codebase is being removed** - The Lox VM (~2,138 LOC), pixel renderer (129 LOC), generic shapes (200+ LOC)
- **Not fixing, replacing** - Pixel buffer → SVG generator, generic graph → domain-specific DAG, general scripting → data pipeline DSL
- **No gradual migration** - Scorched earth approach, rebuild from scratch
- **Evidence over sentiment** - "This took time to build" is not a reason to keep it

**2. Minimize Dependencies, Roll Bespoke When Practical**

External dependencies are liabilities: version conflicts, supply chain risks, compilation time, cognitive overhead.

- **Target: 1-2 dependencies total** - Only `serde`/`serde_json` for JSON parsing
- **Reject common "conveniences"**:
  - ❌ `clap` for CLI - 20 lines of `env::args()` is enough
  - ❌ `petgraph` for graphs - Topological sort is ~50 LOC
  - ❌ `svg` crate - String concatenation works fine
  - ❌ Web frameworks - Not needed for MVP
- **Hand-roll algorithms** - Layout algorithms, graph traversal, rendering
- **Exception: JSON parsing** - Parsing JSON by hand is masochism, `serde` is acceptable
- **Re-evaluate constantly** - If a dep stops pulling weight, delete it

**3. Architecture from Bottom-Up, Not Top-Down**

Start with core data structures and primitives. Build layers on solid foundations. No frameworks, no scaffolding, no boilerplate generators.

**Build order:**
1. **Layer 1: Data model** - What is a `Node`? An `Edge`? A `Pipeline`? (~200 LOC)
2. **Layer 2: Algorithms** - Topological sort, cycle detection, layout (~400 LOC)
3. **Layer 3: Parsers** - Read dbt manifest.json (~200 LOC)
4. **Layer 4: Renderer** - Output SVG (~300 LOC)
5. **Layer 5: CLI** - Tie it together (~200 LOC)

Each layer stands alone. Each layer has zero knowledge of layers above. Tests at every layer.

**Anti-patterns we reject:**
- ❌ Starting with CLI/UX and backfilling logic
- ❌ Framework-first development (Rails, Spring, etc.)
- ❌ "We'll need X eventually" - build when needed, not before
- ❌ Abstractions before concrete use cases

**Result: ~2,050 LOC of focused code** (down from 5,911 LOC of scattered infrastructure)

See `docs/teardown-analysis.md` for complete tear-out plan.

---

## Guiding Principles

### 1. **Data Engineers First**
Every decision prioritized through the lens: "Does this help a data engineer understand their pipeline faster?"

### 2. **Beauty Through Clarity**
D2-level aesthetics, but optimized for data semantics (swimlanes, temporal flow, dependency highlighting).

### 3. **Code-First, UI-Second**
CLI generates static artifacts (SVG/HTML). UI is a convenience layer on top. Scriptable everything.

### 4. **Performance is a Feature**
1000+ node graphs should render in <1 second. 60 FPS interactions. This is why Rust.

### 5. **Integrate, Don't Replace**
We're not building Airflow or dbt. We visualize what they produce. Play nice with existing tools.

### 6. **Opinionated Defaults, Extensible Later**
Zero-config should produce good results. Advanced features (scripting, customization) can come in V2+.

---

## Success Metrics

### Adoption
- [ ] 100 GitHub stars in first month
- [ ] Featured in dbt Community Slack
- [ ] "Show HN" with >200 upvotes
- [ ] 10 data teams using in production within 6 months

### Technical
- [ ] Visualize 1000+ node graph in <1 second
- [ ] Zero crashes on real-world dbt projects (test on 50+ public repos)
- [ ] Support dbt 1.0+, Airflow 2.0+, out of the box

### Qualitative
- [ ] "This finally helped me understand our pipeline" - testimonial
- [ ] Used in data engineering job interviews (candidate diagrams their work)
- [ ] Mentioned in data engineering podcasts/newsletters

---

## What We're Building Toward

**6 months from now:**
```bash
$ cargo install lightweaver
$ cd ~/my-dbt-project
$ lightweaver init
$ lightweaver serve

# Opens browser: beautiful, interactive visualization
# Shows 347 dbt models, color-coded by health
# Click any node: see SQL, docs, sample data
# Filter by team, search for PII, export to PNG

# Share with team:
$ lightweaver export pipeline.html
$ scp pipeline.html wiki.company.com/data-platform/
```

**Impact:**
- New data engineers onboard 10x faster
- Pipeline refactoring decisions made with confidence
- Impact analysis goes from "hours of SQL grepping" to "click and see"
- Data platform documentation stays up-to-date automatically
- Data incidents debugged faster (visual trace through pipeline)

**The platonic ideal:**
> Lightweaver is to data pipelines what Chrome DevTools is to web development: an indispensable tool that makes the invisible visible, the complex comprehensible, and the opaque obvious.

---

## Next Steps

See `docs/teardown-analysis.md` for complete tear-out strategy and architecture plan.

**Phase 1: Scorched Earth (Day 1)**
1. [ ] Delete entire `src/script/` directory (~2,138 LOC)
2. [ ] Delete `src/render.rs`, `src/color.rs` (pixel renderer)
3. [ ] Delete `src/shapes/`, `src/shape_tree.rs` (generic shapes)
4. [ ] Delete `test_scripts/` (Lox examples)
5. [ ] Update `Cargo.toml` - remove `png`, add `serde`/`serde_json`
6. [ ] Create new directory structure: `src/{graph.rs,layout/,parsers/,render/,cli/}`

**Phase 2: Build Bottom-Up (Week 1-2)**
1. [ ] Implement `src/graph.rs` - Core data model (Node, Edge, PipelineGraph)
2. [ ] Implement `src/layout/algorithms.rs` - Topological sort, cycle detection
3. [ ] Implement `src/layout/hierarchical.rs` - Layer assignment, basic layout
4. [ ] Test layout with hand-crafted graphs (unit tests)
5. [ ] Implement `src/parsers/dbt.rs` - Parse manifest.json into PipelineGraph
6. [ ] Implement `src/render/svg.rs` - Generate SVG from layout
7. [ ] Implement `src/cli/` - Hand-rolled arg parsing, wire everything together
8. [ ] Integration test: `lightweaver generate --dbt manifest.json --output pipeline.svg`
9. [ ] Test on dbt-labs/jaffle_shop (real project with ~8 models)
10. [ ] Test on larger dbt project (50+ models)

**Phase 3: Polish (Week 3)**
1. [ ] Add themes (colors, fonts, styling)
2. [ ] Improve layout aesthetics (spacing, alignment)
3. [ ] Add node metadata rendering (tags, descriptions)
4. [ ] Handle edge cases (cycles, disconnected nodes)
5. [ ] Performance testing (1000+ node graphs)
6. [ ] Write comprehensive README with examples
7. [ ] Prepare for "Show HN" launch

**Success criteria:**
- End of Week 1: Generate SVG from jaffle_shop manifest
- End of Week 2: Handle 50+ node projects, beautiful output
- End of Week 3: Ready to share publicly

**Contributions welcome.** But only after Phase 1 complete - we're tearing down first.

---

*"Reality is what I decide it to be"* - but for data pipelines, reality is what Lightweaver shows you it is.
