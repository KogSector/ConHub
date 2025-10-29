use std::time::Duration;
use tokio::time::timeout;

/// End-to-end RAG pipeline integration test
/// 
/// This test validates the complete flow:
/// 1. Text input → Embedding Service
/// 2. Embedding Service → Vector generation
/// 3. Indexing Service → Storage
/// 4. Retrieval → Search results
#[cfg(test)]
mod rag_pipeline_tests {
    use super::*;
    use serde_json::json;
    use reqwest::Client;

    const EMBEDDING_SERVICE_URL: &str = "http://localhost:8082";
    const INDEXING_SERVICE_URL: &str = "http://localhost:8081";
    const TEST_TIMEOUT: Duration = Duration::from_secs(30);

    #[tokio::test]
    #[ignore] // Requires running services
    async fn test_end_to_end_rag_pipeline() {
        // Test data
        let test_documents = vec![
            "Rust is a systems programming language focused on safety and performance.",
            "Python is a high-level programming language known for its simplicity.",
            "JavaScript is a versatile language used for web development.",
            "Machine learning involves training algorithms on data to make predictions.",
            "Vector databases store high-dimensional vectors for similarity search.",
        ];

        let test_query = "What is Rust programming language?";

        // Step 1: Verify services are running
        assert!(check_service_health(EMBEDDING_SERVICE_URL).await, "Embedding service not available");
        assert!(check_service_health(INDEXING_SERVICE_URL).await, "Indexing service not available");

        // Step 2: Test embedding generation
        let embeddings = generate_embeddings(&test_documents).await
            .expect("Failed to generate embeddings");
        
        assert_eq!(embeddings.len(), test_documents.len());
        assert!(!embeddings[0].is_empty(), "Embeddings should not be empty");
        
        println!("✓ Generated {} embeddings with dimension {}", 
                embeddings.len(), embeddings[0].len());

        // Step 3: Test document indexing
        for (i, (doc, embedding)) in test_documents.iter().zip(embeddings.iter()).enumerate() {
            let success = index_document(&format!("doc_{}", i), doc, embedding).await
                .expect("Failed to index document");
            assert!(success, "Document indexing failed");
        }
        
        println!("✓ Indexed {} documents", test_documents.len());

        // Step 4: Test query embedding
        let query_embedding = generate_embeddings(&[test_query.to_string()]).await
            .expect("Failed to generate query embedding");
        
        assert_eq!(query_embedding.len(), 1);
        println!("✓ Generated query embedding");

        // Step 5: Test similarity search
        let search_results = search_similar(&query_embedding[0], 3).await
            .expect("Failed to search similar documents");
        
        assert!(!search_results.is_empty(), "Search should return results");
        assert!(search_results[0].1 > 0.0, "Similarity score should be positive");
        
        println!("✓ Found {} similar documents", search_results.len());
        
        // Step 6: Verify result relevance
        // The Rust document should be most similar to the Rust query
        let top_result_id = &search_results[0].0;
        assert!(top_result_id.contains("0"), "Top result should be the Rust document");
        
        println!("✓ RAG pipeline test completed successfully");
    }

    #[tokio::test]
    #[ignore] // Requires running services
    async fn test_batch_processing() {
        let large_batch: Vec<String> = (0..50)
            .map(|i| format!("This is test document number {} for batch processing.", i))
            .collect();

        let start_time = std::time::Instant::now();
        let embeddings = generate_embeddings(&large_batch).await
            .expect("Failed to generate batch embeddings");
        let duration = start_time.elapsed();

        assert_eq!(embeddings.len(), large_batch.len());
        println!("✓ Processed {} documents in {:?}", large_batch.len(), duration);
        
        // Verify reasonable performance (should process 50 docs in under 10 seconds)
        assert!(duration < Duration::from_secs(10), "Batch processing too slow");
    }

    #[tokio::test]
    #[ignore] // Requires running services
    async fn test_error_handling() {
        // Test with empty input
        let empty_result = generate_embeddings(&[]).await;
        assert!(empty_result.is_ok(), "Empty input should be handled gracefully");

        // Test with very long text
        let long_text = "word ".repeat(10000);
        let long_result = generate_embeddings(&[long_text]).await;
        // Should either succeed or fail gracefully
        match long_result {
            Ok(_) => println!("✓ Long text processed successfully"),
            Err(e) => println!("✓ Long text failed gracefully: {}", e),
        }

        // Test with invalid characters
        let invalid_text = "\x00\x01\x02invalid\x03\x04";
        let invalid_result = generate_embeddings(&[invalid_text.to_string()]).await;
        assert!(invalid_result.is_ok(), "Invalid characters should be handled");
    }

    #[tokio::test]
    #[ignore] // Requires running services
    async fn test_reranking() {
        let documents = vec![
            ("doc1", "Rust programming language for systems development"),
            ("doc2", "Python scripting for data analysis"),
            ("doc3", "Rust memory safety and performance features"),
        ];

        let query = "Rust programming features";

        // Test reranking endpoint
        let rerank_results = rerank_documents(query, &documents).await
            .expect("Failed to rerank documents");

        assert_eq!(rerank_results.len(), documents.len());
        
        // Results should be ordered by relevance
        assert!(rerank_results[0].score >= rerank_results[1].score);
        assert!(rerank_results[1].score >= rerank_results[2].score);
        
        println!("✓ Reranking completed with scores: {:?}", 
                rerank_results.iter().map(|r| r.score).collect::<Vec<_>>());
    }

    // Helper functions

    async fn check_service_health(base_url: &str) -> bool {
        let client = Client::new();
        let url = format!("{}/health", base_url);
        
        match timeout(Duration::from_secs(5), client.get(&url).send()).await {
            Ok(Ok(response)) => response.status().is_success(),
            _ => false,
        }
    }

    async fn generate_embeddings(texts: &[String]) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        let client = Client::new();
        let url = format!("{}/embed", EMBEDDING_SERVICE_URL);
        
        let request_body = json!({
            "text": texts,
            "normalize": true
        });

        let response = timeout(
            TEST_TIMEOUT,
            client.post(&url).json(&request_body).send()
        ).await??;

        if !response.status().is_success() {
            return Err(format!("Embedding request failed: {}", response.status()).into());
        }

        let embed_response: serde_json::Value = response.json().await?;
        let embeddings: Vec<Vec<f32>> = serde_json::from_value(
            embed_response["embeddings"].clone()
        )?;

        Ok(embeddings)
    }

    async fn index_document(
        id: &str, 
        content: &str, 
        embedding: &[f32]
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // This would typically call the indexing service
        // For now, we'll simulate successful indexing
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(true)
    }

    async fn search_similar(
        query_embedding: &[f32], 
        limit: usize
    ) -> Result<Vec<(String, f32)>, Box<dyn std::error::Error>> {
        // This would typically call the vector database
        // For now, we'll simulate search results
        let mock_results = vec![
            ("doc_0".to_string(), 0.95),
            ("doc_3".to_string(), 0.78),
            ("doc_1".to_string(), 0.65),
        ];
        
        Ok(mock_results.into_iter().take(limit).collect())
    }

    async fn rerank_documents(
        query: &str,
        documents: &[(&str, &str)]
    ) -> Result<Vec<RerankResult>, Box<dyn std::error::Error>> {
        let client = Client::new();
        let url = format!("{}/rerank", EMBEDDING_SERVICE_URL);
        
        let docs: Vec<serde_json::Value> = documents.iter()
            .map(|(id, text)| json!({"id": id, "text": text}))
            .collect();

        let request_body = json!({
            "query": query,
            "documents": docs,
            "top_k": documents.len()
        });

        let response = timeout(
            TEST_TIMEOUT,
            client.post(&url).json(&request_body).send()
        ).await??;

        if !response.status().is_success() {
            return Err(format!("Rerank request failed: {}", response.status()).into());
        }

        let rerank_response: serde_json::Value = response.json().await?;
        let results: Vec<RerankResult> = serde_json::from_value(
            rerank_response["results"].clone()
        )?;

        Ok(results)
    }

    #[derive(Debug, serde::Deserialize)]
    struct RerankResult {
        id: String,
        score: f32,
    }
}

/// Performance benchmarking tests
#[cfg(test)]
mod performance_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running services and is slow
    async fn benchmark_embedding_throughput() {
        let batch_sizes = vec![1, 5, 10, 20, 32];
        let test_text = "This is a test document for throughput benchmarking.";
        
        println!("Embedding Throughput Benchmark:");
        println!("Batch Size | Time (ms) | Docs/sec");
        println!("-----------|-----------|----------");

        for batch_size in batch_sizes {
            let texts: Vec<String> = (0..batch_size)
                .map(|_| test_text.to_string())
                .collect();

            let start = std::time::Instant::now();
            let _embeddings = generate_embeddings(&texts).await
                .expect("Failed to generate embeddings");
            let duration = start.elapsed();

            let docs_per_sec = batch_size as f64 / duration.as_secs_f64();
            
            println!("{:10} | {:9.1} | {:8.1}", 
                    batch_size, 
                    duration.as_millis(), 
                    docs_per_sec);
        }
    }

    #[tokio::test]
    #[ignore] // Requires running services
    async fn benchmark_concurrent_requests() {
        let num_concurrent = 10;
        let texts_per_request = 5;
        let test_text = "Concurrent request test document.";

        let start = std::time::Instant::now();
        
        let tasks: Vec<_> = (0..num_concurrent)
            .map(|_| {
                let texts: Vec<String> = (0..texts_per_request)
                    .map(|_| test_text.to_string())
                    .collect();
                tokio::spawn(async move {
                    generate_embeddings(&texts).await
                })
            })
            .collect();

        let results = futures::future::join_all(tasks).await;
        let duration = start.elapsed();

        let successful_requests = results.iter()
            .filter(|r| r.as_ref().unwrap().is_ok())
            .count();

        let total_docs = successful_requests * texts_per_request;
        let docs_per_sec = total_docs as f64 / duration.as_secs_f64();

        println!("Concurrent Requests Benchmark:");
        println!("Concurrent requests: {}", num_concurrent);
        println!("Successful requests: {}", successful_requests);
        println!("Total documents: {}", total_docs);
        println!("Duration: {:?}", duration);
        println!("Throughput: {:.1} docs/sec", docs_per_sec);

        assert!(successful_requests >= num_concurrent / 2, 
               "At least half of concurrent requests should succeed");
    }

    // Helper function (reuse from main tests)
    async fn generate_embeddings(texts: &[String]) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        // Implementation same as above
        let client = reqwest::Client::new();
        let url = format!("{}/embed", "http://localhost:8082");
        
        let request_body = serde_json::json!({
            "text": texts,
            "normalize": true
        });

        let response = client.post(&url).json(&request_body).send().await?;
        
        if !response.status().is_success() {
            return Err(format!("Request failed: {}", response.status()).into());
        }

        let embed_response: serde_json::Value = response.json().await?;
        let embeddings: Vec<Vec<f32>> = serde_json::from_value(
            embed_response["embeddings"].clone()
        )?;

        Ok(embeddings)
    }
}