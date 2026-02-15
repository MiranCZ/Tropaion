use crate::lexer::token::SimpleToken;
use crate::lexer::token::SimpleToken::*;
use crate::parser::binding_power::Bp;
use crate::parser::handlers::*;
use std::collections::HashMap;
use crate::parser::type_parser::parse_array_type;

type TypeNudLookup = HashMap<SimpleToken, TypeNudHandler>;
type TypeLedLookup = HashMap<SimpleToken, TypeLedInfo>;


pub struct TypeLookup {
    pub type_nud_lookup: TypeNudLookup,
    pub type_led_lookup: TypeLedLookup
}


impl TypeLookup {

    pub fn new() -> Self {
        let (nud_lookup, led_lookup) = Self::init_lookups();

        Self {
            type_nud_lookup: nud_lookup,
            type_led_lookup: led_lookup
        }
    }

    fn init_lookups() -> (TypeNudLookup, TypeLedLookup) {
        let mut nud_lookup = HashMap::new();
        let mut led_lookup = HashMap::new();

        let mut nud = |token: SimpleToken, handler: TypeNudHandler| {
            nud_lookup.insert(token, handler);
        };

        let mut led = |token: SimpleToken, bp: Bp, type_handler: TypeLedHandler| {
            led_lookup.insert(token, TypeLedInfo{type_handler, rbp: bp, lfb: bp-1});
        };

        nud(OpenSquare, parse_array_type);

        (nud_lookup, led_lookup)
    }

}