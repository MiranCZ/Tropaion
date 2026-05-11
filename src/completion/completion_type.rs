#[derive(Debug, Copy, Clone)]
pub enum CompletionType {
    Variable,
    Constant,
    Function,
    Struct,
    KwDeclaration,
    KwControl,
    KwReturn,
    KwDefinition,
    KwVisibility
}

impl CompletionType {

    pub fn as_string(&self) -> &str {
        match self {
            CompletionType::Variable => "variable",
            CompletionType::Constant => "constant",
            CompletionType::Function => "function",
            CompletionType::Struct => "struct",
            CompletionType::KwDeclaration => "kw_declaration",
            CompletionType::KwControl => "kw_control",
            CompletionType::KwReturn => "kw_return",
            CompletionType::KwDefinition => "kw_definition",
            CompletionType::KwVisibility => "kw_visibility"
        }
    }

}