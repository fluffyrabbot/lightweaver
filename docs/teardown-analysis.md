# Teardown Analysis - Applying Bottom-Up Philosophy

**Date:** November 17, 2025
**Philosophy:** Tear out suboptimal, minimize deps, build from bottom up

---

## Current Codebase Audit (~5,911 LOC)

### TEAR OUT IMMEDIATELY ❌ (~3,500 LOC - 60% of codebase)

**1. Entire Lox VM Implementation (~2,138 LOC)**
- `src/script/vm/mod.rs` (596 LOC) - Stack-based VM, call frames
- `src/script/vm/compiler.rs` (691 LOC) - Bytecode compiler
- `src/script/parser.rs` (851 LOC) - Pratt parser for Lox
- `src/script/scanner.rs` (385 LOC) - Lexer
- `src/script/vm/chunk.rs` (163 LOC) - Bytecode chunks
- `src/script/vm/debug.rs` (127 LOC) - Disassembler
- `src/script/vm/value.rs` (117 LOC) - Runtime values
- `src/script/vm/gc.rs` (74 LOC) - "GC" (just arena allocator)
- `src/script/vm/object/*` (~200 LOC) - Object system
- `src/script/ast/*` (~400 LOC) - AST nodes
- `src/script/tokens.rs` (96 LOC) - Token types
- `src/script/test.rs` (131 LOC) - Tests

**Why tear out:**
- Built for general-purpose scripting (Lox language)
- We need domain-specific data pipeline primitives, not classes/functions
- 2000+ LOC for feature we won't use in MVP
- If we need scripting later, build minimal DSL tailored to pipelines
- Following "Crafting Interpreters" tutorial ≠ optimal for our use case

**2. Pixel-Based Renderer (~129 LOC)**
- `src/render.rs` - Software rasterization to pixel buffer
- `src/color.rs` - RGBA color manipulation

**Why tear out:**
- We need SVG output (vector graphics), not PNG raster
- Anti-aliased line rendering is cool but wrong output format
- Will rebuild as SVG generator (simpler, actually useful)

**3. Unused Shape Primitives (~200+ LOC)**
- `src/shapes/*` - Rect, Line, Triangle, BoundingBox
- `src/shape_tree.rs` - Tree structure (never used)

**Why tear out:**
- Generic shapes, not data pipeline shapes
- We need: `Node`, `Edge`, `Pipeline`, `Task`, not `Triangle`
- Will rebuild with domain semantics

**4. Test Scripts & Dead Code**
- `test_scripts/test.lox` - Lox language examples
- `test_scripts/test.lw` - Unused
- All the `#![allow(dead_code)]` suppressions
- Commented-out test functions in main.rs

**Why tear out:**
- Not relevant to data pipelines
- Just clutter

### KEEP ✅ (~400 LOC - infrastructure)

**1. Project Structure**
- `Cargo.toml` - Minimal! Only dep is `png` which we'll remove
- `src/main.rs` - Entry point (will gut and rebuild)
- Directory organization (src/, docs/, out/)

**2. Some Primitives**
- `src/macros.rs` - If useful macros exist
- `src/utils.rs` - Utility functions (if any)

**3. Documentation**
- `readme.md` - Will update
- `docs/` - Will expand
- `claude.md` - Our north star

### UNCERTAIN 🤔 (~200 LOC)

**`src/graph/mod.rs` (181 LOC)**
- Generic tree structure with `push`, `peek_deep`, etc.
- Has tests!
- Question: Is this useful for our graph data structure?

**Decision:** TEAR OUT.
- It's a tree, we need a DAG (directed acyclic graph)
- Generic implementation, not pipeline-specific
- Will rebuild with proper graph semantics

---

## Bottom-Up Architecture: What We Actually Need

### Layer 1: Core Data Model (Build First)

**`src/graph.rs` - Pure data structure (~200 LOC)**
```rust
// Domain-specific primitives
pub struct PipelineGraph {
    nodes: HashMap<NodeId, Node>,
    edges: Vec<Edge>,
}

pub struct Node {
    id: NodeId,
    kind: NodeKind,
    metadata: NodeMetadata,
}

pub enum NodeKind {
    DbtModel { materialization: Materialization, rows: Option<u64> },
    AirflowTask { operator: String, schedule: String },
    Table { schema: String, name: String },
    Query { sql: String },
}

pub struct Edge {
    from: NodeId,
    to: NodeId,
    kind: EdgeKind,
}

pub enum EdgeKind {
    DependsOn,      // dbt model dependency
    ReadsFrom,      // query reads table
    WritesTo,       // task writes to table
    TriggeredBy,    // airflow trigger
}
```

**No dependencies. Pure Rust. ~200 LOC max.**

### Layer 2: Graph Algorithms (Build Second)

**`src/layout/mod.rs` - Layout algorithms (~400 LOC)**
```rust
pub trait Layout {
    fn compute(&self, graph: &PipelineGraph) -> LayoutResult;
}

// Sugiyama framework for hierarchical graphs
pub struct HierarchicalLayout {
    direction: Direction,
    layer_spacing: f64,
    node_spacing: f64,
}

impl Layout for HierarchicalLayout {
    fn compute(&self, graph: &PipelineGraph) -> LayoutResult {
        // 1. Assign layers (topological sort)
        let layers = topological_layers(graph);

        // 2. Minimize edge crossings
        let ordered = minimize_crossings(layers);

        // 3. Assign x,y coordinates
        assign_positions(ordered)
    }
}

// Helper algorithms
fn topological_layers(graph: &PipelineGraph) -> Vec<Vec<NodeId>>;
fn detect_cycles(graph: &PipelineGraph) -> Vec<Vec<NodeId>>;
fn find_critical_path(graph: &PipelineGraph, node: NodeId) -> Vec<NodeId>;
```

**Minimal dependencies:**
- Maybe `petgraph`? NO - roll our own. It's just topological sort.
- Pure algorithms, ~400 LOC

### Layer 3: Parsers (Build Third)

**`src/parsers/dbt.rs` - Parse dbt manifest (~200 LOC)**
```rust
pub fn parse_manifest(path: &Path) -> Result<PipelineGraph, ParseError> {
    let manifest: serde_json::Value = serde_json::from_str(&json)?;

    let mut graph = PipelineGraph::new();

    for (name, model) in manifest["nodes"].as_object() {
        graph.add_node(Node {
            kind: NodeKind::DbtModel {
                materialization: model["materialization"],
                rows: None,
            },
            metadata: NodeMetadata {
                name: name.clone(),
                description: model["description"],
                tags: model["tags"],
            }
        });
    }

    // Parse dependencies
    for (name, model) in manifest["nodes"].as_object() {
        for dep in model["depends_on"]["nodes"] {
            graph.add_edge(Edge {
                from: dep,
                to: name,
                kind: EdgeKind::DependsOn,
            });
        }
    }

    Ok(graph)
}
```

**Dependencies:**
- `serde` + `serde_json` - ONLY for parsing JSON
- This is acceptable - parsing JSON by hand is masochism
- ~200 LOC per parser (dbt, airflow, sql)

### Layer 4: Renderer (Build Fourth)

**`src/render/svg.rs` - SVG generation (~300 LOC)**
```rust
pub struct SvgRenderer {
    width: f64,
    height: f64,
    theme: Theme,
}

impl SvgRenderer {
    pub fn render(&self, graph: &PipelineGraph, layout: &LayoutResult) -> String {
        let mut svg = String::new();

        svg.push_str(&format!(r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
            self.width, self.height));

        // Render edges first (background)
        for edge in &graph.edges {
            let from_pos = layout.positions[&edge.from];
            let to_pos = layout.positions[&edge.to];

            svg.push_str(&self.render_edge(from_pos, to_pos, &edge.kind));
        }

        // Render nodes on top
        for (id, node) in &graph.nodes {
            let pos = layout.positions[id];
            svg.push_str(&self.render_node(node, pos));
        }

        svg.push_str("</svg>");
        svg
    }

    fn render_node(&self, node: &Node, pos: Position) -> String {
        match &node.kind {
            NodeKind::DbtModel { .. } => {
                // Rounded rectangle with label
                format!(r#"<g>
                    <rect x="{}" y="{}" width="120" height="60" rx="8" fill="{}" />
                    <text x="{}" y="{}">{}</text>
                </g>"#,
                    pos.x, pos.y, self.theme.model_color,
                    pos.x + 60, pos.y + 30, node.metadata.name
                )
            }
            NodeKind::AirflowTask { .. } => {
                // Different shape for tasks
                format!(r#"<polygon points="..." fill="{}" />"#, self.theme.task_color)
            }
            _ => String::new()
        }
    }
}
```

**No dependencies. Just string building. ~300 LOC.**

### Layer 5: CLI (Build Last)

**`src/main.rs` - Command line interface (~200 LOC)**
```rust
// NO clap. Just hand-rolled arg parsing.
fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("generate") => cmd_generate(&args[2..]),
        Some("serve") => cmd_serve(&args[2..]),
        Some("init") => cmd_init(),
        _ => print_help(),
    }
}

fn cmd_generate(args: &[String]) {
    // Parse --dbt, --output flags manually
    let mut dbt_path = None;
    let mut output = "pipeline.svg";

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dbt" => dbt_path = Some(&args[i+1]),
            "--output" => output = &args[i+1],
            _ => {}
        }
        i += 2;
    }

    let graph = parsers::dbt::parse_manifest(dbt_path.unwrap()).unwrap();
    let layout = HierarchicalLayout::default().compute(&graph).unwrap();
    let svg = SvgRenderer::default().render(&graph, &layout);

    std::fs::write(output, svg).unwrap();
    println!("Generated {}", output);
}
```

**No dependencies. ~200 LOC for basic CLI.**

---

## Dependency Philosophy

### ACCEPTABLE DEPENDENCIES
- `serde` + `serde_json` - Parsing JSON is not worth hand-rolling
- That's it.

### REJECT THESE
- ❌ `clap` - CLI parsing is 20 lines of code
- ❌ `petgraph` - Graph algorithms are simple, roll our own
- ❌ `layouting` or similar - We know our domain better
- ❌ `svg` crate - String building is trivial
- ❌ Any web framework - Not needed for MVP

### MAYBE LATER (V2+)
- `tokio` - If we add async API polling
- `axum` - If we build web UI
- `sqlparser` - If we parse SQL for query viz

---

## Size Estimates (Bottom-Up Build)

```
src/
├── graph.rs              ~200 LOC  (core data model)
├── layout/
│   ├── mod.rs            ~100 LOC  (layout traits)
│   ├── hierarchical.rs   ~300 LOC  (Sugiyama framework)
│   └── algorithms.rs     ~200 LOC  (topo sort, cycles)
├── parsers/
│   ├── mod.rs            ~50 LOC
│   ├── dbt.rs            ~200 LOC  (manifest.json parser)
│   └── airflow.rs        ~200 LOC  (DAG parser) [V2]
├── render/
│   ├── mod.rs            ~50 LOC
│   ├── svg.rs            ~300 LOC  (SVG generation)
│   └── theme.rs          ~100 LOC  (colors, styling)
├── cli/
│   ├── mod.rs            ~100 LOC  (arg parsing)
│   └── commands.rs       ~200 LOC  (generate, serve, init)
└── main.rs               ~50 LOC   (entry point)

Total: ~2,050 LOC (down from 5,911)
```

**65% size reduction. 100% relevance increase.**

---

## Tear-Out Strategy

### Phase 1: Scorched Earth (Day 1)
```bash
# Delete entire script/ directory
rm -rf src/script/

# Delete rendering
rm src/render.rs
rm src/color.rs

# Delete shapes
rm -rf src/shapes/
rm src/shape_tree.rs

# Delete tests
rm -rf test_scripts/

# Delete unused graph
rm src/graph/mod.rs

# Update Cargo.toml
# Remove: png = "0.17.16"
# Add: serde = { version = "1.0", features = ["derive"] }
#      serde_json = "1.0"
```

### Phase 2: Skeleton (Day 1-2)
```bash
# Create new structure
mkdir -p src/{layout,parsers,render,cli}

# Stub files
touch src/graph.rs
touch src/layout/{mod.rs,hierarchical.rs,algorithms.rs}
touch src/parsers/{mod.rs,dbt.rs}
touch src/render/{mod.rs,svg.rs,theme.rs}
touch src/cli/{mod.rs,commands.rs}

# Gut main.rs - keep only entry point
```

### Phase 3: Build Bottom-Up (Week 1-2)
1. Implement `graph.rs` - Get data model right
2. Implement `algorithms.rs` - Topological sort, cycle detection
3. Implement `hierarchical.rs` - Basic layer assignment
4. Test with hand-crafted graph
5. Implement `dbt.rs` parser - Parse real manifest.json
6. Implement `svg.rs` - Render to SVG
7. Implement CLI - Tie it together
8. Test on jaffle_shop (dbt's example project)

---

## Success Criteria

### End of Week 1
```bash
$ lightweaver generate --dbt ./manifest.json --output pipeline.svg
Generated pipeline.svg
```
Opens SVG: 8 nodes (jaffle_shop models), properly laid out, readable.

### End of Week 2
- Handles 50+ node dbt projects
- Detects and highlights cycles
- Configurable layout (vertical/horizontal)
- Color-coded by materialization type
- Proper label rendering (no overlaps)

---

## Philosophy in Action

**"Tear out suboptimal instantly"**
- Identified 60% of codebase as wrong direction
- Not incrementally improving Lox VM - deleting it
- Not fixing pixel renderer - replacing with SVG

**"Minimize deps"**
- From `png` → only `serde`/`serde_json`
- No clap, petgraph, or framework bloat
- 1 dependency (JSON parsing) vs potential 10+

**"Bottom-up architecture"**
- Layer 1: Data model (what is a node?)
- Layer 2: Algorithms (how to arrange?)
- Layer 3: Parsers (how to get data?)
- Layer 4: Render (how to display?)
- Layer 5: CLI (how to invoke?)
- Each layer builds on previous, no top-down "framework"

**Result:** Smaller, faster, more focused codebase that actually solves the problem.
