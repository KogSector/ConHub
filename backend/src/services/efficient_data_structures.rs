use std::collections::{HashMap, HashSet, BTreeMap, VecDeque};
use std::sync::{Arc, Weak};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, Mutex};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use dashmap::DashMap;
use tantivy::{
    collector::TopDocs, query::QueryParser, schema::*, Directory, Index, IndexReader, IndexWriter,
    ReloadPolicy, TantivyDocument, Term,
};
use std::path::Path;
use chrono::{DateTime, Utc};

/// Efficient Trie data structure for prefix searches and autocomplete
#[derive(Debug, Clone)]
pub struct TrieNode {
    pub children: HashMap<char, Arc<RwLock<TrieNode>>>,
    pub is_end_of_word: bool,
    pub frequency: u32,
    pub data: Option<String>,
}

impl TrieNode {
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            is_end_of_word: false,
            frequency: 0,
            data: None,
        }
    }
}

#[derive(Debug)]
pub struct OptimizedTrie {
    root: Arc<RwLock<TrieNode>>,
    total_words: Arc<RwLock<u32>>,
}

impl OptimizedTrie {
    pub fn new() -> Self {
        Self {
            root: Arc::new(RwLock::new(TrieNode::new())),
            total_words: Arc::new(RwLock::new(0)),
        }
    }

    /// Insert a word with frequency tracking
    pub async fn insert(&self, word: &str, data: Option<String>) {
        let mut current = Arc::clone(&self.root);
        
        for ch in word.chars() {
            let current_node = current.read().await;
            
            if let Some(child) = current_node.children.get(&ch) {
                drop(current_node);
                current = Arc::clone(child);
            } else {
                drop(current_node);
                let mut current_mut = current.write().await;
                let new_node = Arc::new(RwLock::new(TrieNode::new()));
                current_mut.children.insert(ch, Arc::clone(&new_node));
                drop(current_mut);
                current = new_node;
            }
        }

        let mut final_node = current.write().await;
        if !final_node.is_end_of_word {
            final_node.is_end_of_word = true;
            let mut total = self.total_words.write().await;
            *total += 1;
        }
        final_node.frequency += 1;
        if let Some(data) = data {
            final_node.data = Some(data);
        }
    }

    /// Search for words with prefix
    pub async fn search_prefix(&self, prefix: &str) -> Vec<(String, u32, Option<String>)> {
        let mut results = Vec::new();
        let mut current = Arc::clone(&self.root);

        // Navigate to the prefix node
        for ch in prefix.chars() {
            let current_node = current.read().await;
            if let Some(child) = current_node.children.get(&ch) {
                drop(current_node);
                current = Arc::clone(child);
            } else {
                return results; // Prefix not found
            }
        }

        // Collect all words with this prefix
        self.collect_words(current, prefix.to_string(), &mut results).await;
        
        // Sort by frequency (descending)
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
    }

    async fn collect_words(
        &self,
        node: Arc<RwLock<TrieNode>>,
        current_word: String,
        results: &mut Vec<(String, u32, Option<String>)>,
    ) {
        let node_read = node.read().await;
        
        if node_read.is_end_of_word {
            results.push((current_word.clone(), node_read.frequency, node_read.data.clone()));
        }

        for (ch, child) in &node_read.children {
            let mut new_word = current_word.clone();
            new_word.push(*ch);
            drop(node_read);
            
            Box::pin(self.collect_words(Arc::clone(child), new_word, results)).await;
            let node_read = node.read().await;
        }
    }

    /// Get autocomplete suggestions
    pub async fn autocomplete(&self, prefix: &str, limit: usize) -> Vec<String> {
        let results = self.search_prefix(prefix).await;
        results.into_iter()
            .take(limit)
            .map(|(word, _, _)| word)
            .collect()
    }
}

/// Bloom Filter for efficient membership testing
#[derive(Debug, Clone)]
pub struct BloomFilter {
    bit_array: Vec<bool>,
    size: usize,
    hash_functions: u32,
    items_count: u32,
}

impl BloomFilter {
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
        let size = Self::optimal_size(expected_items, false_positive_rate);
        let hash_functions = Self::optimal_hash_count(size, expected_items);
        
        Self {
            bit_array: vec![false; size],
            size,
            hash_functions,
            items_count: 0,
        }
    }

    fn optimal_size(n: usize, p: f64) -> usize {
        let m = -((n as f64) * p.ln()) / (2.0_f64.ln().powi(2));
        m.ceil() as usize
    }

    fn optimal_hash_count(m: usize, n: usize) -> u32 {
        let k = ((m as f64) / (n as f64)) * 2.0_f64.ln();
        k.ceil() as u32
    }

    fn hash(&self, item: &str, seed: u32) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        seed.hash(&mut hasher);
        (hasher.finish() as usize) % self.size
    }

    pub fn insert(&mut self, item: &str) {
        for i in 0..self.hash_functions {
            let index = self.hash(item, i);
            self.bit_array[index] = true;
        }
        self.items_count += 1;
    }

    pub fn contains(&self, item: &str) -> bool {
        for i in 0..self.hash_functions {
            let index = self.hash(item, i);
            if !self.bit_array[index] {
                return false;
            }
        }
        true
    }

    pub fn estimated_false_positive_rate(&self) -> f64 {
        let k = self.hash_functions as f64;
        let m = self.size as f64;
        let n = self.items_count as f64;
        
        (1.0 - (-k * n / m).exp()).powf(k)
    }
}

/// Skip List for ordered data with fast search, insert, and delete
#[derive(Debug)]
pub struct SkipListNode<K, V> {
    key: Option<K>,
    value: Option<V>,
    forward: Vec<Option<Arc<RwLock<SkipListNode<K, V>>>>>,
}

impl<K, V> SkipListNode<K, V> {
    fn new(level: usize) -> Self {
        Self {
            key: None,
            value: None,
            forward: vec![None; level + 1],
        }
    }

    fn new_with_data(key: K, value: V, level: usize) -> Self {
        Self {
            key: Some(key),
            value: Some(value),
            forward: vec![None; level + 1],
        }
    }
}

#[derive(Debug)]
pub struct SkipList<K, V> {
    header: Arc<RwLock<SkipListNode<K, V>>>,
    max_level: usize,
    current_level: usize,
    size: usize,
}

impl<K: Ord + Clone, V: Clone> SkipList<K, V> {
    pub fn new(max_level: usize) -> Self {
        Self {
            header: Arc::new(RwLock::new(SkipListNode::new(max_level))),
            max_level,
            current_level: 0,
            size: 0,
        }
    }

    fn random_level(&self) -> usize {
        use rand::Rng;
        let mut level = 0;
        let mut rng = rand::thread_rng();
        
        while rng.gen_bool(0.5) && level < self.max_level {
            level += 1;
        }
        level
    }

    pub async fn insert(&mut self, key: K, value: V) {
        let level = self.random_level();
        let new_node = Arc::new(RwLock::new(SkipListNode::new_with_data(key.clone(), value, level)));
        
        let mut update = vec![None; self.max_level + 1];
        let mut current = Arc::clone(&self.header);

        // Find position to insert
        for i in (0..=self.current_level).rev() {
            loop {
                let current_read = current.read().await;
                if let Some(ref next) = current_read.forward[i] {
                    let next_read = next.read().await;
                    if let Some(ref next_key) = next_read.key {
                        if *next_key < key {
                            drop(next_read);
                            drop(current_read);
                            current = Arc::clone(next);
                            continue;
                        }
                    }
                }
                break;
            }
            update[i] = Some(Arc::clone(&current));
        }

        // Update forward pointers
        for i in 0..=level {
            if let Some(ref update_node) = update[i] {
                let mut update_write = update_node.write().await;
                let mut new_write = new_node.write().await;
                new_write.forward[i] = update_write.forward[i].take();
                update_write.forward[i] = Some(Arc::clone(&new_node));
            }
        }

        if level > self.current_level {
            self.current_level = level;
        }
        self.size += 1;
    }

    pub async fn search(&self, key: &K) -> Option<V> {
        let mut current = Arc::clone(&self.header);

        for i in (0..=self.current_level).rev() {
            loop {
                let current_read = current.read().await;
                if let Some(ref next) = current_read.forward[i] {
                    let next_read = next.read().await;
                    if let Some(ref next_key) = next_read.key {
                        if *next_key < *key {
                            drop(next_read);
                            drop(current_read);
                            current = Arc::clone(next);
                            continue;
                        } else if *next_key == *key {
                            return next_read.value.clone();
                        }
                    }
                }
                break;
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.size
    }
}

/// Real-time search index with live updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub score: f32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct RealTimeSearchIndex {
    // Tantivy index for full-text search
    index: Index,
    index_writer: Arc<Mutex<IndexWriter>>,
    index_reader: IndexReader,
    query_parser: QueryParser,
    
    // Fast lookup structures
    document_cache: Arc<DashMap<String, SearchDocument>>,
    trie_index: OptimizedTrie,
    tag_index: Arc<DashMap<String, HashSet<String>>>, // tag -> document_ids
    bloom_filter: Arc<RwLock<BloomFilter>>,
    
    // Performance tracking
    last_update: Arc<RwLock<Instant>>,
    update_queue: Arc<Mutex<VecDeque<SearchDocument>>>,
    
    // Schema fields
    id_field: Field,
    title_field: Field,
    content_field: Field,
    tags_field: Field,
    timestamp_field: Field,
    score_field: Field,
}

impl RealTimeSearchIndex {
    pub async fn new(index_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Create schema
        let mut schema_builder = Schema::builder();
        let id_field = schema_builder.add_text_field("id", STRING | STORED);
        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let content_field = schema_builder.add_text_field("content", TEXT);
        let tags_field = schema_builder.add_text_field("tags", STRING | STORED);
        let timestamp_field = schema_builder.add_date_field("timestamp", STORED);
        let score_field = schema_builder.add_f64_field("score", STORED);
        let schema = schema_builder.build();

        // Create or open index
        let index = if Path::new(index_path).exists() {
            Index::open_in_dir(index_path)?
        } else {
            std::fs::create_dir_all(index_path)?;
            Index::create_in_dir(index_path, schema.clone())?
        };

        let index_writer = Arc::new(Mutex::new(index.writer(50_000_000)?));
        let index_reader = index.reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;

        let query_parser = QueryParser::for_index(&index, vec![title_field, content_field]);

        Ok(Self {
            index,
            index_writer,
            index_reader,
            query_parser,
            document_cache: Arc::new(DashMap::new()),
            trie_index: OptimizedTrie::new(),
            tag_index: Arc::new(DashMap::new()),
            bloom_filter: Arc::new(RwLock::new(BloomFilter::new(100000, 0.01))),
            last_update: Arc::new(RwLock::new(Instant::now())),
            update_queue: Arc::new(Mutex::new(VecDeque::new())),
            id_field,
            title_field,
            content_field,
            tags_field,
            timestamp_field,
            score_field,
        })
    }

    /// Add or update a document in the index
    pub async fn index_document(&self, document: SearchDocument) -> Result<(), Box<dyn std::error::Error>> {
        // Update bloom filter
        {
            let mut bloom = self.bloom_filter.write().await;
            bloom.insert(&document.id);
        }

        // Update trie with title and content words
        let words: Vec<&str> = document.title.split_whitespace()
            .chain(document.content.split_whitespace())
            .collect();
        
        for word in words {
            self.trie_index.insert(word.to_lowercase().as_str(), Some(document.id.clone())).await;
        }

        // Update tag index
        for tag in &document.tags {
            let mut tag_docs = self.tag_index.entry(tag.clone()).or_insert_with(HashSet::new);
            tag_docs.insert(document.id.clone());
        }

        // Cache document
        self.document_cache.insert(document.id.clone(), document.clone());

        // Add to Tantivy index
        let mut tantivy_doc = TantivyDocument::new();
        tantivy_doc.add_text(self.id_field, &document.id);
        tantivy_doc.add_text(self.title_field, &document.title);
        tantivy_doc.add_text(self.content_field, &document.content);
        tantivy_doc.add_text(self.tags_field, &document.tags.join(" "));
        tantivy_doc.add_date(self.timestamp_field, tantivy::DateTime::from_utc(document.timestamp.into()));
        tantivy_doc.add_f64(self.score_field, document.score as f64);

        {
            let mut writer = self.index_writer.lock().await;
            writer.add_document(tantivy_doc)?;
            writer.commit()?;
        }

        // Update last modification time
        {
            let mut last_update = self.last_update.write().await;
            *last_update = Instant::now();
        }

        Ok(())
    }

    /// Search documents with multiple strategies
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        filters: Option<HashMap<String, String>>,
    ) -> Result<Vec<SearchDocument>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        let mut seen_ids = HashSet::new();

        // 1. Exact prefix matching using Trie
        let trie_results = self.trie_index.search_prefix(query, limit).await;
        for (_, _, doc_id) in trie_results {
            if let Some(doc_id) = doc_id {
                if let Some(doc) = self.document_cache.get(&doc_id) {
                    if seen_ids.insert(doc_id) {
                        results.push(doc.clone());
                    }
                }
            }
        }

        // 2. Full-text search using Tantivy
        if results.len() < limit {
            let searcher = self.index_reader.searcher();
            let query_obj = self.query_parser.parse_query(query)?;
            let top_docs = searcher.search(&query_obj, &TopDocs::with_limit(limit * 2))?;

            for (score, doc_address) in top_docs {
                let retrieved_doc = searcher.doc(doc_address)?;
                if let Some(id_value) = retrieved_doc.get_first(self.id_field) {
                    let id = id_value.as_text().unwrap_or("");
                    if seen_ids.insert(id.to_string()) {
                        if let Some(cached_doc) = self.document_cache.get(id) {
                            let mut doc = cached_doc.clone();
                            doc.score = score;
                            results.push(doc);
                        }
                    }
                }
                if results.len() >= limit {
                    break;
                }
            }
        }

        // 3. Apply filters if provided
        if let Some(filters) = filters {
            results.retain(|doc| {
                filters.iter().all(|(key, value)| {
                    doc.metadata.get(key).map_or(false, |v| v == value)
                })
            });
        }

        // Sort by relevance score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results)
    }

    /// Get autocomplete suggestions
    pub async fn autocomplete(&self, prefix: &str, limit: usize) -> Vec<String> {
        self.trie_index.autocomplete(prefix, limit).await
    }

    /// Search by tags
    pub async fn search_by_tags(&self, tags: &[String], limit: usize) -> Vec<SearchDocument> {
        let mut tag_matches: HashMap<String, usize> = HashMap::new();
        
        for tag in tags {
            if let Some(doc_ids) = self.tag_index.get(tag) {
                for doc_id in doc_ids.iter() {
                    *tag_matches.entry(doc_id.clone()).or_insert(0) += 1;
                }
            }
        }

        let mut results: Vec<_> = tag_matches.into_iter()
            .filter_map(|(doc_id, match_count)| {
                self.document_cache.get(&doc_id).map(|doc| (doc.clone(), match_count))
            })
            .collect();

        // Sort by number of tag matches (descending) then by score
        results.sort_by(|a, b| {
            b.1.cmp(&a.1).then(
                b.0.score.partial_cmp(&a.0.score).unwrap_or(std::cmp::Ordering::Equal)
            )
        });

        results.into_iter()
            .take(limit)
            .map(|(doc, _)| doc)
            .collect()
    }

    /// Check if document exists using bloom filter (fast pre-check)
    pub async fn might_contain(&self, doc_id: &str) -> bool {
        let bloom = self.bloom_filter.read().await;
        bloom.contains(doc_id)
    }

    /// Get real-time statistics
    pub async fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("total_documents".to_string(), self.document_cache.len() as u64);
        stats.insert("total_tags".to_string(), self.tag_index.len() as u64);
        
        let last_update = self.last_update.read().await;
        stats.insert("seconds_since_last_update".to_string(), last_update.elapsed().as_secs());
        
        let bloom = self.bloom_filter.read().await;
        stats.insert("bloom_filter_items".to_string(), bloom.items_count as u64);
        
        stats
    }

    /// Cleanup and optimize index
    pub async fn optimize(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = self.index_writer.lock().await;
        writer.commit()?;
        
        // Clean up old entries in bloom filter if false positive rate is too high
        let bloom = self.bloom_filter.read().await;
        if bloom.estimated_false_positive_rate() > 0.05 {
            drop(bloom);
            let mut bloom_mut = self.bloom_filter.write().await;
            *bloom_mut = BloomFilter::new(self.document_cache.len() * 2, 0.01);
            
            // Rebuild bloom filter
            for doc in self.document_cache.iter() {
                bloom_mut.insert(doc.key());
            }
        }
        
        Ok(())
    }
}

/// Background task for index maintenance
pub async fn search_index_maintenance_task(index: Arc<RealTimeSearchIndex>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // Every 5 minutes
    
    loop {
        interval.tick().await;
        
        if let Err(e) = index.optimize().await {
            log::error!("Failed to optimize search index: {}", e);
        }
        
        let stats = index.get_stats().await;
        log::info!(
            "Search index stats - Documents: {}, Tags: {}, Last update: {}s ago",
            stats.get("total_documents").unwrap_or(&0),
            stats.get("total_tags").unwrap_or(&0),
            stats.get("seconds_since_last_update").unwrap_or(&0)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_trie_operations() {
        let trie = OptimizedTrie::new();
        
        trie.insert("hello", Some("data1".to_string())).await;
        trie.insert("help", Some("data2".to_string())).await;
        trie.insert("world", Some("data3".to_string())).await;
        
        let results = trie.search_prefix("hel").await;
        assert_eq!(results.len(), 2);
        
        let suggestions = trie.autocomplete("he", 5).await;
        assert!(suggestions.contains(&"hello".to_string()));
        assert!(suggestions.contains(&"help".to_string()));
    }

    #[test]
    fn test_bloom_filter() {
        let mut bloom = BloomFilter::new(1000, 0.01);
        
        bloom.insert("test1");
        bloom.insert("test2");
        
        assert!(bloom.contains("test1"));
        assert!(bloom.contains("test2"));
        assert!(!bloom.contains("test3"));
    }

    #[tokio::test]
    async fn test_skip_list() {
        let mut skip_list = SkipList::new(16);
        
        skip_list.insert("key1".to_string(), "value1".to_string()).await;
        skip_list.insert("key2".to_string(), "value2".to_string()).await;
        
        let result = skip_list.search(&"key1".to_string()).await;
        assert_eq!(result, Some("value1".to_string()));
        
        assert_eq!(skip_list.len(), 2);
    }

    #[tokio::test]
    async fn test_real_time_search() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().to_str().unwrap();
        
        let index = RealTimeSearchIndex::new(index_path).await.unwrap();
        
        let doc = SearchDocument {
            id: "doc1".to_string(),
            title: "Test Document".to_string(),
            content: "This is a test document for searching".to_string(),
            tags: vec!["test".to_string(), "document".to_string()],
            timestamp: Utc::now(),
            score: 1.0,
            metadata: HashMap::new(),
        };
        
        index.index_document(doc).await.unwrap();
        
        let results = index.search("test", 10, None).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc1");
        
        let suggestions = index.autocomplete("tes", 5).await;
        assert!(!suggestions.is_empty());
    }
}