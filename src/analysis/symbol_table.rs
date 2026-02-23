use std::collections::HashMap;
use crate::analysis::operator_table::OperatorTable;
use crate::ast::ast_type::AstType;

pub type TypeSymTable = SymbolTable<AstType>;

#[derive(Debug)]
pub struct SymbolTable<T: Clone> {
    pub op_table: OperatorTable,
    symbols: Vec<HashMap<String, T>>
}

impl <T: Clone> SymbolTable<T> {
    
    pub fn new() -> Self {
        let mut new = Self {
            op_table: OperatorTable::new(),
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

    pub fn contains(&self, symbol: &String) -> bool {
        for map in self.symbols.iter() {
            if map.contains_key(symbol) {
                return true;
            }
        }

        false
    }

    pub fn get(&self, symbol: String) -> Option<T> {
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
    
    pub fn last(&self) -> Option<&HashMap<String, T>> {
        self.symbols.last()
    }
    
    pub fn record(&mut self, symbol: String, t: T) {
        self.symbols.last_mut().unwrap().insert(symbol, t);
    }
    
}