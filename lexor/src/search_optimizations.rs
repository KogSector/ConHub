use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use dashmap::DashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use log::{info, debug};

/// High-performance search optimization structures
#[derive(Debug, Clone)]
pub struct SearchOptimizations {
    // Symbol name to file mappings for fast symbol lookup
    symbol_index: Arc<DashMap<String, Vec<Uuid>>>, // symbol_name -> [file_ids]
    
    // File extension to file mappings for language-specific searches
    extension_index: Arc<DashMap<String, Vec<Uuid>>>, // extension -> [file_ids]
    
    // Word frequency index for relevance scoring
    word_frequency: Arc<DashMap<String, u32>>,
    
    // Recent access cache for hot paths
    access_cache: Arc<DashMap<String, (Uuid, std::time::Instant)>>, // key -> (value, last_access)
}

impl SearchOptimizations {
    pub fn new() -> Self {
        Self {
            symbol_index: Arc::new(DashMap::new()),
            extension_index: Arc::new(DashMap::new()),
            word_frequency: Arc::new(DashMap::new()),
            access_cache: Arc::new(DashMap::new()),
        }
    }

    /// Add a symbol to the search index
    pub fn index_symbol(&self, symbol_name: &str, file_id: Uuid) {
        let mut entry = self.symbol_index.entry(symbol_name.to_lowercase()).or_insert(Vec::new());
        if !entry.contains(&file_id) {
            entry.push(file_id);
        }
        
        // Update word frequency
        let mut freq = self.word_frequency.entry(symbol_name.to_lowercase()).or_insert(0);
        *freq += 1;
    }

    /// Add a file to the extension index
    pub fn index_file_extension(&self, extension: &str, file_id: Uuid) {
        let mut entry = self.extension_index.entry(extension.to_lowercase()).or_insert(Vec::new());
        if !entry.contains(&file_id) {
            entry.push(file_id);
        }
    }

    /// Fast symbol lookup with ranking by frequency
    pub fn find_symbol_files(&self, symbol_name: &str) -> Vec<(Uuid, u32)> {
        let key = symbol_name.to_lowercase();
        
        // Check cache first
        if let Some(entry) = self.access_cache.get(&key) {
            let (file_id, last_access) = entry.value();
            if last_access.elapsed().as_secs() < 300 { // 5 minute cache
                debug!("Cache hit for symbol: {}", symbol_name);
                return vec![(*file_id, 100)]; // High score for cached items
            }
        }

        if let Some(file_ids) = self.symbol_index.get(&key) {
            let frequency = self.word_frequency.get(&key).map(|f| *f.value()).unwrap_or(1);
            
            let results: Vec<(Uuid, u32)> = file_ids.iter()
                .map(|&file_id| (file_id, frequency))
                .collect();
            
            // Cache the first result for hot paths
            if let Some(&(first_file_id, _)) = results.first() {
                self.access_cache.insert(key, (first_file_id, std::time::Instant::now()));
            }
            
            results
        } else {
            Vec::new()
        }
    }

    /// Find files by extension with performance optimization
    pub fn find_files_by_extension(&self, extension: &str) -> Vec<Uuid> {
        let key = extension.to_lowercase();
        
        if let Some(file_ids) = self.extension_index.get(&key) {
            file_ids.clone()
        } else {
            Vec::new()
        }
    }

    /// Fuzzy search with efficient string matching
    pub fn fuzzy_find_symbols(&self, query: &str, max_results: usize) -> Vec<(String, u32, f64)> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        
        for entry in self.symbol_index.iter() {
            let symbol_name = entry.key();
            let frequency = self.word_frequency.get(symbol_name).map(|f| *f.value()).unwrap_or(1);
            
            // Calculate similarity score using various metrics
            let similarity = calculate_similarity(&query_lower, symbol_name);
            
            if similarity > 0.3 { // Threshold for relevance
                results.push((symbol_name.clone(), frequency, similarity));
            }
        }
        
        // Sort by similarity * frequency for best results
        results.sort_by(|a, b| {
            let score_a = a.2 * (a.1 as f64).log10();
            let score_b = b.2 * (b.1 as f64).log10();
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        results.truncate(max_results);
        results
    }

    /// Clean up expired cache entries
    pub fn cleanup_cache(&self) {
        let cutoff = std::time::Instant::now() - std::time::Duration::from_secs(600); // 10 minutes
        self.access_cache.retain(|_, (_, last_access)| *last_access > cutoff);
    }

    /// Get optimization statistics
    pub fn get_stats(&self) -> OptimizationStats {
        OptimizationStats {
            symbols_indexed: self.symbol_index.len(),
            extensions_indexed: self.extension_index.len(),
            cache_size: self.access_cache.len(),
            total_word_frequency: self.word_frequency.iter().map(|e| *e.value()).sum(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationStats {
    pub symbols_indexed: usize,
    pub extensions_indexed: usize,
    pub cache_size: usize,
    pub total_word_frequency: u32,
}

/// Calculate similarity between two strings using multiple metrics
fn calculate_similarity(query: &str, target: &str) -> f64 {
    if query == target {
        return 1.0;
    }
    
    if target.contains(query) {
        return 0.8;
    }
    
    if query.len() < 3 || target.len() < 3 {
        return if target.starts_with(query) { 0.7 } else { 0.0 };
    }
    
    // Levenshtein distance based similarity
    let distance = levenshtein_distance(query, target);
    let max_len = query.len().max(target.len());
    
    if max_len == 0 {
        return 1.0;
    }
    
    1.0 - (distance as f64 / max_len as f64)
}

/// Efficient Levenshtein distance calculation
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();
    
    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }
    
    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row = vec![0; b_len + 1];
    
    for i in 1..=a_len {
        curr_row[0] = i;
        
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr_row[j] = std::cmp::min(
                std::cmp::min(curr_row[j - 1] + 1, prev_row[j] + 1),
                prev_row[j - 1] + cost
            );
        }
        
        std::mem::swap(&mut prev_row, &mut curr_row);
    }
    
    prev_row[b_len]
}

impl Default for SearchOptimizations {
    fn default() -> Self {
        Self::new()
    }
}

/// Periodic cleanup task for search optimizations
pub async fn optimization_cleanup_task(optimizations: Arc<SearchOptimizations>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(600)); // Every 10 minutes
    
    loop {
        interval.tick().await;
        optimizations.cleanup_cache();
        
        let stats = optimizations.get_stats();
        info!("Search optimization stats: {:?}", stats);
    }
}