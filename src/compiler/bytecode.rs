
#[derive(Debug, PartialEq, Clone)]
pub enum ByteCode {
    Comment(String),
    Nop,

    NullPtr,
    
    IConst(i32),
    FConst(f32),

    Pop,
    Dup,

    Or,
    And,
    
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    CmpEq,
    CmpNotEq,
    CmpGreater,
    CmpEqGreater,
    CmpLess,
    CmpEqLess,

    IStore(u16),
    FStore(u16),
    AStore(u16),

    ILoad(u16),
    FLoad(u16),
    ALoad(u16),

    CreateStackPtr{
        offset: u16,
        consume_words: u16
    },

    // loads from address on top of the stack + offset
    ILoadOffset(u16),
    FLoadOffset(u16),
    ALoadOffset(u16),

    IStoreOffset(u16),
    FStoreOffset(u16),
    AStoreOffset(u16),

    Goto(i32),
    IfEq(i32),
    IfNe(i32),

    StackFrame(u16),
    Call(u16),
    Ret(u16),
    RetLong(u32)
}
