//! Plugin system for custom parsers
//!
//! This module provides traits and types for creating custom parser plugins
//! that can extend Lightweaver to support additional data sources.
//!
//! # Examples
//!
//! ```rust
//! use lightweaver::plugin::{Parser, ParserPlugin};
//! use lightweaver::graph::PipelineGraph;
//! use lightweaver::parsers::ParseError;
//! use std::path::Path;
//!
//! // Define a custom parser
//! struct MyCustomParser;
//!
//! impl Parser for MyCustomParser {
//!     fn name(&self) -> &str {
//!         "my_custom_format"
//!     }
//!
//!     fn parse(&self, path: &Path) -> Result<PipelineGraph, ParseError> {
//!         // Custom parsing logic here
//!         let mut graph = PipelineGraph::new();
//!         // ... populate graph from custom format ...
//!         Ok(graph)
//!     }
//!
//!     fn supported_extensions(&self) -> Vec<&str> {
//!         vec!["mycustom", "custom"]
//!     }
//! }
//! ```

use crate::graph::PipelineGraph;
use crate::parsers::ParseError;
use std::path::Path;

/// Trait for custom parser plugins
///
/// Implement this trait to create a parser that can read your custom data format
/// and convert it into a PipelineGraph.
pub trait Parser: Send + Sync {
    /// Name of this parser (used for logging and error messages)
    fn name(&self) -> &str;

    /// Parse a file at the given path and return a PipelineGraph
    fn parse(&self, path: &Path) -> Result<PipelineGraph, ParseError>;

    /// File extensions this parser supports (without the dot)
    ///
    /// For example: `vec!["json", "yaml"]`
    fn supported_extensions(&self) -> Vec<&str> {
        vec![]
    }

    /// Optional: Validate if this parser can handle the given file
    ///
    /// Default implementation checks file extension against supported_extensions().
    /// Override to provide custom validation logic (e.g., checking file contents).
    fn can_parse(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return self.supported_extensions().contains(&ext_str);
            }
        }
        false
    }
}

/// Type-erased parser plugin for dynamic dispatch
pub type ParserPlugin = Box<dyn Parser>;

/// Registry for custom parser plugins
pub struct ParserRegistry {
    parsers: Vec<ParserPlugin>,
}

impl ParserRegistry {
    /// Create a new empty parser registry
    pub fn new() -> Self {
        Self {
            parsers: Vec::new(),
        }
    }

    /// Register a custom parser plugin
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lightweaver::plugin::ParserRegistry;
    ///
    /// let mut registry = ParserRegistry::new();
    /// // registry.register(Box::new(MyCustomParser));
    /// ```
    pub fn register(&mut self, parser: ParserPlugin) {
        self.parsers.push(parser);
    }

    /// Find a parser that can handle the given file
    ///
    /// Returns the first parser that indicates it can parse the file.
    pub fn find_parser(&self, path: &Path) -> Option<&dyn Parser> {
        self.parsers
            .iter()
            .find(|p| p.can_parse(path))
            .map(|p| p.as_ref())
    }

    /// Get all registered parsers
    pub fn parsers(&self) -> &[ParserPlugin] {
        &self.parsers
    }

    /// Number of registered parsers
    pub fn len(&self) -> usize {
        self.parsers.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.parsers.is_empty()
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestParser {
        name: String,
        extensions: Vec<&'static str>,
    }

    impl Parser for TestParser {
        fn name(&self) -> &str {
            &self.name
        }

        fn parse(&self, _path: &Path) -> Result<PipelineGraph, ParseError> {
            Ok(PipelineGraph::new())
        }

        fn supported_extensions(&self) -> Vec<&str> {
            self.extensions.clone()
        }
    }

    #[test]
    fn test_parser_registry() {
        let mut registry = ParserRegistry::new();

        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());

        registry.register(Box::new(TestParser {
            name: "test".to_string(),
            extensions: vec!["test"],
        }));

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_find_parser() {
        let mut registry = ParserRegistry::new();

        registry.register(Box::new(TestParser {
            name: "json_parser".to_string(),
            extensions: vec!["json"],
        }));

        registry.register(Box::new(TestParser {
            name: "yaml_parser".to_string(),
            extensions: vec!["yaml", "yml"],
        }));

        // Should find json parser
        let json_path = Path::new("test.json");
        assert!(registry.find_parser(json_path).is_some());
        assert_eq!(registry.find_parser(json_path).unwrap().name(), "json_parser");

        // Should find yaml parser
        let yaml_path = Path::new("test.yaml");
        assert!(registry.find_parser(yaml_path).is_some());
        assert_eq!(registry.find_parser(yaml_path).unwrap().name(), "yaml_parser");

        // Should not find parser for unsupported extension
        let unknown_path = Path::new("test.unknown");
        assert!(registry.find_parser(unknown_path).is_none());
    }
}
