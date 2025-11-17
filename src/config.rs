//! Configuration file support for .lightweaver.toml

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration loaded from .lightweaver.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Default output file path
    pub output: Option<PathBuf>,

    /// Default output format (svg, html, png, pdf)
    pub format: Option<String>,

    /// Default theme (light or dark)
    pub theme: Option<String>,

    /// Suppress progress messages by default
    pub quiet: Option<bool>,

    /// Default filter settings
    pub filter: Option<FilterConfig>,
}

/// Filter configuration section
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FilterConfig {
    /// Default tools to filter by
    pub tools: Option<Vec<String>>,

    /// Default tags to filter by
    pub tags: Option<Vec<String>>,

    /// Default namespaces to filter by
    pub namespaces: Option<Vec<String>>,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path.as_ref())?;
        let config: Config = toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse {}: {}", path.as_ref().display(), e))?;

        // Validate the config
        config.validate()?;

        Ok(config)
    }

    /// Try to load .lightweaver.toml from current directory or parent directories
    pub fn load_from_current_dir() -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let current_dir = std::env::current_dir()?;

        // Search up the directory tree for .lightweaver.toml
        let mut dir = current_dir.as_path();
        loop {
            let config_path = dir.join(".lightweaver.toml");
            if config_path.exists() {
                return Ok(Some(Self::from_file(&config_path)?));
            }

            // Move up to parent directory
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }

        Ok(None)
    }

    /// Validate configuration values
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate theme
        if let Some(theme) = &self.theme {
            if !matches!(theme.as_str(), "light" | "dark") {
                return Err(format!(
                    "Invalid theme '{}' in config. Valid options: light, dark",
                    theme
                ).into());
            }
        }

        // Validate format
        if let Some(format) = &self.format {
            if !matches!(format.as_str(), "svg" | "html" | "png" | "pdf") {
                return Err(format!(
                    "Invalid format '{}' in config. Valid options: svg, html, png, pdf",
                    format
                ).into());
            }
        }

        // Validate filter arrays aren't too large
        if let Some(filter) = &self.filter {
            if let Some(tools) = &filter.tools {
                if tools.len() > 50 {
                    return Err("Too many tools in config filter (maximum 50)".into());
                }
            }
            if let Some(tags) = &filter.tags {
                if tags.len() > 50 {
                    return Err("Too many tags in config filter (maximum 50)".into());
                }
            }
            if let Some(namespaces) = &filter.namespaces {
                if namespaces.len() > 50 {
                    return Err("Too many namespaces in config filter (maximum 50)".into());
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_valid_config() {
        let config_toml = r#"
            output = "my_output.svg"
            format = "html"
            theme = "dark"
            quiet = true

            [filter]
            tools = ["dbt", "airflow"]
            tags = ["production"]
            namespaces = ["postgres://prod"]
        "#;

        let config: Config = toml::from_str(config_toml).unwrap();
        assert_eq!(config.output, Some(PathBuf::from("my_output.svg")));
        assert_eq!(config.format, Some("html".to_string()));
        assert_eq!(config.theme, Some("dark".to_string()));
        assert_eq!(config.quiet, Some(true));

        let filter = config.filter.unwrap();
        assert_eq!(filter.tools, Some(vec!["dbt".to_string(), "airflow".to_string()]));
        assert_eq!(filter.tags, Some(vec!["production".to_string()]));
        assert_eq!(filter.namespaces, Some(vec!["postgres://prod".to_string()]));
    }

    #[test]
    fn test_parse_minimal_config() {
        let config_toml = r#"
            theme = "dark"
        "#;

        let config: Config = toml::from_str(config_toml).unwrap();
        assert_eq!(config.theme, Some("dark".to_string()));
        assert_eq!(config.output, None);
        assert_eq!(config.format, None);
    }

    #[test]
    fn test_validate_invalid_theme() {
        let config = Config {
            theme: Some("invalid".to_string()),
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid theme"));
    }

    #[test]
    fn test_validate_invalid_format() {
        let config = Config {
            format: Some("docx".to_string()),
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid format"));
    }

    #[test]
    fn test_load_from_file() {
        let temp_path = "test_config.toml";
        let mut file = fs::File::create(temp_path).unwrap();
        file.write_all(b"theme = \"dark\"\nquiet = true").unwrap();
        drop(file);

        let config = Config::from_file(temp_path).unwrap();
        assert_eq!(config.theme, Some("dark".to_string()));
        assert_eq!(config.quiet, Some(true));

        fs::remove_file(temp_path).ok();
    }

    #[test]
    fn test_load_invalid_toml() {
        let temp_path = "test_invalid.toml";
        let mut file = fs::File::create(temp_path).unwrap();
        file.write_all(b"theme = invalid syntax").unwrap();
        drop(file);

        let result = Config::from_file(temp_path);
        assert!(result.is_err());

        fs::remove_file(temp_path).ok();
    }
}
