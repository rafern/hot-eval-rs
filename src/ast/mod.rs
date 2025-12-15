use lalrpop_util::lalrpop_mod;

lalrpop_mod!(parser, "/ast/parser.rs");
mod utils;

pub mod ast_node;