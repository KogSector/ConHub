use crate::services::fusion::*;
use std::collections::HashMap;

/// Comprehensive demonstration of the fusion embedding model
pub fn run_fusion_demo() -> anyhow::Result<()> {
    println!("🚀 Fusion Embedding Model Demo");
    println!("================================\n");

    // Initialize the fusion service
    let mut service = FusionEmbeddingService::new();

    // Demo 1: Reference Model Embeddings
    println!("📊 Demo 1: Reference Model Embeddings");
    println!("--------------------------------------");
    
    let sample_text = "Artificial intelligence is transforming the world through advanced machine learning algorithms.";
    
    // Generate embeddings using reference models
    let bge_embedding = ReferenceModels::bge_embedding(sample_text)?;
    let qwen3_embedding = ReferenceModels::qwen3_embedding(sample_text)?;
    let e5_embedding = ReferenceModels::e5_embedding(sample_text)?;
    
    println!("✅ BGE Embedding: {} dimensions", bge_embedding.dimension);
    println!("   Model: {}", bge_embedding.model_name);
    println!("   First 5 values: {:?}", &bge_embedding.vector[..5]);
    
    println!("✅ Qwen3 Embedding: {} dimensions", qwen3_embedding.dimension);
    println!("   Model: {}", qwen3_embedding.model_name);
    println!("   First 5 values: {:?}", &qwen3_embedding.vector[..5]);
    
    println!("✅ E5 Embedding: {} dimensions", e5_embedding.dimension);
    println!("   Model: {}", e5_embedding.model_name);
    println!("   First 5 values: {:?}", &e5_embedding.vector[..5]);

    // Add embeddings to service
    let bge_id = service.add_embedding(bge_embedding);
    let qwen3_id = service.add_embedding(qwen3_embedding);
    let e5_id = service.add_embedding(e5_embedding);

    println!("\n🔗 Demo 2: Fusion Strategies");
    println!("-----------------------------");

    // Demo 2a: Concatenation Fusion
    let concat_id = service.create_fused_embedding(
        vec![bge_id, e5_id], // Same dimension models
        FusionStrategy::Concatenation,
    )?;
    
    let concat_result = service.get_fused_embedding(&concat_id).unwrap();
    println!("✅ Concatenation Fusion:");
    println!("   Input dimensions: 768 + 768");
    println!("   Output dimension: {}", concat_result.dimension);
    println!("   First 5 values: {:?}", &concat_result.fused_vector[..5]);

    // Demo 2b: Sum Fusion
    let sum_id = service.create_fused_embedding(
        vec![bge_id, e5_id],
        FusionStrategy::Sum,
    )?;
    
    let sum_result = service.get_fused_embedding(&sum_id).unwrap();
    println!("✅ Sum Fusion:");
    println!("   Output dimension: {}", sum_result.dimension);
    println!("   First 5 values: {:?}", &sum_result.fused_vector[..5]);

    // Demo 2c: Weighted Sum Fusion
    let weights = vec![0.7, 0.3]; // Favor BGE model
    let weighted_sum_id = service.create_fused_embedding(
        vec![bge_id, e5_id],
        FusionStrategy::WeightedSum(weights.clone()),
    )?;
    
    let weighted_result = service.get_fused_embedding(&weighted_sum_id).unwrap();
    println!("✅ Weighted Sum Fusion (weights: {:?}):", weights);
    println!("   Output dimension: {}", weighted_result.dimension);
    println!("   First 5 values: {:?}", &weighted_result.fused_vector[..5]);

    // Demo 2d: Average Fusion
    let avg_id = service.create_fused_embedding(
        vec![bge_id, e5_id],
        FusionStrategy::Average,
    )?;
    
    let avg_result = service.get_fused_embedding(&avg_id).unwrap();
    println!("✅ Average Fusion:");
    println!("   Output dimension: {}", avg_result.dimension);
    println!("   First 5 values: {:?}", &avg_result.fused_vector[..5]);

    // Demo 2e: Max Fusion
    let max_id = service.create_fused_embedding(
        vec![bge_id, e5_id],
        FusionStrategy::Max,
    )?;
    
    let max_result = service.get_fused_embedding(&max_id).unwrap();
    println!("✅ Max Fusion:");
    println!("   Output dimension: {}", max_result.dimension);
    println!("   First 5 values: {:?}", &max_result.fused_vector[..5]);

    // Demo 2f: Attention-Weighted Fusion
    let attention_id = service.create_fused_embedding(
        vec![bge_id, e5_id],
        FusionStrategy::AttentionWeighted,
    )?;
    
    let attention_result = service.get_fused_embedding(&attention_id).unwrap();
    println!("✅ Attention-Weighted Fusion:");
    println!("   Output dimension: {}", attention_result.dimension);
    println!("   First 5 values: {:?}", &attention_result.fused_vector[..5]);

    println!("\n🎭 Demo 3: Multimodal Fusion");
    println!("-----------------------------");

    // Create synthetic multimodal embeddings
    let text_embedding = Embedding::new(
        EmbeddingModality::Text,
        (0..512).map(|i| (i as f32 * 0.01).sin()).collect(),
        "text-encoder".to_string(),
    )?;

    let image_embedding = Embedding::new(
        EmbeddingModality::Image,
        (0..512).map(|i| (i as f32 * 0.02).cos()).collect(),
        "vision-transformer".to_string(),
    )?;

    let audio_embedding = Embedding::new(
        EmbeddingModality::Audio,
        (0..512).map(|i| (i as f32 * 0.015).tan().tanh()).collect(),
        "audio-encoder".to_string(),
    )?;

    // Add metadata
    let mut text_metadata = HashMap::new();
    text_metadata.insert("content_type".to_string(), "article".to_string());
    text_metadata.insert("language".to_string(), "english".to_string());
    
    let text_embedding = text_embedding.with_metadata(text_metadata);

    let text_id = service.add_embedding(text_embedding);
    let image_id = service.add_embedding(image_embedding);
    let audio_id = service.add_embedding(audio_embedding);

    // Multimodal fusion
    let multimodal_id = service.create_fused_embedding(
        vec![text_id, image_id, audio_id],
        FusionStrategy::AttentionWeighted,
    )?;

    let multimodal_result = service.get_fused_embedding(&multimodal_id).unwrap();
    println!("✅ Multimodal Fusion (Text + Image + Audio):");
    println!("   Modalities: {} embeddings", multimodal_result.individual_embeddings.len());
    println!("   Output dimension: {}", multimodal_result.dimension);
    println!("   Strategy: {:?}", multimodal_result.fusion_strategy);
    println!("   First 5 values: {:?}", &multimodal_result.fused_vector[..5]);

    // Print modality breakdown
    for (i, emb) in multimodal_result.individual_embeddings.iter().enumerate() {
        println!("   Modality {}: {:?} ({})", i + 1, emb.modality, emb.model_name);
    }

    println!("\n⚡ Demo 4: Parallel Batch Processing");
    println!("------------------------------------");

    // Create multiple embedding pairs for batch processing
    let batch_embeddings: Vec<_> = (0..6)
        .map(|i| {
            let vector: Vec<f32> = (0..256)
                .map(|j| ((i * 100 + j) as f32 * 0.01).sin())
                .collect();
            
            let embedding = Embedding::new(
                EmbeddingModality::Text,
                vector,
                format!("batch-model-{}", i),
            ).unwrap();
            
            service.add_embedding(embedding)
        })
        .collect();

    // Create batch fusion requests
    let batch_requests = vec![
        (vec![batch_embeddings[0], batch_embeddings[1]], FusionStrategy::Sum),
        (vec![batch_embeddings[2], batch_embeddings[3]], FusionStrategy::Average),
        (vec![batch_embeddings[4], batch_embeddings[5]], FusionStrategy::Max),
    ];

    let batch_results = service.batch_fuse_embeddings(batch_requests)?;
    
    println!("✅ Batch Processing Results:");
    println!("   Processed {} fusion operations in parallel", batch_results.len());
    
    for (i, &result_id) in batch_results.iter().enumerate() {
        let result = service.get_fused_embedding(&result_id).unwrap();
        println!("   Batch {}: {} strategy, dimension {}", 
                 i + 1, 
                 match result.fusion_strategy {
                     FusionStrategy::Sum => "Sum",
                     FusionStrategy::Average => "Average", 
                     FusionStrategy::Max => "Max",
                     _ => "Other"
                 },
                 result.dimension);
    }

    println!("\n💾 Demo 5: File I/O Operations");
    println!("-------------------------------");

    // Save embeddings to file
    use std::env;
    let temp_dir = env::temp_dir();
    let save_path = temp_dir.join("fusion_demo_embeddings.bin");
    
    let save_ids = vec![bge_id, e5_id, text_id];
    service.save_embeddings_to_file(&save_path, &save_ids)?;
    println!("✅ Saved {} embeddings to: {:?}", save_ids.len(), save_path);

    // Load embeddings from file
    let mut new_service = FusionEmbeddingService::new();
    let loaded_ids = new_service.load_embeddings_from_file(&save_path)?;
    println!("✅ Loaded {} embeddings from file", loaded_ids.len());

    // Verify loaded embeddings
    for (i, &loaded_id) in loaded_ids.iter().enumerate() {
        let loaded_emb = new_service.get_embedding(&loaded_id).unwrap();
        println!("   Embedding {}: {} ({})", i + 1, loaded_emb.model_name, loaded_emb.dimension);
    }

    println!("\n📈 Demo 6: Performance Analysis");
    println!("--------------------------------");

    // Analyze fusion performance
    let performance_embeddings: Vec<_> = (0..10)
        .map(|i| {
            let vector: Vec<f32> = (0..1024)
                .map(|j| ((i * 1000 + j) as f32 * 0.001).sin())
                .collect();
            
            let embedding = Embedding::new(
                EmbeddingModality::Text,
                vector,
                format!("perf-model-{}", i),
            ).unwrap();
            
            service.add_embedding(embedding)
        })
        .collect();

    use std::time::Instant;
    
    // Test different fusion strategies
    let strategies = vec![
        ("Concatenation", FusionStrategy::Concatenation),
        ("Sum", FusionStrategy::Sum),
        ("Average", FusionStrategy::Average),
        ("Max", FusionStrategy::Max),
        ("Attention-Weighted", FusionStrategy::AttentionWeighted),
    ];

    for (name, strategy) in strategies {
        let start = Instant::now();
        let _result = service.create_fused_embedding(
            performance_embeddings[..5].to_vec(),
            strategy,
        )?;
        let duration = start.elapsed();
        
        println!("✅ {} fusion: {:?}", name, duration);
    }

    println!("\n🎯 Demo 7: Similarity Analysis");
    println!("-------------------------------");

    // Calculate cosine similarity between different fusion results
    let similarity_bge_e5 = cosine_similarity(
        &service.get_embedding(&bge_id).unwrap().vector,
        &service.get_embedding(&e5_id).unwrap().vector,
    );
    
    let similarity_sum_avg = cosine_similarity(
        &service.get_fused_embedding(&sum_id).unwrap().fused_vector,
        &service.get_fused_embedding(&avg_id).unwrap().fused_vector,
    );

    println!("✅ Cosine Similarity Analysis:");
    println!("   BGE vs E5 embeddings: {:.4}", similarity_bge_e5);
    println!("   Sum vs Average fusion: {:.4}", similarity_sum_avg);

    println!("\n🏁 Demo Complete!");
    println!("==================");
    println!("Successfully demonstrated:");
    println!("• Reference model embeddings (BGE, Qwen3, E5)");
    println!("• Multiple fusion strategies");
    println!("• Multimodal embedding fusion");
    println!("• Parallel batch processing");
    println!("• File I/O operations");
    println!("• Performance analysis");
    println!("• Similarity analysis");

    // Cleanup
    std::fs::remove_file(&save_path).ok();

    Ok(())
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// Additional utility functions for demonstration
pub fn print_embedding_stats(embedding: &Embedding) {
    let mean: f32 = embedding.vector.iter().sum::<f32>() / embedding.vector.len() as f32;
    let variance: f32 = embedding.vector.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f32>() / embedding.vector.len() as f32;
    let std_dev = variance.sqrt();
    
    let min_val = embedding.vector.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let max_val = embedding.vector.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    
    println!("📊 Embedding Statistics:");
    println!("   Dimension: {}", embedding.dimension);
    println!("   Mean: {:.6}", mean);
    println!("   Std Dev: {:.6}", std_dev);
    println!("   Min: {:.6}", min_val);
    println!("   Max: {:.6}", max_val);
    println!("   Model: {}", embedding.model_name);
    println!("   Modality: {:?}", embedding.modality);
}

pub fn print_fusion_comparison(service: &FusionEmbeddingService, fusion_ids: &[uuid::Uuid]) {
    println!("🔍 Fusion Strategy Comparison:");
    
    for (i, &id) in fusion_ids.iter().enumerate() {
        if let Some(fused) = service.get_fused_embedding(&id) {
            let norm: f32 = fused.fused_vector.iter().map(|x| x * x).sum::<f32>().sqrt();
            println!("   Strategy {}: {:?}, Norm: {:.6}", i + 1, fused.fusion_strategy, norm);
        }
    }
}