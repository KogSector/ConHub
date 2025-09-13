use crate::types::*;
use crate::types::*;
use std::collections::HashMap;
use uuid::Uuid;

pub struct CrossReferenceEngine {
    symbol_definitions: HashMap<String, Vec<Symbol>>,
    symbol_references: HashMap<Uuid, Vec<Reference>>,
}

impl CrossReferenceEngine {
    pub fn new() -> Self {
        Self {
            symbol_definitions: HashMap::new(),
            symbol_references: HashMap::new(),
        }
    }

    pub fn add_symbols(&mut self, symbols: Vec<Symbol>) {
        for symbol in symbols {
            self.symbol_definitions
                .entry(symbol.name.clone())
                .or_insert_with(Vec::new)
                .push(symbol);
        }
    }

    pub fn add_references(&mut self, references: Vec<Reference>) {
        for reference in references {
            self.symbol_references
                .entry(reference.symbol_id)
                .or_insert_with(Vec::new)
                .push(reference);
        }
    }

    pub fn get_cross_reference(&self, symbol_name: &str) -> Option<CrossReference> {
        if let Some(symbols) = self.symbol_definitions.get(symbol_name) {
            let symbol = symbols.first()?.clone();
            
            let definitions = symbols.iter()
                .filter(|s| matches!(s.symbol_type, SymbolType::Function | SymbolType::Class))
                .map(|s| Reference {
                    id: Uuid::new_v4(),
                    symbol_id: s.id,
                    file_id: s.file_id,
                    line: s.line,
                    column: s.column,
                    reference_type: ReferenceType::Definition,
                    context: String::new(),
                })
                .collect();

            let usages = self.symbol_references
                .get(&symbol.id)
                .cloned()
                .unwrap_or_default();

            Some(CrossReference {
                symbol,
                definitions,
                declarations: Vec::new(),
                usages,
                calls: Vec::new(),
            })
        } else {
            None
        }
    }
}