# Lightweaver

Universal Data Pipeline Visualizer supporting 100+ tools via [OpenLineage](https://openlineage.io).

## Features

- **Multi-tool support**: dbt, Airflow, Spark, Fivetran, Dagster, Prefect, and any tool that emits OpenLineage events
- **Cross-tool lineage stitching**: Automatically connects jobs from different tools via matching dataset URNs
- **Multiple output formats**: SVG (vector), PNG (raster), PDF (vector), and interactive HTML
- **Interactive HTML output**: Pan, zoom, search, and click-to-inspect with svg-pan-zoom
- **Production-ready export**: High-quality PNG and PDF for documentation and presentations
- **Powerful filtering**: Filter by tool type, tags, or namespaces to focus on specific parts of your pipeline
- **Beautiful visualizations**: Hand-crafted SVG rendering with light/dark themes
- **Both library and CLI**: Use as a Rust library or command-line tool

## Installation

### As a CLI tool

```bash
cargo install lightweaver
```

### As a library

```toml
[dependencies]
lightweaver = "0.1"
```

## CLI Usage

### Basic usage

```bash
# Static SVG output
lightweaver generate --dbt target/manifest.json --output pipeline.svg

# Interactive HTML (auto-detected from .html extension)
lightweaver generate --openlineage events.json --output lineage.html

# PNG export for presentations (auto-detected)
lightweaver generate --openlineage events.json --output diagram.png

# PDF export for documentation (auto-detected)
lightweaver generate --dbt manifest.json --output lineage.pdf

# Multi-source with dark theme
lightweaver generate \
  --dbt manifest.json \
  --openlineage airflow_events.json \
  --openlineage spark_events.json \
  --theme dark \
  --output merged.svg

# Explicit format specification
lightweaver generate --openlineage events.json --format pdf --output viz.pdf
```

### Options

```
lightweaver generate [OPTIONS]

OPTIONS:
    --dbt <PATH>            Path to dbt manifest.json
    --openlineage <PATH>    Path to OpenLineage events (can be specified multiple times)
    --output <PATH>         Output file path (default: pipeline.svg)
    --format <FORMAT>       Output format: svg, html, png, pdf (auto-detected from extension)
    --theme <THEME>         Color theme: light, dark (default: light)
    --quiet, -q             Suppress progress messages (warnings still shown)

FILTERING:
    --filter-tool <TOOL>        Only show nodes from specific tool (dbt, airflow, spark, etc.)
    --filter-tag <TAG>          Only show nodes with specific tag
    --filter-namespace <NS>     Only show nodes from namespace prefix
```

### Interactive HTML Output

Generate interactive visualizations with pan, zoom, and click-to-inspect:

```bash
lightweaver generate --openlineage events.json --output pipeline.html
```

**Features:**
- 🖱️ **Pan & Zoom** - Mouse wheel to zoom, click-drag to pan
- 🔍 **Search** - Find and highlight nodes by name, ID, or tag
- 👆 **Click to Inspect** - View full node metadata in sidebar
- ⌨️ **Keyboard Shortcuts** - `/` to search, `Esc` to close, arrows to navigate
- 🌓 **Dark Mode Toggle** - Switch themes on the fly
- 📱 **Responsive** - Works on desktop and mobile

## Library Usage

### Simple example

```rust
use lightweaver::PipelineVisualizer;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a visualizer
    let mut viz = PipelineVisualizer::new();

    // Add data sources
    viz.add_dbt_manifest(Path::new("target/manifest.json"))?;
    viz.add_openlineage_events(Path::new("events.json"))?;

    // Render to SVG
    let svg = viz.render_svg()?;
    std::fs::write("pipeline.svg", svg)?;

    Ok(())
}
```

### Multi-source merging

The killer feature: automatic cross-tool lineage stitching!

```rust
use lightweaver::PipelineVisualizer;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut viz = PipelineVisualizer::new()
        .with_theme("dark");

    // Add multiple sources - they'll be automatically merged
    viz.add_dbt_manifest(Path::new("dbt/manifest.json"))?;
    viz.add_openlineage_events(Path::new("airflow/events.json"))?;
    viz.add_openlineage_events(Path::new("spark/events.json"))?;

    // Get statistics
    let stats = viz.stats()?;
    println!("Merged {} sources", stats.source_count);
    println!("{}", stats);

    // Render
    let svg = viz.render_svg()?;
    std::fs::write("cross_tool_lineage.svg", svg)?;

    Ok(())
}
```

### PNG and PDF Export

Generate publication-ready raster and vector outputs:

```rust
use lightweaver::PipelineVisualizer;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut viz = PipelineVisualizer::new();
    viz.add_openlineage_events(Path::new("events.json"))?;

    // Export as PNG (raster, good for presentations)
    let png_bytes = viz.render_png()?;
    std::fs::write("diagram.png", png_bytes)?;

    // Export as PDF (vector, good for documentation)
    let pdf_bytes = viz.render_pdf()?;
    std::fs::write("lineage.pdf", pdf_bytes)?;

    // Interactive HTML (best for exploration)
    let html = viz.render_html()?;
    std::fs::write("interactive.html", html)?;

    Ok(())
}
```

### Programmatic access to the graph

```rust
use lightweaver::PipelineVisualizer;

let mut viz = PipelineVisualizer::new();
viz.add_openlineage_events("events.json")?;

// Get the merged graph
let graph = viz.graph()?;

// Analyze the graph
for job in graph.jobs() {
    println!("Job: {}", job.metadata.name);
    if let Some(tool) = job.tool() {
        println!("  Tool: {:?}", tool);
    }
}

for dataset in graph.datasets() {
    if let Some(urn) = dataset.urn() {
        println!("Dataset: {}", urn);
    }
}
```

### Filtering graphs

Filter graphs to show only specific tools, tags, or namespaces:

```rust
use lightweaver::{PipelineVisualizer, GraphFilter, ToolType};

let mut viz = PipelineVisualizer::new();
viz.add_openlineage_events("events.json")?;

// Filter to show only dbt nodes
let filter = GraphFilter::new().with_tool(ToolType::Dbt);
viz.set_filter(filter);

let svg = viz.render_svg()?;
```

### Interactive HTML rendering

Generate interactive HTML with pan, zoom, and node inspection:

```rust
use lightweaver::PipelineVisualizer;

let mut viz = PipelineVisualizer::new();
viz.add_openlineage_events("events.json")?;

// Render to interactive HTML
let html = viz.render_html()?;
std::fs::write("pipeline.html", html)?;

// Works with dark theme and filters too!
let mut dark_viz = PipelineVisualizer::new().with_theme("dark");
dark_viz.add_openlineage_events("events.json")?;
let dark_html = dark_viz.render_html()?;
```

**CLI filtering:**

```bash
# Show only dbt nodes
lightweaver generate --openlineage events.json --filter-tool dbt

# Show only production nodes with core tag
lightweaver generate \
  --openlineage events.json \
  --filter-tag production \
  --filter-tag core

# Show only postgres prod database
lightweaver generate \
  --openlineage events.json \
  --filter-namespace postgres://prod

# Combine multiple filters
lightweaver generate \
  --openlineage events.json \
  --filter-tool dbt \
  --filter-tool airflow \
  --filter-tag production
```

**Filter logic:**
- Multiple `--filter-tool` flags = OR (show dbt OR airflow)
- Multiple `--filter-tag` flags = AND (must have ALL tags)
- Multiple `--filter-namespace` flags = OR (any matching prefix)

## How It Works

### OpenLineage Standard

Lightweaver uses the [OpenLineage](https://openlineage.io) specification, which is becoming the industry standard for data lineage. It's supported by:

- dbt
- Airflow
- Spark
- Flink
- Dagster
- Prefect
- Great Expectations
- And 100+ other tools

### Cross-Tool Stitching

When you merge multiple sources, Lightweaver:

1. Parses each source into a graph (jobs and datasets)
2. Extracts dataset URNs (e.g., `postgres://warehouse:analytics.customers`)
3. Deduplicates datasets with matching URNs
4. Creates edges between jobs that read/write the same datasets
5. Lays out the unified graph hierarchically
6. Renders a beautiful SVG

This means you can see **end-to-end lineage** across your entire data stack!

Example: A dbt model writes to a table, an Airflow task reads from it, and a Spark job processes it further. Lightweaver shows all three connected automatically.

## Architecture

Lightweaver is built with a "tear-out-and-replace" philosophy:

- **Hand-rolled CLI parsing**: No clap/structopt - we know exactly what we need
- **Zero graphics dependencies**: Pure SVG generation, no resvg/cairo/skia
- **Minimal deps**: Only serde for JSON parsing
- **Universal graph model**: Works with any lineage source

The core is ~2,000 lines of Rust. Fast, clean, maintainable.

## Examples

See the `examples/` directory:

```bash
# Simple example
cargo run --example simple

# Multi-source merging
cargo run --example multi_source
```

## Testing

```bash
# Run all tests
cargo test

# Build and test
cargo build --release
./target/release/lightweaver generate --openlineage test_data/cross_tool_pipeline.json
```

## License

MIT OR Apache-2.0

## Contributing

This project follows the [One-Shot Iteration Protocol](docs/rfcs/RFC-131-one-shot-iteration.md):

1. Build complete features atomically
2. Comprehensive AI review (2 cycles minimum)
3. Fix all non-overengineered issues
4. Ship with single commit

PRs welcome!

## Related Projects

- [OpenLineage](https://openlineage.io) - The lineage specification we build on
- [dbt](https://www.getdbt.com/) - Analytics engineering tool
- [Apache Airflow](https://airflow.apache.org/) - Workflow orchestration
- [Apache Spark](https://spark.apache.org/) - Distributed computing

## Roadmap

- [x] dbt manifest parsing
- [x] OpenLineage event parsing
- [x] Multi-source merging
- [x] Cross-tool stitching
- [x] SVG rendering (light/dark themes)
- [x] Library API
- [x] Graph filtering (by tool, tag, namespace)
- [x] HTML output with interactivity
- [ ] PNG/PDF export
- [ ] Configuration file support
- [ ] Plugin system for custom parsers

Built with ❤️ and [Claude Code](https://claude.com/claude-code)
