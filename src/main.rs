#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate jvm;
extern crate clap;
extern crate llvm_sys;

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
pub mod llvm_code_gen;

use print::{ASTData, ScopeData};
use errors::Error;

use clap::{Arg, App, ArgMatches, ArgGroup};

use std::mem;
use std::fs::File;
use std::io::prelude::*;
use std::io;

fn read_input(filename: &str) -> &'static str {
  match File::open(filename) {
    Ok(mut f) => {
      let mut input = String::new();
      f.read_to_string(&mut input).unwrap();
      unsafe {
        let ret = mem::transmute(&input as &str);
        mem::forget(input);
        ret
      }
    }
    Err(_) => {
      eprintln!("No such file: {}", filename);
      std::process::exit(0)
    }
  }
}

fn compile(input: &'static str, cmd: &ArgMatches) -> Result<(), Vec<Error>> {
  let mut printer = print::IndentPrinter::new();
  let mut program = parser::Parser::new().parse(input)?;
  if cmd.is_present("LEX") {
    program.print_ast(&mut printer);
    printer.flush(&mut io::stdout());
    return Ok(());
  }
  program = symbol_builder::SymbolBuilder::build(program)?;
  program = type_checker::TypeChecker::check(program)?;
  if cmd.is_present("SCOPE") {
    program.print_scope(&mut printer);
    printer.flush(&mut io::stdout());
    return Ok(());
  }
  if cmd.is_present("JVM") {
    jvm_code_gen::JvmCodeGen::gen(program);
    Ok(())
  } else if cmd.is_present("TAC") {
    let tac_program = tac_code_gen::TacCodeGen::gen(&mut program);
    let mut printer = print::IndentPrinter::new();
    tac_program.print_to(&mut printer);
    printer.flush(&mut io::stdout());
    Ok(())
  } else { // llvm
    llvm_code_gen::LLVMCodeGen::gen(program);
    Ok(())
  }
}

fn main() {
  let matches = App::new("Decaf Compiler")
    .author("MashPlant <li-ch17@mails.tsinghua.edu.cn>")
    .arg(Arg::with_name("LEX").short("l").long("lex").help("Dump lexical & syntactical analysis result."))
    .arg(Arg::with_name("SCOPE").short("s").long("scope").help("Dump scope & type check analysis result."))
    .arg(Arg::with_name("TAC").short("t").long("tac").help("Dump tac code."))
    .arg(Arg::with_name("JVM").short("j").long("jvm").help("Dump jvm bytecode to .class file."))
    .arg(Arg::with_name("LLVM").short("L").long("llvm").help("Dump llvm bit code."))
    .group(ArgGroup::with_name("USAGE").required(true).args(&["LEX", "SCOPE", "TAC", "JVM", "LLVM"]))
    .arg(Arg::with_name("INPUT").required(true))
    .get_matches_from(&["", "--llvm", "in.txt"])
  ;
  if let Err(errors) = compile(read_input(matches.value_of("INPUT").unwrap()), &matches) {
    for error in errors { println!("{}", error); }
  }
}