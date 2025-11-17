// Hand-rolled CLI argument parsing

use std::env;
use std::path::PathBuf;

pub struct Args {
    pub command: Command,
}

pub enum Command {
    Generate(GenerateArgs),
    Help,
    Version,
}

pub struct GenerateArgs {
    pub dbt_manifest: Option<PathBuf>,
    pub openlineage: Vec<PathBuf>,  // Support multiple OpenLineage files
    pub output: PathBuf,
    pub theme: String,
    pub quiet: bool,
    pub filter_tools: Vec<String>,      // --filter-tool dbt,airflow
    pub filter_tags: Vec<String>,       // --filter-tag production,core
    pub filter_namespaces: Vec<String>, // --filter-namespace postgres://prod
    pub format: Option<String>,         // --format html|svg (auto-detect from extension if not set)
}

impl Default for GenerateArgs {
    fn default() -> Self {
        Self {
            dbt_manifest: None,
            openlineage: Vec::new(),
            output: PathBuf::from("pipeline.svg"),
            theme: "light".to_string(),
            quiet: false,
            filter_tools: Vec::new(),
            filter_tags: Vec::new(),
            filter_namespaces: Vec::new(),
            format: None,
        }
    }
}

pub fn parse_args() -> Result<Args, String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Ok(Args {
            command: Command::Help,
        });
    }

    let command = match args[1].as_str() {
        "generate" => Command::Generate(parse_generate_args(&args[2..])?),
        "help" | "--help" | "-h" => Command::Help,
        "version" | "--version" | "-v" => Command::Version,
        cmd => return Err(format!("Unknown command: {}", cmd)),
    };

    Ok(Args { command })
}

fn parse_generate_args(args: &[String]) -> Result<GenerateArgs, String> {
    let mut gen_args = GenerateArgs::default();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dbt" => {
                if i + 1 >= args.len() {
                    return Err("Missing file path after --dbt flag.\nUsage: --dbt <path-to-manifest.json>".to_string());
                }
                if gen_args.dbt_manifest.is_some() {
                    return Err("--dbt can only be specified once. Use --openlineage for additional data sources.".to_string());
                }
                let path = PathBuf::from(&args[i + 1]);
                if !path.exists() {
                    return Err(format!("dbt manifest file not found: {}\nPlease check the path and try again.", path.display()));
                }
                if !path.is_file() {
                    return Err(format!("Path is not a file: {}\nPlease provide a path to a dbt manifest.json file.", path.display()));
                }
                gen_args.dbt_manifest = Some(path);
                i += 2;
            }
            "--openlineage" => {
                if i + 1 >= args.len() {
                    return Err("Missing file path after --openlineage flag.\nUsage: --openlineage <path-to-events.json>".to_string());
                }
                let path = PathBuf::from(&args[i + 1]);
                if !path.exists() {
                    return Err(format!("OpenLineage events file not found: {}\nPlease check the path and try again.", path.display()));
                }
                if !path.is_file() {
                    return Err(format!("Path is not a file: {}\nPlease provide a path to an OpenLineage events file.", path.display()));
                }
                gen_args.openlineage.push(path);
                i += 2;
            }
            "--output" | "-o" => {
                if i + 1 >= args.len() {
                    return Err("--output requires a value".to_string());
                }
                let output_path = PathBuf::from(&args[i + 1]);

                // Validate parent directory exists
                if let Some(parent) = output_path.parent() {
                    if !parent.as_os_str().is_empty() && !parent.exists() {
                        return Err(format!(
                            "Output directory does not exist: {}\nPlease create the directory first or choose a different location.",
                            parent.display()
                        ));
                    }
                }

                gen_args.output = output_path;
                i += 2;
            }
            "--theme" => {
                if i + 1 >= args.len() {
                    return Err("--theme requires a value".to_string());
                }
                let theme = &args[i + 1];
                if !matches!(theme.as_str(), "light" | "dark") {
                    return Err(format!("Invalid theme '{}'. Valid options: light, dark", theme));
                }
                gen_args.theme = theme.clone();
                i += 2;
            }
            "--quiet" | "-q" => {
                gen_args.quiet = true;
                i += 1;
            }
            "--filter-tool" => {
                if i + 1 >= args.len() {
                    return Err("--filter-tool requires a value (e.g., dbt, airflow, spark)".to_string());
                }
                if gen_args.filter_tools.len() >= 50 {
                    return Err("Too many --filter-tool flags (maximum 50)".to_string());
                }
                gen_args.filter_tools.push(args[i + 1].clone());
                i += 2;
            }
            "--filter-tag" => {
                if i + 1 >= args.len() {
                    return Err("--filter-tag requires a value (e.g., production, core)".to_string());
                }
                if gen_args.filter_tags.len() >= 50 {
                    return Err("Too many --filter-tag flags (maximum 50)".to_string());
                }
                gen_args.filter_tags.push(args[i + 1].clone());
                i += 2;
            }
            "--filter-namespace" => {
                if i + 1 >= args.len() {
                    return Err("--filter-namespace requires a value (e.g., postgres://prod)".to_string());
                }
                if gen_args.filter_namespaces.len() >= 50 {
                    return Err("Too many --filter-namespace flags (maximum 50)".to_string());
                }
                gen_args.filter_namespaces.push(args[i + 1].clone());
                i += 2;
            }
            "--format" => {
                if i + 1 >= args.len() {
                    return Err("--format requires a value (html or svg)".to_string());
                }
                let format = &args[i + 1];
                if !matches!(format.as_str(), "html" | "svg") {
                    return Err(format!("Invalid format '{}'. Valid options: html, svg", format));
                }
                gen_args.format = Some(format.clone());
                i += 2;
            }
            arg => return Err(format!("Unknown argument: {}", arg)),
        }
    }

    if gen_args.dbt_manifest.is_none() && gen_args.openlineage.is_empty() {
        return Err("No input files specified. Use --dbt <path> or --openlineage <path> to provide data.\nExample: lightweaver generate --dbt target/manifest.json --output pipeline.svg".to_string());
    }

    Ok(gen_args)
}

pub fn print_help() {
    println!(
        r#"Lightweaver - Universal Data Pipeline Visualizer

USAGE:
    lightweaver <COMMAND> [OPTIONS]

COMMANDS:
    generate    Generate a visualization from data pipeline files
    help        Show this help message

GENERATE OPTIONS:
    --dbt <PATH>            Path to dbt manifest.json
    --openlineage <PATH>    Path to OpenLineage JSON events (ndjson or array)
    --output <PATH>         Output file path (default: pipeline.svg)
    --format <FORMAT>       Output format: html, svg (auto-detected from file extension)
    --theme <THEME>         Color theme: light, dark (default: light)
    --quiet, -q             Suppress progress messages (warnings still shown)

FILTERING OPTIONS:
    --filter-tool <TOOL>        Only show nodes from specific tool (dbt, airflow, spark, etc.)
                                Can be specified multiple times
    --filter-tag <TAG>          Only show nodes with specific tag
                                Can be specified multiple times (AND logic)
    --filter-namespace <NS>     Only show nodes from namespace prefix
                                Can be specified multiple times (OR logic)

EXAMPLES:
    # Generate from dbt manifest
    lightweaver generate --dbt target/manifest.json --output viz.svg

    # Generate from OpenLineage events (supports 100+ tools!)
    lightweaver generate --openlineage events.json --output lineage.svg

    # Combine multiple sources (cross-tool lineage!)
    lightweaver generate --dbt manifest.json --openlineage events.json --output merged.svg

    # Use dark theme
    lightweaver generate --openlineage events.json --theme dark

    # Filter to show only dbt models
    lightweaver generate --openlineage events.json --filter-tool dbt

    # Filter to show only production nodes with core tag
    lightweaver generate --openlineage events.json --filter-tag production --filter-tag core

    # Filter to show only postgres prod database
    lightweaver generate --openlineage events.json --filter-namespace postgres://prod

    # Generate interactive HTML (auto-detected from .html extension)
    lightweaver generate --openlineage events.json --output pipeline.html

    # Or explicitly specify format
    lightweaver generate --openlineage events.json --format html --output viz.html

SUPPORTED TOOLS (via OpenLineage):
    dbt, Airflow, Spark, Fivetran, Dagster, Prefect, Flink, Kafka,
    Great Expectations, and any tool emitting OpenLineage events

For more info, visit: https://github.com/fluffyrabbot/lightweaver
"#
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_generate_args() {
        let args = vec![
            "--dbt".to_string(),
            "test_data/jaffle_manifest.json".to_string(),
            "--output".to_string(),
            "out.svg".to_string(),
        ];

        let result = parse_generate_args(&args);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.dbt_manifest, Some(PathBuf::from("test_data/jaffle_manifest.json")));
        assert_eq!(args.output, PathBuf::from("out.svg"));
    }

    #[test]
    fn test_missing_dbt_arg() {
        let args = vec!["--output".to_string(), "out.svg".to_string()];

        let result = parse_generate_args(&args);
        assert!(result.is_err());
    }
}
