pub mod type_lookup;

use crate::ast::ast_type::AstType::*;
use crate::error::parser_error::ParserError;
use crate::lexer::token::Token;
use crate::lexer::token::Token::*;
use crate::parser::handlers::{ReturnedType, TypeLedInfo, TypeNudHandler};
use crate::parser::type_lookups::type_lookup::TypeLookup;
use crate::parser::Parser;

impl Token {
    pub fn type_nud(&self, lookup: &TypeLookup) -> Option<TypeNudHandler> {
        fn handle_symbol(parser: &mut Parser) -> ReturnedType {
            let token = parser.next()?;

            Ok(match token {
                Identifier(v) => {
                    if v == "int" {
                        Int
                    } else if v == "float" {
                        Float
                    } else if v == "bool" {
                        Bool
                    } else {
                        SymbolType(v.clone())
                    }

                },

                _ => panic!()
            })
        }

        match self {
            Identifier(_) => Some(handle_symbol),

            SimpleTokenType(t) => lookup.type_nud_lookup.get(t).copied(),

            _ => None,
        }
    }

    pub fn type_led(&self, lookup: &TypeLookup) -> Option<TypeLedInfo> {
        if let SimpleTokenType(t) = self {
            return lookup.type_led_lookup.get(t).copied();
        }

        None
    }

}
