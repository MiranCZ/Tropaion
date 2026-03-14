pub mod type_lookup;

use crate::analysis::type_registry::TypeRegistry;
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::*;
use crate::error::parser_error::ParserError;
use crate::lexer::token::SimpleToken::{Comma, Greater, Less};
use crate::lexer::token::Token;
use crate::lexer::token::Token::*;
use crate::parser::binding_power::DEFAULT;
use crate::parser::handlers::{ReturnedType, TypeLedInfo, TypeNudHandler};
use crate::parser::type_lookups::type_lookup::TypeLookup;
use crate::parser::Parser;
use crate::parser::type_parser::parse_type;

impl Token {
    pub fn type_nud(&self, lookup: &TypeLookup) -> Option<TypeNudHandler> {
        fn handle_symbol(registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedType {
            let token = parser.next()?;

            Ok(match token {
                Identifier(v) => {
                    let ast_type = if v == "int" {
                        Int
                    } else if v == "float" {
                        Float
                    } else if v == "bool" {
                        Bool
                    } else {
                        return parse_symbol_type(v.clone(), registry, parser);
                    };
                    
                    registry.register(ast_type)
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


fn parse_symbol_type(symbol: String, registry: &mut TypeRegistry,parser: &mut Parser) -> ReturnedType {
    let mut generics = vec![];

    if parser.consume_if_next(Less)? {
        loop {
            generics.push(parse_type(registry, parser, DEFAULT.rbp)?);

            if !parser.consume_if_next(Comma)? {
                parser.expect_next(Greater)?;
                break;
            }
        }
    }

    Ok(registry.register(SymbolType {name: symbol, generics}))
}
