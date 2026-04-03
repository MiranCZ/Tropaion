use crate::analysis::type_registry::TypeRegistry;
use crate::ast::expression::int;
use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::value::Value;
use crate::interpreter::value::Value::{FloatValue, IntValue, RefValue};
use crate::util::arg_convertor::ValueConvertable::{FloatValueConv, IntValueConv, StructValueConv, TupleValueConv, VecValueConv};

pub enum ValueConvertable {
    IntValueConv(i32),
    FloatValueConv(f32),
    VecValueConv(Vec<ValueConvertable>),
    StructValueConv(String, Vec<ValueConvertable>),
    TupleValueConv(Vec<ValueConvertable>)
}

impl ValueConvertable {

    pub fn get_mangled(&self) -> String {
        match self {
            IntValueConv(_) => "i".to_string(),
            FloatValueConv(_) => "f".to_string(),
            VecValueConv(values) => {
                if values.is_empty() {
                    return "LVec_?;".to_string();
                }
                format!("LVec_{};", values[0].get_mangled())
            },
            StructValueConv(name, _) => format!("L{name}_;"),
            TupleValueConv(types) => {
                let mut name = "T".to_string();
                for t in types {
                    name += t.get_mangled().as_str();
                }

                name + ";"
            }
        }
    }

    pub fn into_value(self, interpreter: &mut Interpreter) -> Vec<Value> {
        match self {
            IntValueConv(i) => vec![IntValue(i)],
            FloatValueConv(f) => vec![FloatValue(f)],
            StructValueConv(_, values) |
            TupleValueConv(values) => {
                let ptr = interpreter.stack_top();

                let rf = RefValue {ptr, len: values.len() as u32};

                for x in values {
                    let value = x.into_value(interpreter);

                    for v in value {
                        // result.push(v);
                        unsafe{interpreter.push_to_stack(v).unwrap();}
                    }
                }

                vec![rf]
            }
            VecValueConv(values) => {
                // vector is ptr -> (capacity, len, arr_ptr)
                let ptr = interpreter.stack_top();

                let capacity = values.len();
                let length = values.len();


                unsafe{interpreter.push_to_stack(IntValue(capacity as i32)).unwrap();}
                unsafe{interpreter.push_to_stack(IntValue(length as i32)).unwrap();}

                let arr_ptr = unsafe {interpreter.get_heap().alloc(values.len() as u32)};

                let ptr = RefValue {ptr, len: 3};
                let arr_ptr_value = RefValue {ptr: arr_ptr, len: capacity as u32};


                unsafe{interpreter.push_to_stack(arr_ptr_value).unwrap();}

                let mut offset = 0;
                for x in values {
                    let value = x.into_value(interpreter);

                    for v in value {
                        unsafe {interpreter.get_heap().store(arr_ptr, offset, v)}
                        offset += 1;
                    }
                }

                vec![ptr]
            }
        }
    }

}

pub trait ValueLike {
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

pub struct StructConvertor {
    name: String,
    values: Vec<ValueConvertable>
}

impl StructConvertor {

    pub fn add_field(&mut self, field: impl ValueLike) {
        self.values.push(field.into_convertable());
    }

    pub fn convert(self) -> ValueConvertable {
        StructValueConv(self.name, self.values)
    }

}

pub fn struct_convertor(name: &str) -> StructConvertor {
    StructConvertor{name: name.to_string(), values: vec![]}
}


// tuple impl... just- no
macro_rules! tuple_impls {
    ( $head:ident, $( $tail:ident, )+ ) => {
        impl<$head, $( $tail ),+> ValueLike for ($head, $( $tail ),+)
        where
            $head: ValueLike,
            $( $tail: ValueLike ),+
        {

            fn into_convertable(self) -> ValueConvertable {
                #[allow(non_snake_case)]
                let ($head, $( $tail ),+) = self;

                let values = vec![$head.into_convertable(), $( $tail.into_convertable() ),+];

                TupleValueConv(values)
            }
        }

        tuple_impls!($( $tail, )+);
    };

    ( $head:ident, ) => {
        impl<$head> ValueLike for ($head,)
        where
            $head: ValueLike,
        {
            fn into_convertable(self) -> ValueConvertable {
                #[allow(non_snake_case)]
                let ($head,) = self;

                let values = vec![$head.into_convertable()];
                TupleValueConv(values)
            }
        }
    };

    () => {};
}

tuple_impls!(A, B, C, D, E, F, G, H, I, J,);
