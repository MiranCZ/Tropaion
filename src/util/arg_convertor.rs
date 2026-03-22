use crate::ast::expression::int;
use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::value::Value;
use crate::interpreter::value::Value::{FloatValue, IntValue, RefValue};
use crate::util::arg_convertor::ValueConvertable::{FloatValueConv, IntValueConv, VecValueConv};

pub enum ValueConvertable {
    IntValueConv(i32),
    FloatValueConv(f32),
    VecValueConv(Vec<ValueConvertable>)
}

impl ValueConvertable {

    pub fn get_mangled(&self) -> String {
        match self {
            IntValueConv(_) => "i".to_string(),
            FloatValueConv(_) => "f".to_string(),
            VecValueConv(_) => "LVec;".to_string(),
        }
    }

    pub fn into_value(self, interpreter: &mut Interpreter) -> Vec<Value> {
        match self {
            IntValueConv(i) => vec![IntValue(i)],
            FloatValueConv(f) => vec![FloatValue(f)],
            VecValueConv(values) => {
                // vector is ptr -> (capacity, len, arr_ptr)
                let ptr = interpreter.stack_top();
                let capacity = values.len();
                let length = values.len();

                let arr_ptr = unsafe {interpreter.get_heap().alloc(values.len() as u32)};

                let ptr = RefValue {ptr, len: 3};
                let arr_ptr_value = RefValue {ptr: arr_ptr, len: capacity as u32};

                let mut result = vec![IntValue(capacity as i32), IntValue(length as i32), arr_ptr_value, ptr];

                let mut offset = 0;
                for x in values {
                    let value = x.into_value(interpreter);

                    for v in value {
                        unsafe {interpreter.get_heap().store(arr_ptr, offset, v)}
                        offset += 1;
                    }
                }

                result
            }
        }
    }

}

trait ValueLike {
    fn into_convertable(self) -> ValueConvertable;
}

impl ValueLike for i32 {
    fn into_convertable(self) -> ValueConvertable {
        IntValueConv(self)
    }
}
impl ValueLike for bool {
    fn into_convertable(self) -> ValueConvertable {
        if self {
            IntValueConv(1)
        } else {
            IntValueConv(0)
        }
    }
}
impl ValueLike for f32 {
    fn into_convertable(self) -> ValueConvertable {
        FloatValueConv(self)
    }
}

impl <T: ValueLike> ValueLike for Vec<T> {
    fn into_convertable(self) -> ValueConvertable {
        let mut values = vec![];
        for v in self {
            values.push(v.into_convertable())
        }

        VecValueConv(values)
    }
}

pub fn into_arg<T: ValueLike>(value: T) -> ValueConvertable {
    value.into_convertable()
}