use crate::lexor::types::*;
use std::collections::HashMap;

pub struct CodeAnalyzer;

impl CodeAnalyzer {
    pub fn analyze_complexity(content: &str) -> ComplexityMetrics {
        let lines = content.lines().count();
        let cyclomatic = Self::calculate_cyclomatic_complexity(content);
        
        ComplexityMetrics {
            lines_of_code: lines,
            cyclomatic_complexity: cyclomatic,
            cognitive_complexity: cyclomatic, // Simplified
            maintainability_index: Self::calculate_maintainability(lines, cyclomatic),
        }
    }

    fn calculate_cyclomatic_complexity(content: &str) -> u32 {
        let mut complexity = 1; // Base complexity
        
        for line in content.lines() {
            let line = line.trim();
            if line.contains("if ") || line.contains("while ") || 
               line.contains("for ") || line.contains("match ") ||
               line.contains("case ") || line.contains("&&") || 
               line.contains("||") {
                complexity += 1;
            }
        }
        
        complexity
    }

    fn calculate_maintainability(lines: usize, complexity: u32) -> f32 {
        let halstead_volume = (lines as f32).ln() * 2.0; // Simplified
        171.0 - 5.2 * halstead_volume.ln() - 0.23 * complexity as f32 - 16.2 * (lines as f32).ln()
    }
}

#[derive(Debug, Clone)]
pub struct ComplexityMetrics {
    pub lines_of_code: usize,
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub maintainability_index: f32,
}