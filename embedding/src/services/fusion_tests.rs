#[cfg(test)]
mod tests {
    use super::fusion::*;
    use std::collections::HashMap;

    fn create_test_embedding(modality: EmbeddingModality, vector: Vec<f32>, model: &str) -> Embedding {
        Embedding::new(modality, vector, model.to_string()).unwrap()
    }

    #[test]
    fn test_embedding_creation() {
        let vector = vec![1.0, 2.0, 3.0];
        let embedding = create_test_embedding(EmbeddingModality::Text, vector.clone(), "test-model");
        
        assert_eq!(embedding.vector, vector);
        assert_eq!(embedding.dimension, 3);
        assert_eq!(embedding.modality, EmbeddingModality::Text);
        assert_eq!(embedding.model_name, "test-model");
    }

    #[test]
    fn test_embedding_normalization() {
        let vector = vec![3.0, 4.0]; // Length = 5.0
        let mut embedding = create_test_embedding(EmbeddingModality::Text, vector, "test-model");
        
        embedding.normalize();
        
        let expected = vec![0.6, 0.8];
        assert!((embedding.vector[0] - expected[0]).abs() < 1e-6);
        assert!((embedding.vector[1] - expected[1]).abs() < 1e-6);
    }

    #[test]
    fn test_concatenation_fusion() {
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![1.0, 2.0], "model1");
        let emb2 = create_test_embedding(EmbeddingModality::Image, vec![3.0, 4.0], "model2");
        
        let fused = FusedEmbedding::new(
            vec![emb1, emb2],
            FusionStrategy::Concatenation,
        ).unwrap();
        
        assert_eq!(fused.fused_vector, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(fused.dimension, 4);
    }

    #[test]
    fn test_sum_fusion() {
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![1.0, 2.0], "model1");
        let emb2 = create_test_embedding(EmbeddingModality::Text, vec![3.0, 4.0], "model2");
        
        let fused = FusedEmbedding::new(
            vec![emb1, emb2],
            FusionStrategy::Sum,
        ).unwrap();
        
        assert_eq!(fused.fused_vector, vec![4.0, 6.0]);
        assert_eq!(fused.dimension, 2);
    }

    #[test]
    fn test_weighted_sum_fusion() {
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![1.0, 2.0], "model1");
        let emb2 = create_test_embedding(EmbeddingModality::Text, vec![3.0, 4.0], "model2");
        
        let weights = vec![0.3, 0.7];
        let fused = FusedEmbedding::new(
            vec![emb1, emb2],
            FusionStrategy::WeightedSum(weights),
        ).unwrap();
        
        let expected = vec![0.3 * 1.0 + 0.7 * 3.0, 0.3 * 2.0 + 0.7 * 4.0];
        assert_eq!(fused.fused_vector, expected);
    }

    #[test]
    fn test_average_fusion() {
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![2.0, 4.0], "model1");
        let emb2 = create_test_embedding(EmbeddingModality::Text, vec![4.0, 6.0], "model2");
        
        let fused = FusedEmbedding::new(
            vec![emb1, emb2],
            FusionStrategy::Average,
        ).unwrap();
        
        assert_eq!(fused.fused_vector, vec![3.0, 5.0]);
    }

    #[test]
    fn test_max_fusion() {
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![1.0, 5.0], "model1");
        let emb2 = create_test_embedding(EmbeddingModality::Text, vec![3.0, 2.0], "model2");
        
        let fused = FusedEmbedding::new(
            vec![emb1, emb2],
            FusionStrategy::Max,
        ).unwrap();
        
        assert_eq!(fused.fused_vector, vec![3.0, 5.0]);
    }

    #[test]
    fn test_attention_weighted_fusion() {
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![1.0, 0.0], "model1"); // norm = 1.0
        let emb2 = create_test_embedding(EmbeddingModality::Text, vec![3.0, 4.0], "model2"); // norm = 5.0
        
        let fused = FusedEmbedding::new(
            vec![emb1, emb2],
            FusionStrategy::AttentionWeighted,
        ).unwrap();
        
        // Weights should be 1/6 and 5/6 based on norms
        let expected_x = (1.0/6.0) * 1.0 + (5.0/6.0) * 3.0;
        let expected_y = (1.0/6.0) * 0.0 + (5.0/6.0) * 4.0;
        
        assert!((fused.fused_vector[0] - expected_x).abs() < 1e-6);
        assert!((fused.fused_vector[1] - expected_y).abs() < 1e-6);
    }

    #[test]
    fn test_dimension_mismatch_error() {
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![1.0, 2.0], "model1");
        let emb2 = create_test_embedding(EmbeddingModality::Text, vec![3.0, 4.0, 5.0], "model2");
        
        let result = FusedEmbedding::new(
            vec![emb1, emb2],
            FusionStrategy::Sum,
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_embedding_error() {
        let result = Embedding::new(EmbeddingModality::Text, vec![], "test-model".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_fusion_service() {
        let mut service = FusionEmbeddingService::new();
        
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![1.0, 2.0], "model1");
        let emb2 = create_test_embedding(EmbeddingModality::Text, vec![3.0, 4.0], "model2");
        
        let id1 = service.add_embedding(emb1);
        let id2 = service.add_embedding(emb2);
        
        let fused_id = service.create_fused_embedding(
            vec![id1, id2],
            FusionStrategy::Sum,
        ).unwrap();
        
        let fused = service.get_fused_embedding(&fused_id).unwrap();
        assert_eq!(fused.fused_vector, vec![4.0, 6.0]);
    }

    #[test]
    fn test_reference_models() {
        let text = "Hello, world!";
        
        let bge_emb = ReferenceModels::bge_embedding(text).unwrap();
        assert_eq!(bge_emb.dimension, 768);
        assert_eq!(bge_emb.model_name, "bge-base-en-v1.5");
        assert_eq!(bge_emb.modality, EmbeddingModality::Text);
        
        let qwen_emb = ReferenceModels::qwen3_embedding(text).unwrap();
        assert_eq!(qwen_emb.dimension, 4096);
        assert_eq!(qwen_emb.model_name, "qwen3-8b");
        
        let e5_emb = ReferenceModels::e5_embedding(text).unwrap();
        assert_eq!(e5_emb.dimension, 768);
        assert_eq!(e5_emb.model_name, "e5-base-v2");
    }

    #[test]
    fn test_reference_model_consistency() {
        let text = "Consistent test text";
        
        let emb1 = ReferenceModels::bge_embedding(text).unwrap();
        let emb2 = ReferenceModels::bge_embedding(text).unwrap();
        
        // Same text should produce same embedding
        assert_eq!(emb1.vector, emb2.vector);
    }

    #[test]
    fn test_embedding_normalization_property() {
        let text = "Test normalization";
        let emb = ReferenceModels::bge_embedding(text).unwrap();
        
        // Check if embedding is normalized (L2 norm ≈ 1.0)
        let norm: f32 = emb.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_multimodal_fusion() {
        let text_emb = create_test_embedding(EmbeddingModality::Text, vec![1.0, 2.0, 3.0], "text-model");
        let image_emb = create_test_embedding(EmbeddingModality::Image, vec![4.0, 5.0, 6.0], "image-model");
        let audio_emb = create_test_embedding(EmbeddingModality::Audio, vec![7.0, 8.0, 9.0], "audio-model");
        
        let fused = FusedEmbedding::new(
            vec![text_emb, image_emb, audio_emb],
            FusionStrategy::Average,
        ).unwrap();
        
        assert_eq!(fused.fused_vector, vec![4.0, 5.0, 6.0]); // (1+4+7)/3, (2+5+8)/3, (3+6+9)/3
        assert_eq!(fused.individual_embeddings.len(), 3);
    }

    #[test]
    fn test_batch_fusion() {
        let mut service = FusionEmbeddingService::new();
        
        // Create multiple embeddings
        let embeddings: Vec<_> = (0..6)
            .map(|i| {
                let vector = vec![i as f32, (i + 1) as f32];
                let emb = create_test_embedding(EmbeddingModality::Text, vector, &format!("model{}", i));
                service.add_embedding(emb)
            })
            .collect();
        
        // Create batch requests
        let batch_requests = vec![
            (vec![embeddings[0], embeddings[1]], FusionStrategy::Sum),
            (vec![embeddings[2], embeddings[3]], FusionStrategy::Average),
            (vec![embeddings[4], embeddings[5]], FusionStrategy::Max),
        ];
        
        let fused_ids = service.batch_fuse_embeddings(batch_requests).unwrap();
        assert_eq!(fused_ids.len(), 3);
        
        // Verify results
        let fused1 = service.get_fused_embedding(&fused_ids[0]).unwrap();
        assert_eq!(fused1.fused_vector, vec![2.0, 4.0]); // (0+2, 1+3)
        
        let fused2 = service.get_fused_embedding(&fused_ids[1]).unwrap();
        assert_eq!(fused2.fused_vector, vec![2.5, 3.5]); // ((2+3)/2, (3+4)/2)
        
        let fused3 = service.get_fused_embedding(&fused_ids[2]).unwrap();
        assert_eq!(fused3.fused_vector, vec![5.0, 6.0]); // max(4,5), max(5,6)
    }

    #[test]
    fn test_metadata_handling() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        metadata.insert("timestamp".to_string(), "2024-01-01".to_string());
        
        let embedding = create_test_embedding(EmbeddingModality::Text, vec![1.0, 2.0], "test-model")
            .with_metadata(metadata.clone());
        
        assert_eq!(embedding.metadata, metadata);
        
        let fused = FusedEmbedding::new(
            vec![embedding],
            FusionStrategy::Sum,
        ).unwrap().with_metadata(metadata.clone());
        
        assert_eq!(fused.metadata, metadata);
    }

    #[test]
    fn test_file_io_operations() {
        use std::fs;
        use tempfile::tempdir;
        
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_embeddings.bin");
        
        let mut service = FusionEmbeddingService::new();
        
        // Create test embeddings
        let emb1 = create_test_embedding(EmbeddingModality::Text, vec![1.0, 2.0], "model1");
        let emb2 = create_test_embedding(EmbeddingModality::Image, vec![3.0, 4.0], "model2");
        
        let id1 = service.add_embedding(emb1);
        let id2 = service.add_embedding(emb2);
        
        // Save to file
        service.save_embeddings_to_file(&file_path, &[id1, id2]).unwrap();
        assert!(file_path.exists());
        
        // Load from file into new service
        let mut new_service = FusionEmbeddingService::new();
        let loaded_ids = new_service.load_embeddings_from_file(&file_path).unwrap();
        
        assert_eq!(loaded_ids.len(), 2);
        
        // Verify loaded embeddings
        let loaded_emb1 = new_service.get_embedding(&loaded_ids[0]).unwrap();
        let loaded_emb2 = new_service.get_embedding(&loaded_ids[1]).unwrap();
        
        assert_eq!(loaded_emb1.vector, vec![1.0, 2.0]);
        assert_eq!(loaded_emb2.vector, vec![3.0, 4.0]);
    }
}