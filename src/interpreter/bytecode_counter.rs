use crate::compiler::bytecode::ByteCode;
use crate::interpreter::interpreter::Interpreter;

pub fn get_count(insn: &ByteCode, interpreter: &Interpreter) -> u32 {
    match insn {
        ByteCode::Nop |
        ByteCode::NullPtr |
        ByteCode::Panic |
        ByteCode::IConst(_) |
        ByteCode::FConst(_) |
        ByteCode::Pop |
        ByteCode::Dup |
        ByteCode::Swap |
        ByteCode::I2F |
        ByteCode::F2I => 1,
        
        ByteCode::StrConst(_) => 2,
        
        ByteCode::I2Str => 2,
        ByteCode::F2Str => 2,
        
        ByteCode::Or |
        ByteCode::And |
        ByteCode::Xor |
        ByteCode::BitNot |
        ByteCode::Shl |
        ByteCode::Shr |
        ByteCode::BoolNot => 2,
        
        
        ByteCode::Add |
        ByteCode::Sub => 5,
        
        ByteCode::Mul => 8,
        
        ByteCode::Div |
        ByteCode::Mod => 10,
        
        ByteCode::StrConcat => 2,
        
        ByteCode::Print => 2,
        
        ByteCode::CmpEq |
        ByteCode::CmpNotEq |
        ByteCode::CmpGreater |
        ByteCode::CmpEqGreater |
        ByteCode::CmpLess |
        ByteCode::CmpEqLess => 5,
        
        ByteCode::Store(_) => 20,
        ByteCode::Load(_) => 20,
        
        ByteCode::CreateStackPtr { .. } => 20,
        ByteCode::ILoadOffset(_) |
        ByteCode::FLoadOffset(_) |
        ByteCode::ALoadOffset(_) |
        ByteCode::IStoreOffset(_) |
        ByteCode::FStoreOffset(_) |
        ByteCode::AStoreOffset(_) => 25,
        
        ByteCode::LoadVarOffset |
        ByteCode::StoreVarOffset => 30,
        
        ByteCode::Goto(_) => 10,
        
        ByteCode::IfEq(_) |
        ByteCode::IfNe(_) => 25,
        
        ByteCode::StackFrame(size) => 8 + (*size as u32),
        
        
        ByteCode::Call(_) => 20,
        
        ByteCode::Ret(size) => 20 + (*size as u32),
        
        ByteCode::RetLong(size) => 20 + size,
        
        ByteCode::HeapAlloc(size) => 50 + size*2,
        
        // FIXME: should get value on top of stack and calculate size
        ByteCode::DynHeapAlloc => 80,
        
        _ => 0
    }
    
    
    
}