use std::collections::HashMap;
use crate::ast::statement::Statement::StructStmt;
use crate::lexer::token::SimpleToken;
use crate::lexer::token::SimpleToken::*;
use crate::parser::binding_power::{BindingPower, Bp, ASSIGNMENT, CALL, COMMA, COMPARING, LOGICAL_ADD, LOGICAL_MULT, MEMBER, NULL_DECONSTRUCT, NUMERIC_ADD, NUMERIC_MULT, UNARY};
use crate::parser::expression_parser::{parse_array_access_expr, parse_array_expr, parse_assignment_expr, parse_binary_expr, parse_bool_literal_expr, parse_call_expr, parse_decrement_expr, parse_increment_expr, parse_member_expr, parse_null_deref, parse_null_expr, parse_parenthesis_expr, parse_prefix_expr};
use crate::parser::handlers::{LedHandler, LedInfo, NudHandler, StatementHandler};
use crate::parser::statement_parser::{parse_block_stmt, parse_break_statement, parse_continue_statement, parse_enum_statement, parse_fn_declaration_stmt, parse_if_statement, parse_return_stmt, parse_struct_statement, parse_var_declaration_stmnt, parse_while_statement};

type NudLookup = HashMap<SimpleToken, NudHandler>;
type LedLookup = HashMap<SimpleToken, LedInfo>;
type StatementLookup = HashMap<SimpleToken, StatementHandler>;


pub struct Lookup {
    pub nud_lookup: NudLookup,
    pub led_lookup: LedLookup,
    pub statement_lookup: StatementLookup
}


impl Lookup {

    pub fn new() -> Self {
        let (nud_lookup, led_lookup, statement_lookup) = Self::init_lookups();

        Self {
            nud_lookup,
            led_lookup,
            statement_lookup
        }
    }

    fn init_lookups() -> (NudLookup, LedLookup, StatementLookup) {
        let mut nud_lookup = HashMap::new();
        let mut led_lookup = HashMap::new();
        let mut statement_lookup = HashMap::new();

        let mut nud = |token: SimpleToken, handler: NudHandler| {
            nud_lookup.insert(token, handler);
        };

        let mut led = |token: SimpleToken, bp: BindingPower, handler: LedHandler| {
            led_lookup.insert(token, LedInfo{handler, lbp: bp.lbp, rbp: bp.rbp});
        };

        let mut statement = |token: SimpleToken, handler: StatementHandler| {
            statement_lookup.insert(token, handler);
        };

        nud(Dash, parse_prefix_expr);
        nud(Tilde, parse_prefix_expr);
        nud(Exclamation, parse_prefix_expr);

        nud(True, parse_bool_literal_expr);
        nud(False, parse_bool_literal_expr);

        nud(OpenBracket, parse_parenthesis_expr);
        nud(Null, parse_null_expr);

        nud(OpenSquare, parse_array_expr);
       
        
        led(Dash, NUMERIC_ADD, parse_binary_expr);
        led(Plus, NUMERIC_ADD, parse_binary_expr);

        led(Star, NUMERIC_MULT, parse_binary_expr);
        led(Slash, NUMERIC_MULT, parse_binary_expr);
        led(Percent, NUMERIC_MULT, parse_binary_expr);

        led(VerticalBar, LOGICAL_ADD, parse_binary_expr);
        led(Ampersand, LOGICAL_MULT, parse_binary_expr);

        led(LeftLeft, LOGICAL_MULT, parse_binary_expr);
        led(RightRight, LOGICAL_MULT, parse_binary_expr);

        led(BoolOr, LOGICAL_ADD, parse_binary_expr);
        led(BoolAnd, LOGICAL_MULT, parse_binary_expr);
        led(BitXor, LOGICAL_MULT, parse_binary_expr);

        led(Dot, MEMBER, parse_member_expr);
        led(QuestionDot, MEMBER, parse_member_expr);

        led(OpenBracket, CALL, parse_call_expr);

        led(Equals, COMPARING, parse_binary_expr);
        led(NotEquals, COMPARING, parse_binary_expr);
        led(Less, COMPARING, parse_binary_expr);
        led(LessEquals, COMPARING, parse_binary_expr);
        led(Greater, COMPARING, parse_binary_expr);
        led(GreaterEquals, COMPARING, parse_binary_expr);


        led(PlusPlus, ASSIGNMENT, parse_increment_expr);
        led(MinusMinus, ASSIGNMENT, parse_decrement_expr);

        led(Assign, ASSIGNMENT, parse_assignment_expr);
        led(PlusAssign, ASSIGNMENT, parse_assignment_expr);
        led(DashAssign, ASSIGNMENT, parse_assignment_expr);
        led(StarAssign, ASSIGNMENT, parse_assignment_expr);
        led(SlashAssign, ASSIGNMENT, parse_assignment_expr);
        led(PercentAssign, ASSIGNMENT, parse_assignment_expr);
        led(RightRightAssign, ASSIGNMENT, parse_assignment_expr);
        led(LeftLeftAssign, ASSIGNMENT, parse_assignment_expr);
        led(VertBarAssign, ASSIGNMENT, parse_assignment_expr);
        led(AmpersandAssign, ASSIGNMENT, parse_assignment_expr);
        led(BitXorAssign, ASSIGNMENT, parse_assignment_expr);

        led(OpenSquare, MEMBER, parse_array_access_expr);

        led(TwoExcl, MEMBER, parse_null_deref);
        led(TwoQuestion, NULL_DECONSTRUCT, parse_binary_expr);

        statement(Let, parse_var_declaration_stmnt);
        statement(Const, parse_var_declaration_stmnt);

        statement(If, parse_if_statement);
        statement(While, parse_while_statement);

        statement(Continue, parse_continue_statement);
        statement(Break, parse_break_statement);

        statement(Fn, parse_fn_declaration_stmt);
        statement(Return, parse_return_stmt);
        
        statement(Struct, parse_struct_statement);
        statement(Enum, parse_enum_statement);

        statement(OpenCurly, parse_block_stmt);

        (nud_lookup, led_lookup, statement_lookup)
    }

}