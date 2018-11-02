#![allow(dead_code)]
#![allow(unused_mut)]

extern crate regex;

#[macro_use]
extern crate lazy_static;

use regex::Regex;
use std::collections::HashMap;

/**
 * Stack value.
 */
enum SV {
    Undefined,
    _0(Token),
    _1(Tree)
}

/**
 * Lex rules.
 */
static LEX_RULES: [&'static str; 47] = [
    r"^void",
    r"^int",
    r"^bool",
    r"^string",
    r"^new",
    r"^null",
    r"^class",
    r"^extends",
    r"^this",
    r"^while",
    r"^for",
    r"^if",
    r"^else",
    r"^return",
    r"^break",
    r"^Print",
    r"^ReadInteger",
    r"^ReadLine",
    r"^static",
    r"^instanceof",
    r"^scopy",
    r"^sealed",
    r"^var",
    r"^default",
    r"^in",
    r"^foreach",
    r"^<=",
    r"^>=",
    r"^==",
    r"^!=",
    r"^&&",
    r"^\|\|",
    r"^%%",
    r"^\+\+",
    r"^\|\|\|",
    r"^\[",
    r"^\]",
    r"^\+",
    r"^\*",
    r"^\(",
    r"^\)",
    r"^\s+",
    r"^\d+",
    r"^[A-Za-z][_0-9A-Za-z]*",
    r"^;",
    r"^\[",
    r"^\]"
];

/**
 * EOF value.
 */
static EOF: &'static str = "$";

/**
 * A macro for map literals.
 *
 * hashmap!{ 1 => "one", 2 => "two" };
 */
macro_rules! hashmap(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

/**
 * Unwraps a SV for the result. The result type is known from the grammar.
 */
macro_rules! get_result {
    ($r:expr, $ty:ident) => (match $r { SV::$ty(v) => v, _ => unreachable!() });
}

/**
 * Pops a SV with needed enum value.
 */
macro_rules! pop {
    ($s:expr, $ty:ident) => (get_result!($s.pop().unwrap(), $ty));
}

/**
 * Productions data.
 *
 * 0 - encoded non-terminal, 1 - length of RHS to pop from the stack
 */
static PRODUCTIONS : [[i32; 2]; 13] = [
    [-1, 1],
    [0, 2],
    [0, 1],
    [1, 2],
    [2, 2],
    [3, 2],
    [4, 1],
    [4, 1],
    [4, 1],
    [4, 1],
    [4, 2],
    [4, 3],
    [5, 1]
];

/**
 * Table entry.
 */
enum TE {
    Accept,

    // Shift, and transit to the state.
    Shift(usize),

    // Reduce by a production number.
    Reduce(usize),

    // Simple state transition.
    Transit(usize),
}

lazy_static! {
    /**
     * Lexical rules grouped by lexer state (by start condition).
     */
    static ref LEX_RULES_BY_START_CONDITIONS: HashMap<&'static str, Vec<i32>> = hashmap! { "INITIAL" => vec! [ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46 ] };

    /**
     * Maps a string name of a token type to its encoded number (the first
     * token number starts after all numbers for non-terminal).
     */
    static ref TOKENS_MAP: HashMap<&'static str, i32> = hashmap! { "CLASS" => 6, "INT" => 7, "VOID" => 8, "BOOL" => 9, "STRING" => 10, "IDENTIFIER" => 11, "';'" => 12, "'['" => 13, "']'" => 14, "$" => 15 };

    /**
     * Parsing table.
     *
     * Vector index is the state number, value is a map
     * from an encoded symbol to table entry (TE).
     */
    static ref TABLE: Vec<HashMap<i32, TE>>= vec![
    hashmap! { 0 => TE::Transit(1), 1 => TE::Transit(2), 6 => TE::Shift(3) },
    hashmap! { 1 => TE::Transit(4), 6 => TE::Shift(3), 15 => TE::Accept },
    hashmap! { 6 => TE::Reduce(2), 15 => TE::Reduce(2) },
    hashmap! { 5 => TE::Transit(5), 11 => TE::Shift(6) },
    hashmap! { 6 => TE::Reduce(1), 15 => TE::Reduce(1) },
    hashmap! { 6 => TE::Reduce(3), 15 => TE::Reduce(3) },
    hashmap! { 6 => TE::Reduce(12), 15 => TE::Reduce(12) }
];
}

// ------------------------------------
// Module include prologue.
//
// Should include at least result type:
//
// type TResult = <...>;
//
// Can also include parsing hooks:
//
//   fn on_parse_begin(parser: &mut Parser, string: &'static str) {
//     ...
//   }
//
//   fn on_parse_begin(parser: &mut Parser, string: &'static str) {
//     ...
//   }
//

pub mod ast;

use ast::*;
use std::mem;

impl Parser {
    fn get_loc(&self) -> Location {
        Location(self.tokenizer.token_start_line, self.tokenizer.token_start_column)
    }
}

impl Token {
    fn get_loc(&self) -> Location {
        Location(self.start_line, self.start_column)
    }
}

macro_rules! get_move {
    ($r:expr, $ty:ident) => ({
        if let TreeData::$ty(v) = mem::replace(&mut $r.data, TreeData::None) {
            v
        } else {
            unreachable!()
        }
    });
}

macro_rules! get_ref {
    ($r:expr, $ty:ident) => (match &mut $r.data { TreeData::$ty(v) => v, _ => unreachable!() });
}

// Final result type returned from `parse` method call.
pub type TResult = Tree;

// ---  end of Module include ---------

/**
 * Generic tokenizer used by the parser in the Syntax tool.
 *
 * https://www.npmjs.com/package/syntax-cli
 */

// ------------------------------------------------------------------
// Token.

#[derive(Debug, Clone, Copy)]
struct Token {
    kind: i32,
    value: &'static str,

    start_offset: i32,
    end_offset: i32,
    start_line: i32,
    end_line: i32,
    start_column: i32,
    end_column: i32,
}

// NOTE: LEX_RULES_BY_START_CONDITIONS, and TOKENS_MAP
// are defined in the lazy_static! block in lr.templates.rs

// ------------------------------------------------------------------
// Tokenizer.

struct Tokenizer {
    /**
     * Tokenizing string.
     */
    string: &'static str,

    /**
     * Cursor for current symbol.
     */
    cursor: i32,

    /**
     * States.
     */
    states: Vec<&'static str>,

    /**
     * Line-based location tracking.
     */
    current_line: i32,
    current_column: i32,
    current_line_begin_offset: i32,

    /**
     * Location data of a matched token.
     */
    token_start_offset: i32,
    token_end_offset: i32,
    token_start_line: i32,
    token_end_line: i32,
    token_start_column: i32,
    token_end_column: i32,

    /**
     * Matched text, and its length.
     */
    yytext: &'static str,
    yyleng: usize,

    handlers: [fn(&mut Tokenizer) -> &'static str; 47],
}

impl Tokenizer {

    /**
     * Creates a new Tokenizer instance.
     *
     * The same instance can be then reused in parser
     * by calling `init_string`.
     */
    pub fn new() -> Tokenizer {
        let mut tokenizer = Tokenizer {
            string: "",
            cursor: 0,

            states: Vec::new(),

            current_line: 1,
            current_column: 0,
            current_line_begin_offset: 0,

            token_start_offset: 0,
            token_end_offset: 0,
            token_start_line: 0,
            token_end_line: 0,
            token_start_column: 0,
            token_end_column: 0,

            yytext: "",
            yyleng: 0,

            handlers: [
    Tokenizer::_lex_rule0,
    Tokenizer::_lex_rule1,
    Tokenizer::_lex_rule2,
    Tokenizer::_lex_rule3,
    Tokenizer::_lex_rule4,
    Tokenizer::_lex_rule5,
    Tokenizer::_lex_rule6,
    Tokenizer::_lex_rule7,
    Tokenizer::_lex_rule8,
    Tokenizer::_lex_rule9,
    Tokenizer::_lex_rule10,
    Tokenizer::_lex_rule11,
    Tokenizer::_lex_rule12,
    Tokenizer::_lex_rule13,
    Tokenizer::_lex_rule14,
    Tokenizer::_lex_rule15,
    Tokenizer::_lex_rule16,
    Tokenizer::_lex_rule17,
    Tokenizer::_lex_rule18,
    Tokenizer::_lex_rule19,
    Tokenizer::_lex_rule20,
    Tokenizer::_lex_rule21,
    Tokenizer::_lex_rule22,
    Tokenizer::_lex_rule23,
    Tokenizer::_lex_rule24,
    Tokenizer::_lex_rule25,
    Tokenizer::_lex_rule26,
    Tokenizer::_lex_rule27,
    Tokenizer::_lex_rule28,
    Tokenizer::_lex_rule29,
    Tokenizer::_lex_rule30,
    Tokenizer::_lex_rule31,
    Tokenizer::_lex_rule32,
    Tokenizer::_lex_rule33,
    Tokenizer::_lex_rule34,
    Tokenizer::_lex_rule35,
    Tokenizer::_lex_rule36,
    Tokenizer::_lex_rule37,
    Tokenizer::_lex_rule38,
    Tokenizer::_lex_rule39,
    Tokenizer::_lex_rule40,
    Tokenizer::_lex_rule41,
    Tokenizer::_lex_rule42,
    Tokenizer::_lex_rule43,
    Tokenizer::_lex_rule44,
    Tokenizer::_lex_rule45,
    Tokenizer::_lex_rule46
],
        };

        tokenizer
    }

    /**
     * Initializes a parsing string.
     */
    pub fn init_string(&mut self, string: &'static str) -> &mut Tokenizer {
        self.string = string;

        // Initialize states.
        self.states.clear();
        self.states.push("INITIAL");

        self.cursor = 0;
        self.current_line = 1;
        self.current_column = 0;
        self.current_line_begin_offset = 0;

        self.token_start_offset = 0;
        self.token_end_offset = 0;
        self.token_start_line = 0;
        self.token_end_line = 0;
        self.token_start_column = 0;
        self.token_end_column = 0;

        self
    }

    /**
     * Returns next token.
     */
    pub fn get_next_token(&mut self) -> Token {
        if !self.has_more_tokens() {
            self.yytext = EOF;
            return self.to_token(EOF)
        }

        let str_slice = &self.string[self.cursor as usize..];

        let lex_rules_for_state = LEX_RULES_BY_START_CONDITIONS
            .get(self.get_current_state())
            .unwrap();

        for i in 0..lex_rules_for_state.len() {
            let lex_rule = LEX_RULES[i];

            if let Some(matched) = self._match(str_slice, &Regex::new(lex_rule).unwrap()) {

                // Manual handling of EOF token (the end of string). Return it
                // as `EOF` symbol.
                if matched.len() == 0 {
                    self.cursor = self.cursor + 1;
                }

                self.yytext = matched;
                self.yyleng = matched.len();

                let token_type = self.handlers[i](self);

                // "" - no token (skip)
                if token_type.len() == 0 {
                    return self.get_next_token();
                }

                return self.to_token(token_type)
            }
        }

        if self.is_eof() {
            self.cursor = self.cursor + 1;
            self.yytext = EOF;
            return self.to_token(EOF);
        }

        self.panic_unexpected_token(
            &str_slice[0..1],
            self.current_line,
            self.current_column
        );

        unreachable!()
    }

    /**
     * Throws default "Unexpected token" exception, showing the actual
     * line from the source, pointing with the ^ marker to the bad token.
     * In addition, shows `line:column` location.
     */
    fn panic_unexpected_token(&self, string: &'static str, line: i32, column: i32) {
        let line_source = self.string
            .split('\n')
            .collect::<Vec<&str>>()
            [(line - 1) as usize];

        let pad = ::std::iter::repeat(" ")
            .take(column as usize)
            .collect::<String>();

        let line_data = format!("\n\n{}\n{}^\n", line_source, pad);

        panic!(
            "{} Unexpected token: \"{}\" at {}:{}.",
            line_data,
            string,
            line,
            column
        );
    }

    fn capture_location(&mut self, matched: &'static str) {
        let nl_re = Regex::new(r"\n").unwrap();

        // Absolute offsets.
        self.token_start_offset = self.cursor;

        // Line-based locations, start.
        self.token_start_line = self.current_line;
        self.token_start_column = self.token_start_offset - self.current_line_begin_offset;

        // Extract `\n` in the matched token.
        for cap in nl_re.captures_iter(matched) {
            self.current_line = self.current_line + 1;
            self.current_line_begin_offset = self.token_start_offset +
                cap.get(0).unwrap().start() as i32 + 1;
        }

        self.token_end_offset = self.cursor + matched.len() as i32;

        // Line-based locations, end.
        self.token_end_line = self.current_line;
        self.token_end_column = self.token_end_offset - self.current_line_begin_offset;
        self.current_column = self.token_end_column;
    }

    fn _match(&mut self, str_slice: &'static str, re: &Regex) -> Option<&'static str> {
        match re.captures(str_slice) {
            Some(caps) => {
                let matched = caps.get(0).unwrap().as_str();
                self.capture_location(matched);
                self.cursor = self.cursor + (matched.len() as i32);
                Some(matched)
            },
            None => None
        }
    }

    fn to_token(&self, token_type: &'static str) -> Token {
        Token {
            kind: *TOKENS_MAP.get(token_type).unwrap(),
            value: self.yytext,
            start_offset: self.token_start_offset,
            end_offset: self.token_end_offset,
            start_line: self.token_start_line,
            end_line: self.token_end_line,
            start_column: self.token_start_column,
            end_column: self.token_end_column,
        }
    }

    /**
     * Whether there are still tokens in the stream.
     */
    pub fn has_more_tokens(&mut self) -> bool {
        self.cursor <= self.string.len() as i32
    }

    /**
     * Whether the cursor is at the EOF.
     */
    pub fn is_eof(&mut self) -> bool {
        self.cursor == self.string.len() as i32
    }

    /**
     * Returns current tokenizing state.
     */
    pub fn get_current_state(&mut self) -> &'static str {
        match self.states.last() {
            Some(last) => last,
            None => "INITIAL"
        }
    }

    /**
     * Enters a new state pushing it on the states stack.
     */
    pub fn push_state(&mut self, state: &'static str) -> &mut Tokenizer {
        self.states.push(state);
        self
    }

    /**
     * Alias for `push_state`.
     */
    pub fn begin(&mut self, state: &'static str) -> &mut Tokenizer {
        self.push_state(state);
        self
    }

    /**
     * Exits a current state popping it from the states stack.
     */
    pub fn pop_state(&mut self) -> &'static str {
        match self.states.pop() {
            Some(top) => top,
            None => "INITIAL"
        }
    }

    /**
     * Lex rule handlers.
     */
    fn _lex_rule0(&mut self) -> &'static str {
return "VOID";
}

fn _lex_rule1(&mut self) -> &'static str {
return "INT";
}

fn _lex_rule2(&mut self) -> &'static str {
return "BOOL";
}

fn _lex_rule3(&mut self) -> &'static str {
return "STRING";
}

fn _lex_rule4(&mut self) -> &'static str {
return "NEW";
}

fn _lex_rule5(&mut self) -> &'static str {
return "NULL";
}

fn _lex_rule6(&mut self) -> &'static str {
return "CLASS";
}

fn _lex_rule7(&mut self) -> &'static str {
return "EXTENDS";
}

fn _lex_rule8(&mut self) -> &'static str {
return "THIS";
}

fn _lex_rule9(&mut self) -> &'static str {
return "WHILE";
}

fn _lex_rule10(&mut self) -> &'static str {
return "FOR";
}

fn _lex_rule11(&mut self) -> &'static str {
return "IF";
}

fn _lex_rule12(&mut self) -> &'static str {
return "ELSE";
}

fn _lex_rule13(&mut self) -> &'static str {
return "RETURN";
}

fn _lex_rule14(&mut self) -> &'static str {
return "BREAK";
}

fn _lex_rule15(&mut self) -> &'static str {
return "PRINT";
}

fn _lex_rule16(&mut self) -> &'static str {
return "READ_INTEGER";
}

fn _lex_rule17(&mut self) -> &'static str {
return "READ_LINE";
}

fn _lex_rule18(&mut self) -> &'static str {
return "STATIC";
}

fn _lex_rule19(&mut self) -> &'static str {
return "INSTANCEOF";
}

fn _lex_rule20(&mut self) -> &'static str {
return "SCOPY";
}

fn _lex_rule21(&mut self) -> &'static str {
return "SEALED";
}

fn _lex_rule22(&mut self) -> &'static str {
return "VAR";
}

fn _lex_rule23(&mut self) -> &'static str {
return "DEFAULT";
}

fn _lex_rule24(&mut self) -> &'static str {
return "IN";
}

fn _lex_rule25(&mut self) -> &'static str {
return "FOREACH";
}

fn _lex_rule26(&mut self) -> &'static str {
return "LESS_EQUAL";
}

fn _lex_rule27(&mut self) -> &'static str {
return "GREATER_EQUAL";
}

fn _lex_rule28(&mut self) -> &'static str {
return "EQUAL";
}

fn _lex_rule29(&mut self) -> &'static str {
return "NOT_EQUAL";
}

fn _lex_rule30(&mut self) -> &'static str {
return "AND";
}

fn _lex_rule31(&mut self) -> &'static str {
return "OR";
}

fn _lex_rule32(&mut self) -> &'static str {
return "REPEAT";
}

fn _lex_rule33(&mut self) -> &'static str {
return "CONCAT";
}

fn _lex_rule34(&mut self) -> &'static str {
return "GUARD_SPLIT";
}

fn _lex_rule35(&mut self) -> &'static str {
return "COMP_LEFT";
}

fn _lex_rule36(&mut self) -> &'static str {
return "COMP_RIGHT";
}

fn _lex_rule37(&mut self) -> &'static str {
return "+";
}

fn _lex_rule38(&mut self) -> &'static str {
return "*";
}

fn _lex_rule39(&mut self) -> &'static str {
return "(";
}

fn _lex_rule40(&mut self) -> &'static str {
return ")";
}

fn _lex_rule41(&mut self) -> &'static str {
return "";
}

fn _lex_rule42(&mut self) -> &'static str {
return "NUMBER";
}

fn _lex_rule43(&mut self) -> &'static str {
return "IDENTIFIER";
}

fn _lex_rule44(&mut self) -> &'static str {
return "';'";
}

fn _lex_rule45(&mut self) -> &'static str {
return "'['";
}

fn _lex_rule46(&mut self) -> &'static str {
return "']'";
}
}

// ------------------------------------------------------------------
// Parser.

/**
 * Parser.
 */
pub struct Parser {
    /**
     * Parsing stack: semantic values.
     */
    values_stack: Vec<SV>,

    /**
     * Parsing stack: state numbers.
     */
    states_stack: Vec<usize>,

    /**
     * Tokenizer instance.
     */
    tokenizer: Tokenizer,

    /**
     * Semantic action handlers.
     */
    handlers: [fn(&mut Parser) -> SV; 13],
}

impl Parser {
    /**
     * Creates a new Parser instance.
     */
    pub fn new() -> Parser {
        Parser {
            // Stacks.
            values_stack: Vec::new(),
            states_stack: Vec::new(),

            tokenizer: Tokenizer::new(),

            handlers: [
    Parser::_handler0,
    Parser::_handler1,
    Parser::_handler2,
    Parser::_handler3,
    Parser::_handler4,
    Parser::_handler5,
    Parser::_handler6,
    Parser::_handler7,
    Parser::_handler8,
    Parser::_handler9,
    Parser::_handler10,
    Parser::_handler11,
    Parser::_handler12
],
        }
    }

    /**
     * Parses a string.
     */
    pub fn parse(&mut self, string: &'static str) -> TResult {
        

        // Initialize the tokenizer and the string.
        self.tokenizer.init_string(string);

        // Initialize the stacks.
        self.values_stack.clear();

        // Initial 0 state.
        self.states_stack.clear();
        self.states_stack.push(0);

        let mut token = self.tokenizer.get_next_token();
        let mut shifted_token = token;

        loop {
            let state = *self.states_stack.last().unwrap();
            let column = token.kind;

            if !TABLE[state].contains_key(&column) {
                self.unexpected_token(&token);
                break;
            }

            let entry = &TABLE[state][&column];

            match entry {

                // Shift a token, go to state.
                &TE::Shift(next_state) => {
                    // Push token.
                    self.values_stack.push(SV::_0(token));

                    // Push next state number: "s5" -> 5
                    self.states_stack.push(next_state as usize);

                    shifted_token = token;
                    token = self.tokenizer.get_next_token();
                },

                // Reduce by production.
                &TE::Reduce(production_number) => {
                    let production = PRODUCTIONS[production_number];

                    self.tokenizer.yytext = shifted_token.value;
                    self.tokenizer.yyleng = shifted_token.value.len();

                    let mut rhs_length = production[1];
                    while rhs_length > 0 {
                        self.states_stack.pop();
                        rhs_length = rhs_length - 1;
                    }

                    // Call the handler, push result onto the stack.
                    let result_value = self.handlers[production_number](self);

                    let previous_state = *self.states_stack.last().unwrap();
                    let symbol_to_reduce_with = production[0];

                    // Then push LHS onto the stack.
                    self.values_stack.push(result_value);

                    let next_state = match &TABLE[previous_state][&symbol_to_reduce_with] {
                        &TE::Transit(next_state) => next_state,
                        _ => unreachable!(),
                    };

                    self.states_stack.push(next_state);
                },

                // Accept the string.
                &TE::Accept => {
                    // Pop state number.
                    self.states_stack.pop();

                    // Pop the parsed value.
                    let parsed = self.values_stack.pop().unwrap();

                    if self.states_stack.len() != 1 ||
                        self.states_stack.pop().unwrap() != 0 ||
                        self.tokenizer.has_more_tokens() {
                        self.unexpected_token(&token);
                    }

                    let result = get_result!(parsed, _1);
                    
                    return result;
                },

                _ => unreachable!(),
            }
        }

        unreachable!();
    }

    fn unexpected_token(&mut self, token: &Token) {
        if token.value == EOF && !self.tokenizer.has_more_tokens() {
            self.unexpected_end_of_input();
        }

        self.tokenizer.panic_unexpected_token(
            token.value,
            token.start_line,
            token.start_column
        );
    }

    fn unexpected_end_of_input(&mut self) {
        panic!("\n\nUnexpected end of input.\n\n");
    }

    fn _handler0(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = self.values_stack.pop().unwrap();

let __ = _1;
__
}

fn _handler1(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _1);

let mut ret = _1;
        get_ref!(ret, Program).classes.push(get_move!(_2, ClassDef));
        let __ = ret;
SV::_1(__)
}

fn _handler2(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = Tree {
            loc: NO_LOCATION,
            data: TreeData::Program(Program {
                classes: vec![get_move!(_1, ClassDef)],
            })
        };
SV::_1(__)
}

fn _handler3(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _0);

let __ = Tree {
            loc: _1.get_loc(),
            data: TreeData::ClassDef(ClassDef {
                loc: _1.get_loc(),
                name: get_move!(_2, Identifier),
                parent: None,
                fields: Vec::new(),
                sealed: false,
            })
        };
SV::_1(__)
}

fn _handler4(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler5(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _1);

let __ = Tree {
            loc: _2.loc,
            data: TreeData::VarDef(VarDef {
                loc: _2.loc,
                name: get_move!(_2, Identifier),
                type_: get_move!(_1, Type),
            })
        };
SV::_1(__)
}

fn _handler6(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Tree {
            loc: _1.get_loc(),
            data: TreeData::Type(Type::Basic("int")),
        };
SV::_1(__)
}

fn _handler7(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Tree {
            loc: _1.get_loc(),
            data: TreeData::Type(Type::Basic("void")),
        };
SV::_1(__)
}

fn _handler8(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Tree {
            loc: _1.get_loc(),
            data: TreeData::Type(Type::Basic("bool")),
        };
SV::_1(__)
}

fn _handler9(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Tree {
            loc: _1.get_loc(),
            data: TreeData::Type(Type::Basic("string")),
        };
SV::_1(__)
}

fn _handler10(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _0);

let __ = Tree {
            loc: _1.get_loc(),
            data: TreeData::Type(Type::Class(get_move!(_2, Identifier))),
        };
SV::_1(__)
}

fn _handler11(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _1);

let __ = Tree {
            loc: _1.loc,
            data: TreeData::Type(Type::Array(Some(Box::new(get_move!(_1, Type))))),
        };
SV::_1(__)
}

fn _handler12(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();

let __ = Tree {
            loc: self.get_loc(),
            // self.tokenizer.yytext.to_string() return s the current name
            data: TreeData::Identifier(self.tokenizer.yytext.to_string()),
        };
SV::_1(__)
}
}
