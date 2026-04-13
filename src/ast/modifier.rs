use crate::error::parser_error::ParserError;
use crate::error::parser_error::ParserError::ClashingModifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Modifier {
    public: Option<bool>,
    is_static: Option<bool>
}

impl Modifier {

    pub fn new() -> Modifier {
        Modifier {public: None, is_static: None}
    }

    pub fn with_public(&self) -> Modifier {
        let mut new = *self;
        
        new.public = Some(true);
        new
    }
    
    pub fn with_static(&self) -> Modifier {
        let mut new = *self;
        
        new.is_static = Some(true);
        new
    }

    pub fn is_public(&self) -> bool {
        if let Some(p) = self.public {
            return p;
        }

        false
    }

    pub fn is_private(&self) -> bool {
        if let Some(p) = self.public {
            return p;
        }

        true
    }
    
    pub fn is_static(&self) -> bool {
        if let Some(s) = self.is_static {
            return s;
        }
        false
    }

    pub fn has_visibility(&self) -> bool {
        self.public.is_some()
    }

}