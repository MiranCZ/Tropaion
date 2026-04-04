use crate::error::parser_error::ParserError;
use crate::error::parser_error::ParserError::ClashingModifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Modifier {
    public: Option<bool>
}

impl Modifier {

    pub fn new() -> Modifier {
        Modifier {public: None}
    }

    pub fn public(mut self) -> Result<Modifier, ParserError> {
        if self.has_visibility() {
            return Err(ClashingModifier);
        }

        self.public = Some(true);
        Ok(self)
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

    pub fn has_visibility(&self) -> bool {
        self.public.is_some()
    }

}