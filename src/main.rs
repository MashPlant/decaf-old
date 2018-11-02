extern crate parser;

use std::io;

fn main() {
    let mut parser = parser::Parser::new();
    let node = parser.parse("class main class another");
    let mut printer = parser::ast::IndentPrinter::new();
    node.print_to(&mut printer);
    printer.flush(&mut io::stdout());
}
