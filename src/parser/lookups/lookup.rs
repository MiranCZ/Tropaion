use std::collections::HashMap;
use crate::lexer::token::SimpleToken;
use crate::lexer::token::SimpleToken::*;
use crate::parser::binding_power::{Bp, ASSIGNMENT, LOGICAL_ADD, LOGICAL_MULT, NUMERIC_ADD, NUMERIC_MULT};
use crate::parser::expression_parser::{parse_binary_expr, parse_prefix_expr};
use crate::parser::handlers::{LedHandler, LedInfo, NudHandler, StatementHandler};
use crate::parser::statement_parser::{parse_fn_declaration_stmt, parse_return_stmt, parse_var_declaration_stmnt};

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

        let mut led = |token: SimpleToken, bp: Bp, handler: LedHandler| {
            led_lookup.insert(token, LedInfo{handler, rbp: bp, lfb: bp-1});
        };

        let mut statement = |token: SimpleToken, handler: StatementHandler| {
            statement_lookup.insert(token, handler);
        };

        nud(Dash, parse_prefix_expr);
        nud(Tilde, parse_prefix_expr);
        nud(Exclamation, parse_prefix_expr);

        led(Dash, NUMERIC_ADD, parse_binary_expr);
        led(Plus, NUMERIC_ADD, parse_binary_expr);

        led(Star, NUMERIC_MULT, parse_binary_expr);
        led(Slash, NUMERIC_MULT, parse_binary_expr);
        led(Percent, NUMERIC_MULT, parse_binary_expr);

        led(VerticalBar, LOGICAL_ADD, parse_binary_expr);
        led(Ampersand, LOGICAL_MULT, parse_binary_expr);

        led(LeftLeft, LOGICAL_MULT, parse_binary_expr);
        led(RightRight, LOGICAL_MULT, parse_binary_expr);

        led(BitOr, LOGICAL_ADD, parse_binary_expr);
        led(BitAnd, LOGICAL_MULT, parse_binary_expr);
        led(BitXor, LOGICAL_MULT, parse_binary_expr);

        statement(Let, parse_var_declaration_stmnt);
        statement(Const, parse_var_declaration_stmnt);

        statement(Fn, parse_fn_declaration_stmt);
        statement(Return, parse_return_stmt);

        (nud_lookup, led_lookup, statement_lookup)
    }

}