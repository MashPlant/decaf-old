extern crate parser;
extern crate util;

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

fn main() {
    let mut input = String::new();
    {
        let args: Vec<String> = env::args().collect();
        let filename = if args.len() > 1 { &args[1] } else { "in.txt" };
        let mut f = File::open(filename).unwrap();
        f.read_to_string(&mut input).unwrap();
    }
    let input = string_to_static_str(input);

    let mut parser = parser::Parser::new();

    let node = parser.parse(input);
    let mut printer = util::print::IndentPrinter::new();
    node.print_to(&mut printer);
    printer.flush(&mut io::stdout());
}
