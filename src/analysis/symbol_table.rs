use std::collections::HashMap;
use crate::ast::ast_type::AstType;

#[derive(Debug)]
pub struct SymbolTable {
    symbols: Vec<HashMap<String, AstType>>
}

impl SymbolTable {
    
    pub fn new() -> Self {
        let mut new = Self {
            symbols: vec![]
        };
        new.push();
        
        new
    }
    
    pub fn push(&mut self) {
        self.symbols.push(HashMap::new());
    }
    
    pub fn pop(&mut self) {
        self.symbols.pop();
    }
    
    pub fn get_type(&self, symbol: String) -> Option<AstType> {
        let mut t = None;
        
        for map in self.symbols.iter() {
            if let Some(result) = map.get(&symbol) {
                // IMPORTANT: do not break here, we should also check lower symbols
                // TODO: A future optimization is searching in a reverse order so we can break early
                t = Some(result.clone());
            }
        }
        
        t
    }
    
    pub fn record_type(&mut self, symbol: String, t: AstType) {
        self.symbols.last_mut().unwrap().insert(symbol, t);
    }
    
}