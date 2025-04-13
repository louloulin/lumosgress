#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use serde_json::json;
    use crate::plugins::prompt_transform::{
        PromptTransformer, TransformationType, PromptTransformConfig, PromptTransformation
    };

    #[test]
    fn test_enhanced_transformations() {
        // Create a transformer with test templates
        let mut transformer = PromptTransformer::new();
        
        // Create test configurations for each new transformation type
        let format_prompt_transformation = PromptTransformation {
            transform_type: TransformationType::FormatPrompt,
            provider: Some("openai".to_string()),
            content: "Format this into bullet points".to_string(),
            template: None,
            target_lang: None,
            format_style: Some("bullet".to_string()),
            max_tokens: None,
            enhancement_level: None,
            rag_source: None,
            custom_params: None,
        };
        
        let extract_keywords_transformation = PromptTransformation {
            transform_type: TransformationType::ExtractKeywords,
            provider: None,
            content: "Focus on these key concepts".to_string(),
            template: None,
            target_lang: None,
            format_style: None,
            max_tokens: None,
            enhancement_level: None,
            rag_source: None,
            custom_params: None,
        };
        
        let enhance_content_transformation = PromptTransformation {
            transform_type: TransformationType::EnhanceContent,
            provider: None,
            content: "Include examples and detailed explanations".to_string(),
            template: None,
            target_lang: None,
            format_style: None,
            max_tokens: None,
            enhancement_level: Some("advanced".to_string()),
            rag_source: None,
            custom_params: None,
        };
        
        let translate_prompt_transformation = PromptTransformation {
            transform_type: TransformationType::TranslatePrompt,
            provider: None,
            content: "".to_string(),
            template: None,
            target_lang: Some("Spanish".to_string()),
            format_style: None,
            max_tokens: None,
            enhancement_level: None,
            rag_source: None,
            custom_params: None,
        };
        
        // Test format_prompt transformation
        let mut json_body = json!({
            "messages": [
                {"role": "user", "content": "Tell me about artificial intelligence. Explain its history. Discuss its applications."}
            ]
        });
        
        let result = transformer.format_prompt(&mut json_body, "bullet", "Bullet points");
        assert!(result);
        let content = json_body["messages"][0]["content"].as_str().unwrap();
        assert!(content.contains("• Tell me about artificial intelligence"));
        assert!(content.contains("• Explain its history"));
        assert!(content.contains("• Discuss its applications"));
        
        // Test extract_keywords transformation
        let mut json_body = json!({
            "messages": [
                {"role": "user", "content": "Explain the principles of machine learning and neural networks applied to computer vision"}
            ]
        });
        
        let result = transformer.extract_keywords(&mut json_body, "Focus on these key concepts");
        assert!(result);
        let content = json_body["messages"][0]["content"].as_str().unwrap();
        assert!(content.contains("Keywords:"));
        assert!(content.contains("machine"));
        assert!(content.contains("learning"));
        assert!(content.contains("neural"));
        assert!(content.contains("networks"));
        assert!(content.contains("computer"));
        assert!(content.contains("vision"));
        
        // Test enhance_content transformation
        let mut json_body = json!({
            "messages": [
                {"role": "user", "content": "Tell me about quantum computing"}
            ]
        });
        
        let result = transformer.enhance_content(&mut json_body, "advanced", "Include specific examples of quantum algorithms");
        assert!(result);
        let content = json_body["messages"][0]["content"].as_str().unwrap();
        assert!(content.contains("expert in this subject"));
        assert!(content.contains("in-depth analysis"));
        assert!(content.contains("Include specific examples of quantum algorithms"));
        assert!(content.contains("multiple viewpoints"));
        
        // Test translate_prompt transformation
        let mut json_body = json!({
            "messages": [
                {"role": "user", "content": "What is the capital of France?"}
            ]
        });
        
        let result = transformer.translate_prompt(&mut json_body, "Spanish");
        assert!(result);
        let content = json_body["messages"][0]["content"].as_str().unwrap();
        assert!(content.contains("Translate the following to Spanish:"));
        assert!(content.contains("What is the capital of France?"));
        
        println!("All enhanced transformation tests passed!");
    }
} 