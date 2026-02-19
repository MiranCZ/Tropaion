use crate::ast::ast_type::AstType;
use crate::lexer::token::SimpleToken;
use crate::lexer::token::SimpleToken::{Ampersand, VertBarAssign, BitXor, BoolAnd, BoolOr, Dash, DashAssign, Equals, Greater, GreaterEquals, LeftLeft, LeftLeftAssign, Less, LessEquals, NotEquals, Percent, PercentAssign, Plus, PlusAssign, RightRight, RightRightAssign, Slash, SlashAssign, Star, StarAssign, VerticalBar, AmpersandAssign, BitXorAssign, Assign};
use std::collections::HashMap;

type SimpleType = usize;

const Bool: SimpleType = 0;
const Int: SimpleType = 1;
const Float: SimpleType = 2;

#[derive(Debug)]
pub struct OperatorTable {
    table: HashMap<(SimpleType, SimpleToken, SimpleType), SimpleType>,
}

impl OperatorTable {
    pub fn new() -> Self {
        let mut new = Self {
            table: HashMap::new(),
        };
        new.init();

        new
    }

    pub fn get_op_result(&self, right: AstType, op: SimpleToken, left: AstType) -> Option<AstType> {
        let right = from_ast_type(right);
        let left = from_ast_type(left);

        let result = self.table.get(&(right, op, left));

        if let Some(res) = result {
            return Some(to_ast_type(*res));
        }

        None
    }

    fn init(&mut self) {
        for op in [
            Plus,
            Dash,
            Star,
            Slash,
            Percent,
            PlusAssign,
            DashAssign,
            StarAssign,
            SlashAssign,
            PercentAssign,
        ] {
            self.add(Int, op, Int, Int);
            self.add(Float, op, Float, Float);
        }

        // bit ops should probably be disallowed for float?
        // TODO specify this
        for bit_op in [
            RightRight,
            LeftLeft,
            Ampersand,
            VerticalBar,
            BitXor,
            RightRightAssign,
            LeftLeftAssign,
            AmpersandAssign,
            VertBarAssign,
            BitXorAssign
        ] {
            self.add(Int, bit_op, Int, Int);
        }

        // TODO should comparisons int-float be allowed?
        for comp in [Equals, NotEquals, Less, LessEquals, Greater, GreaterEquals] {
            self.add(Int, comp, Int, Bool);
            self.add(Float, comp, Float, Bool);
        }

        for bool_op in [BoolOr, BoolAnd] {
            self.add(Bool, bool_op, Bool, Bool);
        }

        for t in [Bool, Int, Float] {
            self.add(t, Assign, t, t);
        }
    }

    fn add(&mut self, left: SimpleType, op: SimpleToken, right: SimpleType, result: SimpleType) {
        self.table.insert((left, op, right), result);
    }
}

fn from_ast_type(t: AstType) -> SimpleType {
    match t {
        AstType::Bool => Bool,
        AstType::Int => Int,
        AstType::Float => Float,
        _ => panic!("Not a simple type {:?}", t),
    }
}

fn to_ast_type(t: SimpleType) -> AstType {
    if t == Bool {
        return AstType::Bool;
    }
    if t == Int {
        return AstType::Int;
    }
    if t == Float {
        return AstType::Float;
    }

    panic!("Invalid simple type! {t:?}")
}
