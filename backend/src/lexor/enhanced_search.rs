use crate::lexor::types::*;
use tantivy::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSearchQuery {
    pub query: String,
    pub semantic_search: bool,
    pub similarity_threshold: f32,
    pub boost_recent: bool,
    pub boost_popular: bool,
    pub include_tests: bool,
    pub max_file_size: Option<u64>,
    pub date_range: Option<DateRange>,
    pub complexity_filter: Option<ComplexityRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub from: chrono::DateTime<chrono::Utc>,
    pub to: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityRange {
    pub min_complexity: u32,
    pub max_complexity: u32,
}

pub struct EnhancedSearchEngine {
    base_engine: crate::lexor::search::SearchEngine,
    embeddings: HashMap<String, Vec<f32>>,
    popularity_scores: HashMap<String, f32>,
}

impl EnhancedSearchEngine {
    pub fn new(index: Index, schema: Schema) -> Result<Self, Box<dyn std::error::Error>> {
        let base_engine = crate::lexor::search::SearchEngine::new(index, schema)?;
        
        Ok(Self {
            base_engine,
            embeddings: HashMap::new(),
            popularity_scores: HashMap::new(),
        })
    }

    pub fn enhanced_search(&self, query: &EnhancedSearchQuery) -> Result<SearchResult, Box<dyn std::error::Error>> {
        let mut base_query = SearchQuery {
            query: query.query.clone(),
            query_type: QueryType::FullText,
            projects: None,
            file_types: None,
            languages: None,
            path_filter: None,
            case_sensitive: false,
            regex: false,
            limit: 100,
            offset: 0,
        };

        // Apply filters
        if !query.include_tests {
            base_query.path_filter = Some("!test".to_string());
        }

        let mut results = self.base_engine.search(&base_query)?;

        // Apply enhanced scoring
        if query.boost_recent || query.boost_popular || query.semantic_search {
            self.apply_enhanced_scoring(&mut results, query)?;
        }

        // Apply post-filters
        self.apply_post_filters(&mut results, query)?;

        Ok(results)
    }

    fn apply_enhanced_scoring(&self, results: &mut SearchResult, query: &EnhancedSearchQuery) -> Result<(), Box<dyn std::error::Error>> {
        for hit in &mut results.results {
            let mut boost = 1.0;

            // Recency boost
            if query.boost_recent {
                let days_old = (chrono::Utc::now() - hit.file.last_modified).num_days();
                boost *= (1.0 / (1.0 + days_old as f32 * 0.01)).max(0.1);
            }

            // Popularity boost
            if query.boost_popular {
                if let Some(popularity) = self.popularity_scores.get(&hit.file.path.to_string_lossy().to_string()) {
                    boost *= 1.0 + popularity;
                }
            }

            // Semantic similarity
            if query.semantic_search {
                if let Some(similarity) = self.calculate_semantic_similarity(&query.query, &hit.file.path.to_string_lossy()) {
                    if similarity >= query.similarity_threshold {
                        boost *= 1.0 + similarity;
                    }
                }
            }

            hit.score *= boost;
        }

        // Re-sort by new scores
        results.results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(())
    }

    fn apply_post_filters(&self, results: &mut SearchResult, query: &EnhancedSearchQuery) -> Result<(), Box<dyn std::error::Error>> {
        results.results.retain(|hit| {
            // File size filter
            if let Some(max_size) = query.max_file_size {
                if hit.file.size > max_size {
                    return false;
                }
            }

            // Date range filter
            if let Some(date_range) = &query.date_range {
                if hit.file.last_modified < date_range.from || hit.file.last_modified > date_range.to {
                    return false;
                }
            }

            true
        });

        Ok(())
    }

    fn calculate_semantic_similarity(&self, query: &str, file_path: &str) -> Option<f32> {
        // Simplified semantic similarity - in production, use proper embeddings
        let query_words: std::collections::HashSet<_> = query.to_lowercase().split_whitespace().collect();
        let path_words: std::collections::HashSet<_> = file_path.to_lowercase().split(&['/', '\\', '.', '_', '-'][..]).collect();
        
        let intersection = query_words.intersection(&path_words).count();
        let union = query_words.union(&path_words).count();
        
        if union > 0 {
            Some(intersection as f32 / union as f32)
        } else {
            None
        }
    }

    pub fn update_popularity_scores(&mut self, file_access_stats: HashMap<String, u32>) {
        for (file_path, access_count) in file_access_stats {
            let score = (access_count as f32).ln() / 10.0; // Logarithmic scaling
            self.popularity_scores.insert(file_path, score);
        }
    }
}