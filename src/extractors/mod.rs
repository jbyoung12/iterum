pub mod cli;
pub mod sqlite;
pub mod web;

use crate::models::*;

/// Output from an extractor — zero or more records to store.
#[derive(Debug, Default)]
pub struct Extracted {
    pub facts: Vec<FactRecord>,
    pub constraints: Vec<ConstraintRecord>,
    pub failure_patterns: Vec<FailurePatternRecord>,
    pub observations: Vec<ObservationRecord>,
}

impl Extracted {
    pub fn merge(&mut self, other: Extracted) {
        self.facts.extend(other.facts);
        self.constraints.extend(other.constraints);
        self.failure_patterns.extend(other.failure_patterns);
        self.observations.extend(other.observations);
    }
}

pub trait Extractor: Send + Sync {
    fn can_handle(&self, event: &ToolResultEvent) -> bool;
    fn extract(&self, event: &ToolResultEvent) -> Extracted;
}

pub struct ExtractorRegistry {
    extractors: Vec<Box<dyn Extractor>>,
}

impl Default for ExtractorRegistry {
    fn default() -> Self {
        Self {
            extractors: vec![
                Box::new(sqlite::SqliteExtractor),
                Box::new(web::WebExtractor),
                Box::new(cli::CliExtractor),
            ],
        }
    }
}

impl ExtractorRegistry {
    pub fn extract(&self, event: &ToolResultEvent) -> Extracted {
        let mut result = Extracted::default();
        for extractor in &self.extractors {
            if extractor.can_handle(event) {
                result.merge(extractor.extract(event));
            }
        }
        result
    }
}
