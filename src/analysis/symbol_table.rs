use std::collections::HashMap;
use crate::analysis::operator_table::OperatorTable;
use crate::analysis::type_registry::TypeEntry;
use crate::ast::ast_type::AstType;

pub type TypeSymTable = SymbolTable<TypeEntry, bool>;

#[derive(Debug)]
pub struct SymbolTable<T: Clone, E: Clone> {
    pub op_table: OperatorTable,
    pub symbols: Vec<HashMap<String, (T, Option<E>)>>
}

impl <T: Clone, E: Clone> SymbolTable<T, E> {
    
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

    pub fn contains_in_current(&self, symbol: &String) -> bool {
        if let Some(last) = self.last() {
            return last.contains_key(symbol);
        }

        panic!("Symbol table is empty!")
    }

    pub fn get_with_info(&self, symbol: String) -> Option<(T, Option<E>)> {
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
    
    pub fn get(&self, symbol: String) -> Option<T> {
        let t = self.get_with_info(symbol);
        
        if let Some(value) = t {
           return Some(value.0);
        }

        None
    }
    
    pub fn last(&self) -> Option<&HashMap<String, (T, Option<E>)>> {
        self.symbols.last()
    }
    
    pub fn record(&mut self, symbol: String, t: T) {
        self.symbols.last_mut().unwrap().insert(symbol, (t, None));
    }

    pub fn record_with_info(&mut self, symbol: String, t: T, info: E) {
        self.symbols.last_mut().unwrap().insert(symbol, (t, Some(info)));
    }
    
}

impl TypeSymTable {

    pub fn record_return_type(&mut self, t: TypeEntry) {
        self.record("::return".to_string(), t);
    }

    pub fn get_return_type(&self) -> Option<TypeEntry> {
        self.get("::return".to_string())
    }

}