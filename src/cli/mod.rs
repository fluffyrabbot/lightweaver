// Hand-rolled CLI argument parsing

use std::env;
use std::path::PathBuf;

pub struct Args {
    pub command: Command,
}

pub enum Command {
    Generate(GenerateArgs),
    Help,
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
                    return Err("--dbt requires a value".to_string());
                }
                gen_args.dbt_manifest = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "--openlineage" => {
                if i + 1 >= args.len() {
                    return Err("--openlineage requires a value".to_string());
                }
                gen_args.openlineage.push(PathBuf::from(&args[i + 1]));
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
                gen_args.theme = args[i + 1].clone();
                i += 2;
            }
            arg => return Err(format!("Unknown argument: {}", arg)),
        }
    }

    if gen_args.dbt_manifest.is_none() && gen_args.openlineage.is_empty() {
        return Err("At least one input source (--dbt or --openlineage) is required".to_string());
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
            "manifest.json".to_string(),
            "--output".to_string(),
            "out.svg".to_string(),
        ];

        let result = parse_generate_args(&args).unwrap();
        assert_eq!(result.dbt_manifest.unwrap(), PathBuf::from("manifest.json"));
        assert_eq!(result.output, PathBuf::from("out.svg"));
    }

    #[test]
    fn test_missing_dbt_arg() {
        let args = vec!["--output".to_string(), "out.svg".to_string()];

        let result = parse_generate_args(&args);
        assert!(result.is_err());
    }
}
