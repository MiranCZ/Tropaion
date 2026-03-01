use std::cmp::Ordering;
use std::ops::{Add, BitAnd, BitOr, Div, Mul, Rem, Sub};
use crate::interpreter::value::Value::*;
use crate::interpreter::value::ValueType::*;

macro_rules! impl_math_op {
    ($trait:ident, $method:ident) => {
        impl $trait for Value {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                match self {
                    Value::IntValue(v) => Value::IntValue(v.$method(rhs.try_as_int())),
                    Value::FloatValue(v) => Value::FloatValue(v.$method(rhs.try_as_float())),
                    Value::RefValue{..} => panic!("Cannot apply {} to references", stringify!($method)),
                    Value::Null => panic!("Cannot write operations with null!")
                }
            }
        }
    };
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Value {
    Null,
    IntValue(i32),
    FloatValue(f32),
    RefValue{
        ptr: u32,
        len: u32 
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ValueType {
    Null,
    Int,
    Float,
    Address
}

impl ValueType {

    pub fn assignable(&self, value: &Value) -> bool {
        match (self, value) {
            (ValueType::Null, Value::Null) |
            (Int, IntValue(_)) |
            (Float, FloatValue(_)) |
            (Address, RefValue{..}) |
            (Address, Value::Null) => true,

            _ => false
        }
    }

}

impl Value {

    pub fn try_as_int(&self) -> i32 {
        if let IntValue(v) = self {
            return *v;
        }

        panic!("Attempting to get {self:?} as int!");
    }

    pub fn try_as_float(&self) -> f32 {
        if let FloatValue(v) = self {
            return *v;
        }

        panic!("Attempting to get {self:?} as float!");
    }

}

impl_math_op!(Add, add);
impl_math_op!(Sub, sub);
impl_math_op!(Mul, mul);
impl_math_op!(Div, div);
impl_math_op!(Rem, rem);

impl BitOr for Value {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match self {
            IntValue(v) => IntValue(v | rhs.try_as_int()),

            _ => panic!("Invalid BitOr for {self:?} {rhs:?}")
        }
    }
}

impl BitAnd for Value {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match self {
            IntValue(v) => IntValue(v & rhs.try_as_int()),

            _ => panic!("Invalid BitAnd for {self:?} {rhs:?}")
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            IntValue(v) => v.partial_cmp(&other.try_as_int()),
            FloatValue(v) => v.partial_cmp(&other.try_as_float()),

            RefValue{..} => panic!("Cannot compare references"),
            Value::Null => panic!("Cannot make operations with null!")
        }
    }
}

