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