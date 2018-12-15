extern crate regex;

#[macro_use]
extern crate lazy_static;

extern crate backend;

pub mod ast;
pub mod types;
pub mod errors;
pub mod loc;
pub mod parser;
pub mod print;
pub mod symbol_builder;
pub mod config;
pub mod symbol;
pub mod type_checker;
pub mod jvm_code_gen;
pub mod tac;
pub mod util;
pub mod tac_code_gen;

use symbol_builder::SymbolBuilder;
use type_checker::TypeChecker;
use ast::Program;
use errors::Error;

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
  let program = parser::Parser::new().parse(input)?;
  let program = SymbolBuilder::new().build(program)?;
  let program = TypeChecker::new().check(program)?;
  Ok(program)
}

fn main() {
  let mut input = String::new();
  {
    let filename = env::args().nth(1).unwrap_or_else(|| {
//      "in.txt".to_string()
      eprintln!("Please specify input filename");
      std::process::exit(1);
    });
    let mut f = File::open(filename).unwrap();
    f.read_to_string(&mut input).unwrap();
  }
  let input = string_to_static_str(input);

  match compile(input) {
    Ok(program) => {
      let mut code_gen = jvm_code_gen::JvmCodeGen::new();
      code_gen.gen(program);
    }
    Err(errors) => for error in errors { println!("{}", error); },
  }
}
