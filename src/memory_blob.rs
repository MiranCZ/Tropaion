use crate::error::ok;
use crate::error::runtime_error::{RuntimeError, ValueTypeVariant};
use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::value::Value;
use crate::interpreter::value::Value::RefValue;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MemoryBlob {
    mem: Vec<Value>,
    ptr: usize
}

pub fn resolve_blob(value: Value, interpreter: &Interpreter) -> MemoryBlob {
    let mut mem = vec![];

    resolve_value(value, &mut mem, interpreter, &mut HashMap::new());

    MemoryBlob{
        mem,
        ptr: 0
    }
}

fn resolve_value(value: Value, mem: &mut Vec<Value>, interpreter: &Interpreter, address_map: &mut HashMap<u32, u32>) {
    if let RefValue {ptr, len} = value {
        if let Some(mapped_ptr) = address_map.get(&ptr) {
            mem.push(RefValue {ptr: *mapped_ptr, len})
        } else {
            let mapped = mem.len() as u32;
            address_map.insert(ptr, mapped);
            mem.push(RefValue {ptr: mapped, len});

            for i in ptr..(ptr+len) {
                let v = unsafe{interpreter.load_at_ptr(i as usize)};
                resolve_value(v, mem, interpreter, address_map);
            }
        }
    } else {
        mem.push(value)
    }
}


impl MemoryBlob {

    pub fn next_int(&mut self) -> Result<i32, RuntimeError> {
        let value = self.next()?;
        if let Value::IntValue(i) = value {
            Ok(*i)
        } else {
            Err(RuntimeError::TypeMismatch {expected: ValueTypeVariant::Int, got: *value })
        }
    }

    pub fn next_float(&mut self) -> Result<f32, RuntimeError> {
        let value = self.next()?;
        if let Value::FloatValue(f) = value {
            Ok(*f)
        } else {
            Err(RuntimeError::TypeMismatch {expected: ValueTypeVariant::Float, got: *value })
        }
    }

    pub fn expect_end(&self) -> Result<(), RuntimeError> {
        if self.ptr < self.mem.len() {
            return Err(RuntimeError::DanglingValue);
        }
        ok()
    }

    fn next(&mut self) -> Result<&Value, RuntimeError> {
        let v = self.mem.get(self.ptr);
        self.ptr += 1;

        if let Some(value) = v {
            return Ok(value);
        }

        return Err(RuntimeError::StackUnderflow(""))
    }

}