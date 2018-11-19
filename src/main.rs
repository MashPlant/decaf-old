extern crate regex;

#[macro_use]
extern crate lazy_static;

pub mod ast;
pub mod errors;
pub mod loc;
pub mod parser;
pub mod util;
pub mod symbol_builder;
pub mod config;
pub mod symbol;

use symbol_builder::SymbolBuilder;
use ast::Program;
use errors::Error;

use std::io;
use std::mem;
use std::env;
use std::fs::File;
use std::io::prelude::*;

fn string_to_static_str(s: String) -> &'static str {
    unsafe {
        let ret = mem::transmute(&s as &str);
        mem::forget(s);
        ret
    }
}

fn compile(input: &'static str) -> Result<Program, Vec<Error>> {
    let mut parser = parser::Parser::new();
    let program = parser.parse(input)?;
    let symbol_builder = SymbolBuilder::new();
    let program = symbol_builder.build(program)?;
    Ok(program)
}

fn main() {
    let mut input = String::new();
    {
        let filename = env::args().nth(1).unwrap_or_else(|| {
            eprintln!("Please specify input filename");
            std::process::exit(1);
        });
        let mut f = File::open(filename).unwrap();
        f.read_to_string(&mut input).unwrap();
    }
    let input = string_to_static_str(input);

    match compile(input) {
        Ok(program) => {
            let mut printer = util::IndentPrinter::new();
            program.print_ast(&mut printer);
            printer.flush(&mut io::stdout());
        }
        Err(errors) => for error in errors { println!("{}", error); },
    }

//    let mut parser = parser::Parser::new();
//
//    let _ = parser.parse(input)
//        .map(|program| {
//            let mut printer = util::IndentPrinter::new();
//            program.print_to(&mut printer);
//            printer.flush(&mut io::stdout());
//            program
//        })
//        .map(|program|{
//            let mut symbol_builder = SymbolBuilder::new();
//            symbol_builder.build(program)
//        })
//        .map_err(|errors| { for error in errors { println!("{}", error); } })
//        ;
}
