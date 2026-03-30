
#[derive(Debug, PartialEq, Clone)]
pub enum ByteCode {
    Comment(String),
    Nop,

    NullPtr,
    
    IConst(i32),
    FConst(f32),
    StrConst(String),

    Pop,
    Dup,
    Swap,
    
    I2F,
    F2I,

    Or,
    And,
    Xor,
    BitNot,
    BoolNot,

    Shl,
    Shr,

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

    Store(u16),
    // IStore(u16),
    // FStore(u16),
    // AStore(u16),

    Load(u16),
    // ILoad(u16),
    // FLoad(u16),
    // ALoad(u16),

    CreateStackPtr{
        offset: u16,
        consume_words: u16
    },

    // loads from address on top of the stack + offset
    ILoadOffset(u32),
    FLoadOffset(u32),
    ALoadOffset(u32),

    IStoreOffset(u32),
    FStoreOffset(u32),
    AStoreOffset(u32),

    LoadVarOffset,
    // ILoadVarOffset,
    // FLoadVarOffset,
    // ALoadVarOffset,

    StoreVarOffset,
    // IStoreVarOffset,
    // FStoreVarOffset,
    // AStoreVarOffset,


    Goto(i32),
    IfEq(i32),
    IfNe(i32),

    StackFrame(u16),
    Call(u16),
    Ret(u16),
    RetLong(u32),
    
    HeapAlloc(u32),
    DynHeapAlloc
}
