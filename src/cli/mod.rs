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
}

impl Default for GenerateArgs {
    fn default() -> Self {
        Self {
            dbt_manifest: None,
            openlineage: Vec::new(),
            output: PathBuf::from("pipeline.svg"),
            theme: "light".to_string(),
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
                gen_args.output = PathBuf::from(&args[i + 1]);
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
    --output <PATH>         Output SVG file path (default: pipeline.svg)
    --theme <THEME>         Color theme: light, dark (default: light)

EXAMPLES:
    # Generate from dbt manifest
    lightweaver generate --dbt target/manifest.json --output viz.svg

    # Generate from OpenLineage events (supports 100+ tools!)
    lightweaver generate --openlineage events.json --output lineage.svg

    # Combine multiple sources (cross-tool lineage!)
    lightweaver generate --dbt manifest.json --openlineage events.json --output merged.svg

    # Use dark theme
    lightweaver generate --openlineage events.json --theme dark

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

        let result = parse_generate_args(&args).unwrap();
        assert_eq!(result.dbt_manifest.unwrap(), PathBuf::from("test_data/jaffle_manifest.json"));
        assert_eq!(result.output, PathBuf::from("out.svg"));
    }

    #[test]
    fn test_missing_dbt_arg() {
        let args = vec!["--output".to_string(), "out.svg".to_string()];

        let result = parse_generate_args(&args);
        assert!(result.is_err());
    }
}
