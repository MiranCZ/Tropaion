use crate::ast::ast_type::AstType;
use crate::lexer::token::SimpleToken;
use crate::lexer::token::SimpleToken::{Ampersand, VertBarAssign, BitXor, BoolAnd, BoolOr, Dash, DashAssign, Equals, Greater, GreaterEquals, LeftLeft, LeftLeftAssign, Less, LessEquals, NotEquals, Percent, PercentAssign, Plus, PlusAssign, RightRight, RightRightAssign, Slash, SlashAssign, Star, StarAssign, VerticalBar, AmpersandAssign, BitXorAssign, Assign};
use std::collections::HashMap;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType::NullableType;
use crate::error::analysis_error::AnalysisError;

type SimpleType = usize;

const BOOL: SimpleType = 0;
const INT: SimpleType = 1;
const FLOAT: SimpleType = 2;
const STRING_T: SimpleType = 3;

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

    pub fn get_op_result(&self,registry: &TypeRegistry ,left_type: TypeEntry, op: SimpleToken, right_type: TypeEntry) -> Result<AstType, AnalysisError> {
        if right_type.get(registry).loose_equals(&left_type.get(registry), registry) {
            if let Equals = op {
                return Ok(AstType::Bool);
            }
            if let NotEquals = op {
                return Ok(AstType::Bool);
            }
        }

        if matches!(op, SimpleToken::TwoQuestion) && let NullableType{underlying} = left_type.get(registry) {
            if matches!(right_type.get(registry), NullableType {..}) {
                return Err(AnalysisError::illegal_binary_expression(left_type, op, right_type, registry));
            }

            if underlying.get(registry).loose_equals(&right_type.get(registry), registry) {
                return Ok(underlying.get(registry));
            }
        }

        let right = from_ast_type(right_type.get(registry), registry);
        let left = from_ast_type(left_type.get(registry), registry);

        let (right, left) = if let Some(r) = right && let Some(l) = left {
            (r, l)
        } else {
            return Err(AnalysisError::illegal_binary_expression(left_type, op, right_type, registry));
        };

        let result = self.table.get(&(right, op, left));

        if let Some(res) = result && let Some(ast_type) = to_ast_type(*res) {
            return Ok(ast_type);
        }

        Err(AnalysisError::illegal_binary_expression(left_type, op, right_type, registry))
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
            self.add(INT, op, INT, INT);
            self.add(FLOAT, op, FLOAT, FLOAT);
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
            self.add(INT, bit_op, INT, INT);
        }

        // TODO should comparisons int-float be allowed?
        for comp in [Equals, NotEquals, Less, LessEquals, Greater, GreaterEquals] {
            self.add(INT, comp, INT, BOOL);
            self.add(FLOAT, comp, FLOAT, BOOL);
        }

        for bool_op in [BoolOr, BoolAnd] {
            self.add(BOOL, bool_op, BOOL, BOOL);
        }

        for t in [BOOL, INT, FLOAT] {
            self.add(t, Assign, t, t);
        }

        self.add(STRING_T, Plus, STRING_T, STRING_T);
    }

    fn add(&mut self, left: SimpleType, op: SimpleToken, right: SimpleType, result: SimpleType) {
        self.table.insert((left, op, right), result);
    }
}

fn from_ast_type(t: AstType, registry: &TypeRegistry) -> Option<SimpleType> {
    match t {
        AstType::Bool => Some(BOOL),
        AstType::Int => Some(INT),
        AstType::Float => Some(FLOAT),
        AstType::StringType => Some(STRING_T),
        _ => None,
    }
}

fn to_ast_type(t: SimpleType) -> Option<AstType> {
    if t == BOOL {
        return Some(AstType::Bool);
    }
    if t == INT {
        return Some(AstType::Int);
    }
    if t == FLOAT {
        return Some(AstType::Float);
    }
    if t == STRING_T {
        return Some(AstType::StringType);
    }

    None
}
