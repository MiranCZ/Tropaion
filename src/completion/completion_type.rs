#[derive(Debug, Copy, Clone)]
pub enum CompletionType {
    Variable,
    Constant,
    Function,
    Struct,
    Enum,
    Field,
    Method,
    Parameter,
    EnumValue,
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
            CompletionType::Enum => "enum",
            CompletionType::Field => "field",
            CompletionType::Method => "method",
            CompletionType::Parameter => "parameter",
            CompletionType::EnumValue => "enum_value",
            CompletionType::KwDeclaration => "kw_declaration",
            CompletionType::KwControl => "kw_control",
            CompletionType::KwReturn => "kw_return",
            CompletionType::KwDefinition => "kw_definition",
            CompletionType::KwVisibility => "kw_visibility"
        }
    }

}