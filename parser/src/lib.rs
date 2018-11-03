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
    _1(Sem),
    _2(Program)
}

/**
 * Lex rules.
 */
static LEX_RULES: [&'static str; 71] = [
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
    r"^\+",
    r"^-",
    r"^\*",
    r"^%",
    r"^=",
    r"^<",
    r"^>",
    r"^\.",
    r"^,",
    r"^;",
    r"^!",
    r"^\(",
    r"^\)",
    r"^\[",
    r"^\]",
    r"^\{",
    r"^\}",
    r"^:",
    r"^\s+",
    r"^\d+",
    r"^[A-Za-z][_0-9A-Za-z]*",
    r"^\{",
    r"^\}",
    r"^\(",
    r"^\)",
    r"^,",
    r"^;",
    r"^:",
    r"^=",
    r"^\+",
    r"^\-",
    r"^\*",
    r"^%",
    r"^\[",
    r"^\]",
    r"^\."
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
static PRODUCTIONS : [[i32; 2]; 77] = [
    [-1, 1],
    [0, 1],
    [1, 2],
    [1, 1],
    [2, 7],
    [3, 1],
    [3, 0],
    [4, 2],
    [4, 0],
    [5, 2],
    [5, 2],
    [5, 0],
    [6, 7],
    [6, 6],
    [7, 1],
    [7, 0],
    [8, 3],
    [8, 1],
    [9, 3],
    [10, 2],
    [10, 0],
    [11, 1],
    [11, 2],
    [11, 1],
    [11, 1],
    [11, 1],
    [11, 2],
    [11, 2],
    [11, 2],
    [11, 2],
    [11, 1],
    [11, 1],
    [11, 1],
    [12, 5],
    [13, 9],
    [14, 1],
    [15, 6],
    [16, 2],
    [16, 0],
    [17, 6],
    [18, 9],
    [19, 1],
    [19, 1],
    [20, 2],
    [20, 0],
    [21, 4],
    [22, 1],
    [22, 0],
    [23, 5],
    [23, 3],
    [24, 2],
    [24, 1],
    [25, 4],
    [26, 3],
    [26, 1],
    [27, 3],
    [27, 4],
    [27, 1],
    [27, 0],
    [28, 1],
    [28, 3],
    [28, 3],
    [28, 3],
    [28, 3],
    [29, 2],
    [29, 4],
    [30, 2],
    [30, 0],
    [31, 2],
    [32, 2],
    [33, 1],
    [33, 1],
    [33, 1],
    [33, 1],
    [33, 2],
    [33, 3],
    [34, 1]
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
    static ref LEX_RULES_BY_START_CONDITIONS: HashMap<&'static str, Vec<i32>> = hashmap! { "INITIAL" => vec! [ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70 ] };

    /**
     * Maps a string name of a token type to its encoded number (the first
     * token number starts after all numbers for non-terminal).
     */
    static ref TOKENS_MAP: HashMap<&'static str, i32> = hashmap! { "CLASS" => 35, "SEALED" => 36, "EXTENDS" => 37, "STATIC" => 38, "WHILE" => 39, "FOR" => 40, "SimpleStatement" => 41, "BREAK" => 42, "IF" => 43, "ELSE" => 44, "SCOPY" => 45, "FOREACH" => 46, "IN" => 47, "VAR" => 48, "GUARD_SPLIT" => 49, "RETURN" => 50, "PRINT" => 51, "INT" => 52, "VOID" => 53, "BOOL" => 54, "STRING" => 55, "IDENTIFIER" => 56, "'{'" => 57, "'}'" => 58, "'('" => 59, "')'" => 60, "','" => 61, "';'" => 62, "':'" => 63, "'='" => 64, "'+'" => 65, "'-'" => 66, "'*'" => 67, "'%'" => 68, "'['" => 69, "']'" => 70, "'.'" => 71, "$" => 72 };

    /**
     * Parsing table.
     *
     * Vector index is the state number, value is a map
     * from an encoded symbol to table entry (TE).
     */
    static ref TABLE: Vec<HashMap<i32, TE>>= vec![
    hashmap! { 0 => TE::Transit(1), 1 => TE::Transit(2), 2 => TE::Transit(3), 3 => TE::Transit(4), 35 => TE::Reduce(6), 36 => TE::Shift(5) },
    hashmap! { 72 => TE::Accept },
    hashmap! { 2 => TE::Transit(6), 3 => TE::Transit(4), 35 => TE::Reduce(6), 36 => TE::Shift(5), 72 => TE::Reduce(1) },
    hashmap! { 35 => TE::Reduce(3), 36 => TE::Reduce(3), 72 => TE::Reduce(3) },
    hashmap! { 35 => TE::Shift(7) },
    hashmap! { 35 => TE::Reduce(5) },
    hashmap! { 35 => TE::Reduce(2), 36 => TE::Reduce(2), 72 => TE::Reduce(2) },
    hashmap! { 34 => TE::Transit(8), 56 => TE::Shift(9) },
    hashmap! { 4 => TE::Transit(10), 37 => TE::Shift(11), 57 => TE::Reduce(8) },
    hashmap! { 37 => TE::Reduce(76), 39 => TE::Reduce(76), 47 => TE::Reduce(76), 56 => TE::Reduce(76), 57 => TE::Reduce(76), 59 => TE::Reduce(76), 60 => TE::Reduce(76), 61 => TE::Reduce(76), 62 => TE::Reduce(76), 63 => TE::Reduce(76), 64 => TE::Reduce(76), 65 => TE::Reduce(76), 66 => TE::Reduce(76), 67 => TE::Reduce(76), 68 => TE::Reduce(76), 69 => TE::Reduce(76), 70 => TE::Reduce(76), 71 => TE::Reduce(76) },
    hashmap! { 57 => TE::Shift(12) },
    hashmap! { 34 => TE::Transit(152), 56 => TE::Shift(9) },
    hashmap! { 5 => TE::Transit(13), 35 => TE::Reduce(11), 38 => TE::Reduce(11), 52 => TE::Reduce(11), 53 => TE::Reduce(11), 54 => TE::Reduce(11), 55 => TE::Reduce(11), 58 => TE::Reduce(11) },
    hashmap! { 6 => TE::Transit(16), 31 => TE::Transit(15), 32 => TE::Transit(17), 33 => TE::Transit(18), 35 => TE::Shift(23), 38 => TE::Shift(24), 52 => TE::Shift(19), 53 => TE::Shift(20), 54 => TE::Shift(21), 55 => TE::Shift(22), 58 => TE::Shift(14) },
    hashmap! { 35 => TE::Reduce(4), 36 => TE::Reduce(4), 72 => TE::Reduce(4) },
    hashmap! { 35 => TE::Reduce(9), 38 => TE::Reduce(9), 52 => TE::Reduce(9), 53 => TE::Reduce(9), 54 => TE::Reduce(9), 55 => TE::Reduce(9), 58 => TE::Reduce(9) },
    hashmap! { 35 => TE::Reduce(10), 38 => TE::Reduce(10), 52 => TE::Reduce(10), 53 => TE::Reduce(10), 54 => TE::Reduce(10), 55 => TE::Reduce(10), 58 => TE::Reduce(10) },
    hashmap! { 62 => TE::Shift(25) },
    hashmap! { 34 => TE::Transit(26), 56 => TE::Shift(9), 69 => TE::Shift(27) },
    hashmap! { 56 => TE::Reduce(70), 69 => TE::Reduce(70) },
    hashmap! { 56 => TE::Reduce(71), 69 => TE::Reduce(71) },
    hashmap! { 56 => TE::Reduce(72), 69 => TE::Reduce(72) },
    hashmap! { 56 => TE::Reduce(73), 69 => TE::Reduce(73) },
    hashmap! { 34 => TE::Transit(70), 56 => TE::Shift(9) },
    hashmap! { 33 => TE::Transit(146), 35 => TE::Shift(23), 52 => TE::Shift(19), 53 => TE::Shift(20), 54 => TE::Shift(21), 55 => TE::Shift(22) },
    hashmap! { 35 => TE::Reduce(68), 38 => TE::Reduce(68), 39 => TE::Reduce(68), 40 => TE::Reduce(68), 42 => TE::Reduce(68), 43 => TE::Reduce(68), 44 => TE::Reduce(68), 45 => TE::Reduce(68), 46 => TE::Reduce(68), 48 => TE::Reduce(68), 49 => TE::Reduce(68), 50 => TE::Reduce(68), 51 => TE::Reduce(68), 52 => TE::Reduce(68), 53 => TE::Reduce(68), 54 => TE::Reduce(68), 55 => TE::Reduce(68), 56 => TE::Reduce(68), 57 => TE::Reduce(68), 58 => TE::Reduce(68), 62 => TE::Reduce(68) },
    hashmap! { 59 => TE::Shift(28), 62 => TE::Reduce(69) },
    hashmap! { 70 => TE::Shift(69) },
    hashmap! { 7 => TE::Transit(29), 8 => TE::Transit(30), 32 => TE::Transit(31), 33 => TE::Transit(32), 35 => TE::Shift(23), 52 => TE::Shift(19), 53 => TE::Shift(20), 54 => TE::Shift(21), 55 => TE::Shift(22), 60 => TE::Reduce(15) },
    hashmap! { 60 => TE::Shift(33) },
    hashmap! { 60 => TE::Reduce(14), 61 => TE::Shift(144) },
    hashmap! { 60 => TE::Reduce(17), 61 => TE::Reduce(17) },
    hashmap! { 34 => TE::Transit(68), 56 => TE::Shift(9), 69 => TE::Shift(27) },
    hashmap! { 9 => TE::Transit(34), 57 => TE::Shift(35) },
    hashmap! { 35 => TE::Reduce(13), 38 => TE::Reduce(13), 52 => TE::Reduce(13), 53 => TE::Reduce(13), 54 => TE::Reduce(13), 55 => TE::Reduce(13), 58 => TE::Reduce(13) },
    hashmap! { 10 => TE::Transit(36), 35 => TE::Reduce(20), 39 => TE::Reduce(20), 40 => TE::Reduce(20), 42 => TE::Reduce(20), 43 => TE::Reduce(20), 45 => TE::Reduce(20), 46 => TE::Reduce(20), 48 => TE::Reduce(20), 50 => TE::Reduce(20), 51 => TE::Reduce(20), 52 => TE::Reduce(20), 53 => TE::Reduce(20), 54 => TE::Reduce(20), 55 => TE::Reduce(20), 56 => TE::Reduce(20), 57 => TE::Reduce(20), 58 => TE::Reduce(20), 62 => TE::Reduce(20) },
    hashmap! { 9 => TE::Transit(50), 11 => TE::Transit(38), 12 => TE::Transit(42), 13 => TE::Transit(43), 14 => TE::Transit(46), 15 => TE::Transit(41), 17 => TE::Transit(47), 18 => TE::Transit(48), 21 => TE::Transit(49), 24 => TE::Transit(44), 25 => TE::Transit(45), 27 => TE::Transit(40), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(39), 32 => TE::Transit(17), 33 => TE::Transit(32), 35 => TE::Shift(23), 39 => TE::Shift(56), 40 => TE::Shift(57), 42 => TE::Shift(60), 43 => TE::Shift(55), 45 => TE::Shift(61), 46 => TE::Shift(62), 48 => TE::Shift(52), 50 => TE::Shift(58), 51 => TE::Shift(59), 52 => TE::Shift(19), 53 => TE::Shift(20), 54 => TE::Shift(21), 55 => TE::Shift(22), 56 => TE::Reduce(67), 57 => TE::Shift(35), 58 => TE::Shift(37), 62 => TE::Reduce(58) },
    hashmap! { 35 => TE::Reduce(18), 38 => TE::Reduce(18), 39 => TE::Reduce(18), 40 => TE::Reduce(18), 42 => TE::Reduce(18), 43 => TE::Reduce(18), 44 => TE::Reduce(18), 45 => TE::Reduce(18), 46 => TE::Reduce(18), 48 => TE::Reduce(18), 49 => TE::Reduce(18), 50 => TE::Reduce(18), 51 => TE::Reduce(18), 52 => TE::Reduce(18), 53 => TE::Reduce(18), 54 => TE::Reduce(18), 55 => TE::Reduce(18), 56 => TE::Reduce(18), 57 => TE::Reduce(18), 58 => TE::Reduce(18), 62 => TE::Reduce(18) },
    hashmap! { 35 => TE::Reduce(19), 39 => TE::Reduce(19), 40 => TE::Reduce(19), 42 => TE::Reduce(19), 43 => TE::Reduce(19), 45 => TE::Reduce(19), 46 => TE::Reduce(19), 48 => TE::Reduce(19), 50 => TE::Reduce(19), 51 => TE::Reduce(19), 52 => TE::Reduce(19), 53 => TE::Reduce(19), 54 => TE::Reduce(19), 55 => TE::Reduce(19), 56 => TE::Reduce(19), 57 => TE::Reduce(19), 58 => TE::Reduce(19), 62 => TE::Reduce(19) },
    hashmap! { 35 => TE::Reduce(21), 39 => TE::Reduce(21), 40 => TE::Reduce(21), 42 => TE::Reduce(21), 43 => TE::Reduce(21), 44 => TE::Reduce(21), 45 => TE::Reduce(21), 46 => TE::Reduce(21), 48 => TE::Reduce(21), 49 => TE::Reduce(21), 50 => TE::Reduce(21), 51 => TE::Reduce(21), 52 => TE::Reduce(21), 53 => TE::Reduce(21), 54 => TE::Reduce(21), 55 => TE::Reduce(21), 56 => TE::Reduce(21), 57 => TE::Reduce(21), 58 => TE::Reduce(21), 62 => TE::Reduce(21) },
    hashmap! { 62 => TE::Shift(63) },
    hashmap! { 35 => TE::Reduce(23), 39 => TE::Reduce(23), 40 => TE::Reduce(23), 42 => TE::Reduce(23), 43 => TE::Reduce(23), 44 => TE::Reduce(23), 45 => TE::Reduce(23), 46 => TE::Reduce(23), 48 => TE::Reduce(23), 49 => TE::Reduce(23), 50 => TE::Reduce(23), 51 => TE::Reduce(23), 52 => TE::Reduce(23), 53 => TE::Reduce(23), 54 => TE::Reduce(23), 55 => TE::Reduce(23), 56 => TE::Reduce(23), 57 => TE::Reduce(23), 58 => TE::Reduce(23), 62 => TE::Reduce(23) },
    hashmap! { 35 => TE::Reduce(24), 39 => TE::Reduce(24), 40 => TE::Reduce(24), 42 => TE::Reduce(24), 43 => TE::Reduce(24), 44 => TE::Reduce(24), 45 => TE::Reduce(24), 46 => TE::Reduce(24), 48 => TE::Reduce(24), 49 => TE::Reduce(24), 50 => TE::Reduce(24), 51 => TE::Reduce(24), 52 => TE::Reduce(24), 53 => TE::Reduce(24), 54 => TE::Reduce(24), 55 => TE::Reduce(24), 56 => TE::Reduce(24), 57 => TE::Reduce(24), 58 => TE::Reduce(24), 62 => TE::Reduce(24) },
    hashmap! { 35 => TE::Reduce(25), 39 => TE::Reduce(25), 40 => TE::Reduce(25), 42 => TE::Reduce(25), 43 => TE::Reduce(25), 44 => TE::Reduce(25), 45 => TE::Reduce(25), 46 => TE::Reduce(25), 48 => TE::Reduce(25), 49 => TE::Reduce(25), 50 => TE::Reduce(25), 51 => TE::Reduce(25), 52 => TE::Reduce(25), 53 => TE::Reduce(25), 54 => TE::Reduce(25), 55 => TE::Reduce(25), 56 => TE::Reduce(25), 57 => TE::Reduce(25), 58 => TE::Reduce(25), 62 => TE::Reduce(25) },
    hashmap! { 62 => TE::Shift(64) },
    hashmap! { 62 => TE::Shift(65) },
    hashmap! { 62 => TE::Shift(66) },
    hashmap! { 62 => TE::Shift(67) },
    hashmap! { 35 => TE::Reduce(30), 39 => TE::Reduce(30), 40 => TE::Reduce(30), 42 => TE::Reduce(30), 43 => TE::Reduce(30), 44 => TE::Reduce(30), 45 => TE::Reduce(30), 46 => TE::Reduce(30), 48 => TE::Reduce(30), 49 => TE::Reduce(30), 50 => TE::Reduce(30), 51 => TE::Reduce(30), 52 => TE::Reduce(30), 53 => TE::Reduce(30), 54 => TE::Reduce(30), 55 => TE::Reduce(30), 56 => TE::Reduce(30), 57 => TE::Reduce(30), 58 => TE::Reduce(30), 62 => TE::Reduce(30) },
    hashmap! { 35 => TE::Reduce(31), 39 => TE::Reduce(31), 40 => TE::Reduce(31), 42 => TE::Reduce(31), 43 => TE::Reduce(31), 44 => TE::Reduce(31), 45 => TE::Reduce(31), 46 => TE::Reduce(31), 48 => TE::Reduce(31), 49 => TE::Reduce(31), 50 => TE::Reduce(31), 51 => TE::Reduce(31), 52 => TE::Reduce(31), 53 => TE::Reduce(31), 54 => TE::Reduce(31), 55 => TE::Reduce(31), 56 => TE::Reduce(31), 57 => TE::Reduce(31), 58 => TE::Reduce(31), 62 => TE::Reduce(31) },
    hashmap! { 35 => TE::Reduce(32), 39 => TE::Reduce(32), 40 => TE::Reduce(32), 42 => TE::Reduce(32), 43 => TE::Reduce(32), 44 => TE::Reduce(32), 45 => TE::Reduce(32), 46 => TE::Reduce(32), 48 => TE::Reduce(32), 49 => TE::Reduce(32), 50 => TE::Reduce(32), 51 => TE::Reduce(32), 52 => TE::Reduce(32), 53 => TE::Reduce(32), 54 => TE::Reduce(32), 55 => TE::Reduce(32), 56 => TE::Reduce(32), 57 => TE::Reduce(32), 58 => TE::Reduce(32), 62 => TE::Reduce(32) },
    hashmap! { 62 => TE::Reduce(59), 64 => TE::Shift(71), 65 => TE::Reduce(59), 66 => TE::Reduce(59), 67 => TE::Reduce(59), 68 => TE::Reduce(59), 69 => TE::Reduce(59), 71 => TE::Reduce(59) },
    hashmap! { 34 => TE::Transit(87), 56 => TE::Shift(9) },
    hashmap! { 62 => TE::Reduce(57), 65 => TE::Shift(74), 66 => TE::Shift(75), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 71 => TE::Shift(79) },
    hashmap! { 34 => TE::Transit(86), 56 => TE::Shift(9) },
    hashmap! { 57 => TE::Shift(91), 59 => TE::Shift(90) },
    hashmap! { 59 => TE::Shift(98) },
    hashmap! { 59 => TE::Shift(102) },
    hashmap! { 28 => TE::Transit(110), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67), 62 => TE::Reduce(51) },
    hashmap! { 59 => TE::Shift(111) },
    hashmap! { 62 => TE::Reduce(35) },
    hashmap! { 59 => TE::Shift(117) },
    hashmap! { 59 => TE::Shift(122) },
    hashmap! { 35 => TE::Reduce(22), 39 => TE::Reduce(22), 40 => TE::Reduce(22), 42 => TE::Reduce(22), 43 => TE::Reduce(22), 44 => TE::Reduce(22), 45 => TE::Reduce(22), 46 => TE::Reduce(22), 48 => TE::Reduce(22), 49 => TE::Reduce(22), 50 => TE::Reduce(22), 51 => TE::Reduce(22), 52 => TE::Reduce(22), 53 => TE::Reduce(22), 54 => TE::Reduce(22), 55 => TE::Reduce(22), 56 => TE::Reduce(22), 57 => TE::Reduce(22), 58 => TE::Reduce(22), 62 => TE::Reduce(22) },
    hashmap! { 35 => TE::Reduce(26), 39 => TE::Reduce(26), 40 => TE::Reduce(26), 42 => TE::Reduce(26), 43 => TE::Reduce(26), 44 => TE::Reduce(26), 45 => TE::Reduce(26), 46 => TE::Reduce(26), 48 => TE::Reduce(26), 49 => TE::Reduce(26), 50 => TE::Reduce(26), 51 => TE::Reduce(26), 52 => TE::Reduce(26), 53 => TE::Reduce(26), 54 => TE::Reduce(26), 55 => TE::Reduce(26), 56 => TE::Reduce(26), 57 => TE::Reduce(26), 58 => TE::Reduce(26), 62 => TE::Reduce(26) },
    hashmap! { 35 => TE::Reduce(27), 39 => TE::Reduce(27), 40 => TE::Reduce(27), 42 => TE::Reduce(27), 43 => TE::Reduce(27), 44 => TE::Reduce(27), 45 => TE::Reduce(27), 46 => TE::Reduce(27), 48 => TE::Reduce(27), 49 => TE::Reduce(27), 50 => TE::Reduce(27), 51 => TE::Reduce(27), 52 => TE::Reduce(27), 53 => TE::Reduce(27), 54 => TE::Reduce(27), 55 => TE::Reduce(27), 56 => TE::Reduce(27), 57 => TE::Reduce(27), 58 => TE::Reduce(27), 62 => TE::Reduce(27) },
    hashmap! { 35 => TE::Reduce(28), 39 => TE::Reduce(28), 40 => TE::Reduce(28), 42 => TE::Reduce(28), 43 => TE::Reduce(28), 44 => TE::Reduce(28), 45 => TE::Reduce(28), 46 => TE::Reduce(28), 48 => TE::Reduce(28), 49 => TE::Reduce(28), 50 => TE::Reduce(28), 51 => TE::Reduce(28), 52 => TE::Reduce(28), 53 => TE::Reduce(28), 54 => TE::Reduce(28), 55 => TE::Reduce(28), 56 => TE::Reduce(28), 57 => TE::Reduce(28), 58 => TE::Reduce(28), 62 => TE::Reduce(28) },
    hashmap! { 35 => TE::Reduce(29), 39 => TE::Reduce(29), 40 => TE::Reduce(29), 42 => TE::Reduce(29), 43 => TE::Reduce(29), 44 => TE::Reduce(29), 45 => TE::Reduce(29), 46 => TE::Reduce(29), 48 => TE::Reduce(29), 49 => TE::Reduce(29), 50 => TE::Reduce(29), 51 => TE::Reduce(29), 52 => TE::Reduce(29), 53 => TE::Reduce(29), 54 => TE::Reduce(29), 55 => TE::Reduce(29), 56 => TE::Reduce(29), 57 => TE::Reduce(29), 58 => TE::Reduce(29), 62 => TE::Reduce(29) },
    hashmap! { 60 => TE::Reduce(69), 61 => TE::Reduce(69), 62 => TE::Reduce(69) },
    hashmap! { 56 => TE::Reduce(75), 69 => TE::Reduce(75) },
    hashmap! { 56 => TE::Reduce(74), 69 => TE::Reduce(74) },
    hashmap! { 28 => TE::Transit(72), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67) },
    hashmap! { 62 => TE::Reduce(55), 65 => TE::Shift(74), 66 => TE::Shift(75), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 71 => TE::Shift(79) },
    hashmap! { 39 => TE::Reduce(59), 60 => TE::Reduce(59), 61 => TE::Reduce(59), 62 => TE::Reduce(59), 63 => TE::Reduce(59), 65 => TE::Reduce(59), 66 => TE::Reduce(59), 67 => TE::Reduce(59), 68 => TE::Reduce(59), 69 => TE::Reduce(59), 70 => TE::Reduce(59), 71 => TE::Reduce(59) },
    hashmap! { 28 => TE::Transit(80), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67) },
    hashmap! { 28 => TE::Transit(81), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67) },
    hashmap! { 28 => TE::Transit(82), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67) },
    hashmap! { 28 => TE::Transit(83), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67) },
    hashmap! { 28 => TE::Transit(84), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67) },
    hashmap! { 56 => TE::Reduce(66) },
    hashmap! { 39 => TE::Reduce(60), 60 => TE::Reduce(60), 61 => TE::Reduce(60), 62 => TE::Reduce(60), 63 => TE::Reduce(60), 65 => TE::Reduce(60), 66 => TE::Reduce(60), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 70 => TE::Reduce(60), 71 => TE::Shift(79) },
    hashmap! { 39 => TE::Reduce(61), 60 => TE::Reduce(61), 61 => TE::Reduce(61), 62 => TE::Reduce(61), 63 => TE::Reduce(61), 65 => TE::Reduce(61), 66 => TE::Reduce(61), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 70 => TE::Reduce(61), 71 => TE::Shift(79) },
    hashmap! { 39 => TE::Reduce(62), 60 => TE::Reduce(62), 61 => TE::Reduce(62), 62 => TE::Reduce(62), 63 => TE::Reduce(62), 65 => TE::Reduce(62), 66 => TE::Reduce(62), 67 => TE::Reduce(62), 68 => TE::Reduce(62), 69 => TE::Shift(78), 70 => TE::Reduce(62), 71 => TE::Shift(79) },
    hashmap! { 39 => TE::Reduce(63), 60 => TE::Reduce(63), 61 => TE::Reduce(63), 62 => TE::Reduce(63), 63 => TE::Reduce(63), 65 => TE::Reduce(63), 66 => TE::Reduce(63), 67 => TE::Reduce(63), 68 => TE::Reduce(63), 69 => TE::Shift(78), 70 => TE::Reduce(63), 71 => TE::Shift(79) },
    hashmap! { 65 => TE::Shift(74), 66 => TE::Shift(75), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 70 => TE::Shift(85), 71 => TE::Shift(79) },
    hashmap! { 39 => TE::Reduce(65), 60 => TE::Reduce(65), 61 => TE::Reduce(65), 62 => TE::Reduce(65), 63 => TE::Reduce(65), 64 => TE::Reduce(65), 65 => TE::Reduce(65), 66 => TE::Reduce(65), 67 => TE::Reduce(65), 68 => TE::Reduce(65), 69 => TE::Reduce(65), 70 => TE::Reduce(65), 71 => TE::Reduce(65) },
    hashmap! { 39 => TE::Reduce(64), 60 => TE::Reduce(64), 61 => TE::Reduce(64), 62 => TE::Reduce(64), 63 => TE::Reduce(64), 64 => TE::Reduce(64), 65 => TE::Reduce(64), 66 => TE::Reduce(64), 67 => TE::Reduce(64), 68 => TE::Reduce(64), 69 => TE::Reduce(64), 70 => TE::Reduce(64), 71 => TE::Reduce(64) },
    hashmap! { 64 => TE::Shift(88) },
    hashmap! { 28 => TE::Transit(89), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67) },
    hashmap! { 62 => TE::Reduce(56), 65 => TE::Shift(74), 66 => TE::Shift(75), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 71 => TE::Shift(79) },
    hashmap! { 28 => TE::Transit(92), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67) },
    hashmap! { 22 => TE::Transit(134), 23 => TE::Transit(135), 28 => TE::Transit(136), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67), 58 => TE::Reduce(47) },
    hashmap! { 60 => TE::Shift(93), 65 => TE::Shift(74), 66 => TE::Shift(75), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 71 => TE::Shift(79) },
    hashmap! { 9 => TE::Transit(50), 11 => TE::Transit(94), 12 => TE::Transit(42), 13 => TE::Transit(43), 14 => TE::Transit(46), 15 => TE::Transit(41), 17 => TE::Transit(47), 18 => TE::Transit(48), 21 => TE::Transit(49), 24 => TE::Transit(44), 25 => TE::Transit(45), 27 => TE::Transit(40), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(39), 32 => TE::Transit(17), 33 => TE::Transit(32), 35 => TE::Shift(23), 39 => TE::Shift(56), 40 => TE::Shift(57), 42 => TE::Shift(60), 43 => TE::Shift(55), 45 => TE::Shift(61), 46 => TE::Shift(62), 48 => TE::Shift(52), 50 => TE::Shift(58), 51 => TE::Shift(59), 52 => TE::Shift(19), 53 => TE::Shift(20), 54 => TE::Shift(21), 55 => TE::Shift(22), 56 => TE::Reduce(67), 57 => TE::Shift(35), 62 => TE::Reduce(58) },
    hashmap! { 16 => TE::Transit(95), 35 => TE::Reduce(38), 39 => TE::Reduce(38), 40 => TE::Reduce(38), 42 => TE::Reduce(38), 43 => TE::Reduce(38), 44 => TE::Shift(96), 45 => TE::Reduce(38), 46 => TE::Reduce(38), 48 => TE::Reduce(38), 49 => TE::Reduce(38), 50 => TE::Reduce(38), 51 => TE::Reduce(38), 52 => TE::Reduce(38), 53 => TE::Reduce(38), 54 => TE::Reduce(38), 55 => TE::Reduce(38), 56 => TE::Reduce(38), 57 => TE::Reduce(38), 58 => TE::Reduce(38), 62 => TE::Reduce(38) },
    hashmap! { 35 => TE::Reduce(36), 39 => TE::Reduce(36), 40 => TE::Reduce(36), 42 => TE::Reduce(36), 43 => TE::Reduce(36), 44 => TE::Reduce(36), 45 => TE::Reduce(36), 46 => TE::Reduce(36), 48 => TE::Reduce(36), 49 => TE::Reduce(36), 50 => TE::Reduce(36), 51 => TE::Reduce(36), 52 => TE::Reduce(36), 53 => TE::Reduce(36), 54 => TE::Reduce(36), 55 => TE::Reduce(36), 56 => TE::Reduce(36), 57 => TE::Reduce(36), 58 => TE::Reduce(36), 62 => TE::Reduce(36) },
    hashmap! { 9 => TE::Transit(50), 11 => TE::Transit(97), 12 => TE::Transit(42), 13 => TE::Transit(43), 14 => TE::Transit(46), 15 => TE::Transit(41), 17 => TE::Transit(47), 18 => TE::Transit(48), 21 => TE::Transit(49), 24 => TE::Transit(44), 25 => TE::Transit(45), 27 => TE::Transit(40), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(39), 32 => TE::Transit(17), 33 => TE::Transit(32), 35 => TE::Shift(23), 39 => TE::Shift(56), 40 => TE::Shift(57), 42 => TE::Shift(60), 43 => TE::Shift(55), 45 => TE::Shift(61), 46 => TE::Shift(62), 48 => TE::Shift(52), 50 => TE::Shift(58), 51 => TE::Shift(59), 52 => TE::Shift(19), 53 => TE::Shift(20), 54 => TE::Shift(21), 55 => TE::Shift(22), 56 => TE::Reduce(67), 57 => TE::Shift(35), 62 => TE::Reduce(58) },
    hashmap! { 35 => TE::Reduce(37), 39 => TE::Reduce(37), 40 => TE::Reduce(37), 42 => TE::Reduce(37), 43 => TE::Reduce(37), 44 => TE::Reduce(37), 45 => TE::Reduce(37), 46 => TE::Reduce(37), 48 => TE::Reduce(37), 49 => TE::Reduce(37), 50 => TE::Reduce(37), 51 => TE::Reduce(37), 52 => TE::Reduce(37), 53 => TE::Reduce(37), 54 => TE::Reduce(37), 55 => TE::Reduce(37), 56 => TE::Reduce(37), 57 => TE::Reduce(37), 58 => TE::Reduce(37), 62 => TE::Reduce(37) },
    hashmap! { 28 => TE::Transit(99), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67) },
    hashmap! { 60 => TE::Shift(100), 65 => TE::Shift(74), 66 => TE::Shift(75), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 71 => TE::Shift(79) },
    hashmap! { 9 => TE::Transit(50), 11 => TE::Transit(101), 12 => TE::Transit(42), 13 => TE::Transit(43), 14 => TE::Transit(46), 15 => TE::Transit(41), 17 => TE::Transit(47), 18 => TE::Transit(48), 21 => TE::Transit(49), 24 => TE::Transit(44), 25 => TE::Transit(45), 27 => TE::Transit(40), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(39), 32 => TE::Transit(17), 33 => TE::Transit(32), 35 => TE::Shift(23), 39 => TE::Shift(56), 40 => TE::Shift(57), 42 => TE::Shift(60), 43 => TE::Shift(55), 45 => TE::Shift(61), 46 => TE::Shift(62), 48 => TE::Shift(52), 50 => TE::Shift(58), 51 => TE::Shift(59), 52 => TE::Shift(19), 53 => TE::Shift(20), 54 => TE::Shift(21), 55 => TE::Shift(22), 56 => TE::Reduce(67), 57 => TE::Shift(35), 62 => TE::Reduce(58) },
    hashmap! { 35 => TE::Reduce(33), 39 => TE::Reduce(33), 40 => TE::Reduce(33), 42 => TE::Reduce(33), 43 => TE::Reduce(33), 44 => TE::Reduce(33), 45 => TE::Reduce(33), 46 => TE::Reduce(33), 48 => TE::Reduce(33), 49 => TE::Reduce(33), 50 => TE::Reduce(33), 51 => TE::Reduce(33), 52 => TE::Reduce(33), 53 => TE::Reduce(33), 54 => TE::Reduce(33), 55 => TE::Reduce(33), 56 => TE::Reduce(33), 57 => TE::Reduce(33), 58 => TE::Reduce(33), 62 => TE::Reduce(33) },
    hashmap! { 41 => TE::Shift(103) },
    hashmap! { 62 => TE::Shift(104) },
    hashmap! { 28 => TE::Transit(105), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67) },
    hashmap! { 62 => TE::Shift(106), 65 => TE::Shift(74), 66 => TE::Shift(75), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 71 => TE::Shift(79) },
    hashmap! { 41 => TE::Shift(107) },
    hashmap! { 60 => TE::Shift(108) },
    hashmap! { 9 => TE::Transit(50), 11 => TE::Transit(109), 12 => TE::Transit(42), 13 => TE::Transit(43), 14 => TE::Transit(46), 15 => TE::Transit(41), 17 => TE::Transit(47), 18 => TE::Transit(48), 21 => TE::Transit(49), 24 => TE::Transit(44), 25 => TE::Transit(45), 27 => TE::Transit(40), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(39), 32 => TE::Transit(17), 33 => TE::Transit(32), 35 => TE::Shift(23), 39 => TE::Shift(56), 40 => TE::Shift(57), 42 => TE::Shift(60), 43 => TE::Shift(55), 45 => TE::Shift(61), 46 => TE::Shift(62), 48 => TE::Shift(52), 50 => TE::Shift(58), 51 => TE::Shift(59), 52 => TE::Shift(19), 53 => TE::Shift(20), 54 => TE::Shift(21), 55 => TE::Shift(22), 56 => TE::Reduce(67), 57 => TE::Shift(35), 62 => TE::Reduce(58) },
    hashmap! { 35 => TE::Reduce(34), 39 => TE::Reduce(34), 40 => TE::Reduce(34), 42 => TE::Reduce(34), 43 => TE::Reduce(34), 44 => TE::Reduce(34), 45 => TE::Reduce(34), 46 => TE::Reduce(34), 48 => TE::Reduce(34), 49 => TE::Reduce(34), 50 => TE::Reduce(34), 51 => TE::Reduce(34), 52 => TE::Reduce(34), 53 => TE::Reduce(34), 54 => TE::Reduce(34), 55 => TE::Reduce(34), 56 => TE::Reduce(34), 57 => TE::Reduce(34), 58 => TE::Reduce(34), 62 => TE::Reduce(34) },
    hashmap! { 62 => TE::Reduce(50), 65 => TE::Shift(74), 66 => TE::Shift(75), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 71 => TE::Shift(79) },
    hashmap! { 26 => TE::Transit(112), 28 => TE::Transit(113), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67) },
    hashmap! { 60 => TE::Shift(114), 61 => TE::Shift(115) },
    hashmap! { 60 => TE::Reduce(54), 61 => TE::Reduce(54), 65 => TE::Shift(74), 66 => TE::Shift(75), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 71 => TE::Shift(79) },
    hashmap! { 62 => TE::Reduce(52) },
    hashmap! { 28 => TE::Transit(116), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67) },
    hashmap! { 60 => TE::Reduce(53), 61 => TE::Reduce(53), 65 => TE::Shift(74), 66 => TE::Shift(75), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 71 => TE::Shift(79) },
    hashmap! { 34 => TE::Transit(118), 56 => TE::Shift(9) },
    hashmap! { 61 => TE::Shift(119) },
    hashmap! { 28 => TE::Transit(120), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67) },
    hashmap! { 60 => TE::Shift(121), 65 => TE::Shift(74), 66 => TE::Shift(75), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 71 => TE::Shift(79) },
    hashmap! { 62 => TE::Reduce(39) },
    hashmap! { 19 => TE::Transit(123), 33 => TE::Transit(125), 35 => TE::Shift(23), 48 => TE::Shift(124), 52 => TE::Shift(19), 53 => TE::Shift(20), 54 => TE::Shift(21), 55 => TE::Shift(22) },
    hashmap! { 34 => TE::Transit(126), 56 => TE::Shift(9) },
    hashmap! { 56 => TE::Reduce(41) },
    hashmap! { 56 => TE::Reduce(42), 69 => TE::Shift(27) },
    hashmap! { 47 => TE::Shift(127) },
    hashmap! { 28 => TE::Transit(128), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67) },
    hashmap! { 20 => TE::Transit(129), 39 => TE::Shift(130), 60 => TE::Reduce(44), 65 => TE::Shift(74), 66 => TE::Shift(75), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 71 => TE::Shift(79) },
    hashmap! { 60 => TE::Shift(131) },
    hashmap! { 28 => TE::Transit(133), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67) },
    hashmap! { 9 => TE::Transit(50), 11 => TE::Transit(132), 12 => TE::Transit(42), 13 => TE::Transit(43), 14 => TE::Transit(46), 15 => TE::Transit(41), 17 => TE::Transit(47), 18 => TE::Transit(48), 21 => TE::Transit(49), 24 => TE::Transit(44), 25 => TE::Transit(45), 27 => TE::Transit(40), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(39), 32 => TE::Transit(17), 33 => TE::Transit(32), 35 => TE::Shift(23), 39 => TE::Shift(56), 40 => TE::Shift(57), 42 => TE::Shift(60), 43 => TE::Shift(55), 45 => TE::Shift(61), 46 => TE::Shift(62), 48 => TE::Shift(52), 50 => TE::Shift(58), 51 => TE::Shift(59), 52 => TE::Shift(19), 53 => TE::Shift(20), 54 => TE::Shift(21), 55 => TE::Shift(22), 56 => TE::Reduce(67), 57 => TE::Shift(35), 62 => TE::Reduce(58) },
    hashmap! { 35 => TE::Reduce(40), 39 => TE::Reduce(40), 40 => TE::Reduce(40), 42 => TE::Reduce(40), 43 => TE::Reduce(40), 44 => TE::Reduce(40), 45 => TE::Reduce(40), 46 => TE::Reduce(40), 48 => TE::Reduce(40), 49 => TE::Reduce(40), 50 => TE::Reduce(40), 51 => TE::Reduce(40), 52 => TE::Reduce(40), 53 => TE::Reduce(40), 54 => TE::Reduce(40), 55 => TE::Reduce(40), 56 => TE::Reduce(40), 57 => TE::Reduce(40), 58 => TE::Reduce(40), 62 => TE::Reduce(40) },
    hashmap! { 60 => TE::Reduce(43), 65 => TE::Shift(74), 66 => TE::Shift(75), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 71 => TE::Shift(79) },
    hashmap! { 58 => TE::Shift(137) },
    hashmap! { 49 => TE::Shift(138), 58 => TE::Reduce(46) },
    hashmap! { 63 => TE::Shift(142), 65 => TE::Shift(74), 66 => TE::Shift(75), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 71 => TE::Shift(79) },
    hashmap! { 35 => TE::Reduce(45), 39 => TE::Reduce(45), 40 => TE::Reduce(45), 42 => TE::Reduce(45), 43 => TE::Reduce(45), 44 => TE::Reduce(45), 45 => TE::Reduce(45), 46 => TE::Reduce(45), 48 => TE::Reduce(45), 49 => TE::Reduce(45), 50 => TE::Reduce(45), 51 => TE::Reduce(45), 52 => TE::Reduce(45), 53 => TE::Reduce(45), 54 => TE::Reduce(45), 55 => TE::Reduce(45), 56 => TE::Reduce(45), 57 => TE::Reduce(45), 58 => TE::Reduce(45), 62 => TE::Reduce(45) },
    hashmap! { 28 => TE::Transit(139), 29 => TE::Transit(73), 30 => TE::Transit(54), 56 => TE::Reduce(67) },
    hashmap! { 63 => TE::Shift(140), 65 => TE::Shift(74), 66 => TE::Shift(75), 67 => TE::Shift(76), 68 => TE::Shift(77), 69 => TE::Shift(78), 71 => TE::Shift(79) },
    hashmap! { 9 => TE::Transit(50), 11 => TE::Transit(141), 12 => TE::Transit(42), 13 => TE::Transit(43), 14 => TE::Transit(46), 15 => TE::Transit(41), 17 => TE::Transit(47), 18 => TE::Transit(48), 21 => TE::Transit(49), 24 => TE::Transit(44), 25 => TE::Transit(45), 27 => TE::Transit(40), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(39), 32 => TE::Transit(17), 33 => TE::Transit(32), 35 => TE::Shift(23), 39 => TE::Shift(56), 40 => TE::Shift(57), 42 => TE::Shift(60), 43 => TE::Shift(55), 45 => TE::Shift(61), 46 => TE::Shift(62), 48 => TE::Shift(52), 50 => TE::Shift(58), 51 => TE::Shift(59), 52 => TE::Shift(19), 53 => TE::Shift(20), 54 => TE::Shift(21), 55 => TE::Shift(22), 56 => TE::Reduce(67), 57 => TE::Shift(35), 62 => TE::Reduce(58) },
    hashmap! { 49 => TE::Reduce(48), 58 => TE::Reduce(48) },
    hashmap! { 9 => TE::Transit(50), 11 => TE::Transit(143), 12 => TE::Transit(42), 13 => TE::Transit(43), 14 => TE::Transit(46), 15 => TE::Transit(41), 17 => TE::Transit(47), 18 => TE::Transit(48), 21 => TE::Transit(49), 24 => TE::Transit(44), 25 => TE::Transit(45), 27 => TE::Transit(40), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(39), 32 => TE::Transit(17), 33 => TE::Transit(32), 35 => TE::Shift(23), 39 => TE::Shift(56), 40 => TE::Shift(57), 42 => TE::Shift(60), 43 => TE::Shift(55), 45 => TE::Shift(61), 46 => TE::Shift(62), 48 => TE::Shift(52), 50 => TE::Shift(58), 51 => TE::Shift(59), 52 => TE::Shift(19), 53 => TE::Shift(20), 54 => TE::Shift(21), 55 => TE::Shift(22), 56 => TE::Reduce(67), 57 => TE::Shift(35), 62 => TE::Reduce(58) },
    hashmap! { 49 => TE::Reduce(49), 58 => TE::Reduce(49) },
    hashmap! { 32 => TE::Transit(145), 33 => TE::Transit(32), 35 => TE::Shift(23), 52 => TE::Shift(19), 53 => TE::Shift(20), 54 => TE::Shift(21), 55 => TE::Shift(22) },
    hashmap! { 60 => TE::Reduce(16), 61 => TE::Reduce(16) },
    hashmap! { 34 => TE::Transit(147), 56 => TE::Shift(9), 69 => TE::Shift(27) },
    hashmap! { 59 => TE::Shift(148) },
    hashmap! { 7 => TE::Transit(149), 8 => TE::Transit(30), 32 => TE::Transit(31), 33 => TE::Transit(32), 35 => TE::Shift(23), 52 => TE::Shift(19), 53 => TE::Shift(20), 54 => TE::Shift(21), 55 => TE::Shift(22), 60 => TE::Reduce(15) },
    hashmap! { 60 => TE::Shift(150) },
    hashmap! { 9 => TE::Transit(151), 57 => TE::Shift(35) },
    hashmap! { 35 => TE::Reduce(12), 38 => TE::Reduce(12), 52 => TE::Reduce(12), 53 => TE::Reduce(12), 54 => TE::Reduce(12), 55 => TE::Reduce(12), 58 => TE::Reduce(12) },
    hashmap! { 57 => TE::Reduce(7) }
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
        if let SemValue::$ty(v) = mem::replace(&mut $r.value, SemValue::None) {
            v
        } else {
            unreachable!()
        }
    });
}

macro_rules! get_ref {
    ($r:expr, $ty:ident) => (match &mut $r.value { SemValue::$ty(v) => v, _ => unreachable!() });
}

fn gen_binary(mut left: Sem, opt: Token, mut right: Sem, kind: Operator) -> Sem {
    Sem {
        loc: opt.get_loc(),
        value: SemValue::Expr(Expr::Binary(Binary {
            loc: opt.get_loc(),
            opt: kind,
            left: Box::new(get_move!(left, Expr)),
            right: Box::new(get_move!(right, Expr)),
        })),
    }
}

// Final result type returned from `parse` method call.
pub type TResult = Program;

#[derive(Debug)]
pub struct Sem {
    pub loc: Location,
    pub value: SemValue,
}

#[derive(Debug)]
pub enum SemValue {
    IntLiteral(i32),
    BoolLiteral(bool),
    StringLiteral(String),
    Identifier(String),
    ClassList(Vec<ClassDef>),
    FieldList(Vec<FieldDef>),
    VarDefList(Vec<VarDef>),
    StatementList(Vec<Statement>),
    ExprList(Vec<Expr>),
    GuardedList(Vec<(Expr, Statement)>),
    ClassDef(ClassDef),
    VarDef(VarDef),
    MethodDef(MethodDef),
    Type(Type),
    Statement(Statement),
    Block(Block),
    Expr(Expr),
    LValue(LValue),
    Sealed(bool),
    Static(bool),
    None,
}

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

    handlers: [fn(&mut Tokenizer) -> &'static str; 71],
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
    Tokenizer::_lex_rule46,
    Tokenizer::_lex_rule47,
    Tokenizer::_lex_rule48,
    Tokenizer::_lex_rule49,
    Tokenizer::_lex_rule50,
    Tokenizer::_lex_rule51,
    Tokenizer::_lex_rule52,
    Tokenizer::_lex_rule53,
    Tokenizer::_lex_rule54,
    Tokenizer::_lex_rule55,
    Tokenizer::_lex_rule56,
    Tokenizer::_lex_rule57,
    Tokenizer::_lex_rule58,
    Tokenizer::_lex_rule59,
    Tokenizer::_lex_rule60,
    Tokenizer::_lex_rule61,
    Tokenizer::_lex_rule62,
    Tokenizer::_lex_rule63,
    Tokenizer::_lex_rule64,
    Tokenizer::_lex_rule65,
    Tokenizer::_lex_rule66,
    Tokenizer::_lex_rule67,
    Tokenizer::_lex_rule68,
    Tokenizer::_lex_rule69,
    Tokenizer::_lex_rule70
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
return "'+'";
}

fn _lex_rule36(&mut self) -> &'static str {
return "'-'";
}

fn _lex_rule37(&mut self) -> &'static str {
return "'*'";
}

fn _lex_rule38(&mut self) -> &'static str {
return "'%'";
}

fn _lex_rule39(&mut self) -> &'static str {
return "'='";
}

fn _lex_rule40(&mut self) -> &'static str {
return "'<'";
}

fn _lex_rule41(&mut self) -> &'static str {
return "'>'";
}

fn _lex_rule42(&mut self) -> &'static str {
return "'.'";
}

fn _lex_rule43(&mut self) -> &'static str {
return "','";
}

fn _lex_rule44(&mut self) -> &'static str {
return "';'";
}

fn _lex_rule45(&mut self) -> &'static str {
return "'!'";
}

fn _lex_rule46(&mut self) -> &'static str {
return "'('";
}

fn _lex_rule47(&mut self) -> &'static str {
return "')'";
}

fn _lex_rule48(&mut self) -> &'static str {
return "'['";
}

fn _lex_rule49(&mut self) -> &'static str {
return "']'";
}

fn _lex_rule50(&mut self) -> &'static str {
return "'{'";
}

fn _lex_rule51(&mut self) -> &'static str {
return "'}'";
}

fn _lex_rule52(&mut self) -> &'static str {
return "':'";
}

fn _lex_rule53(&mut self) -> &'static str {
return "";
}

fn _lex_rule54(&mut self) -> &'static str {
return "NUMBER";
}

fn _lex_rule55(&mut self) -> &'static str {
return "IDENTIFIER";
}

fn _lex_rule56(&mut self) -> &'static str {
return "'{'";
}

fn _lex_rule57(&mut self) -> &'static str {
return "'}'";
}

fn _lex_rule58(&mut self) -> &'static str {
return "'('";
}

fn _lex_rule59(&mut self) -> &'static str {
return "')'";
}

fn _lex_rule60(&mut self) -> &'static str {
return "','";
}

fn _lex_rule61(&mut self) -> &'static str {
return "';'";
}

fn _lex_rule62(&mut self) -> &'static str {
return "':'";
}

fn _lex_rule63(&mut self) -> &'static str {
return "'='";
}

fn _lex_rule64(&mut self) -> &'static str {
return "'+'";
}

fn _lex_rule65(&mut self) -> &'static str {
return "'-'";
}

fn _lex_rule66(&mut self) -> &'static str {
return "'*'";
}

fn _lex_rule67(&mut self) -> &'static str {
return "'%'";
}

fn _lex_rule68(&mut self) -> &'static str {
return "'['";
}

fn _lex_rule69(&mut self) -> &'static str {
return "']'";
}

fn _lex_rule70(&mut self) -> &'static str {
return "'.'";
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
    handlers: [fn(&mut Parser) -> SV; 77],
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
    Parser::_handler12,
    Parser::_handler13,
    Parser::_handler14,
    Parser::_handler15,
    Parser::_handler16,
    Parser::_handler17,
    Parser::_handler18,
    Parser::_handler19,
    Parser::_handler20,
    Parser::_handler21,
    Parser::_handler22,
    Parser::_handler23,
    Parser::_handler24,
    Parser::_handler25,
    Parser::_handler26,
    Parser::_handler27,
    Parser::_handler28,
    Parser::_handler29,
    Parser::_handler30,
    Parser::_handler31,
    Parser::_handler32,
    Parser::_handler33,
    Parser::_handler34,
    Parser::_handler35,
    Parser::_handler36,
    Parser::_handler37,
    Parser::_handler38,
    Parser::_handler39,
    Parser::_handler40,
    Parser::_handler41,
    Parser::_handler42,
    Parser::_handler43,
    Parser::_handler44,
    Parser::_handler45,
    Parser::_handler46,
    Parser::_handler47,
    Parser::_handler48,
    Parser::_handler49,
    Parser::_handler50,
    Parser::_handler51,
    Parser::_handler52,
    Parser::_handler53,
    Parser::_handler54,
    Parser::_handler55,
    Parser::_handler56,
    Parser::_handler57,
    Parser::_handler58,
    Parser::_handler59,
    Parser::_handler60,
    Parser::_handler61,
    Parser::_handler62,
    Parser::_handler63,
    Parser::_handler64,
    Parser::_handler65,
    Parser::_handler66,
    Parser::_handler67,
    Parser::_handler68,
    Parser::_handler69,
    Parser::_handler70,
    Parser::_handler71,
    Parser::_handler72,
    Parser::_handler73,
    Parser::_handler74,
    Parser::_handler75,
    Parser::_handler76
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

                    let result = get_result!(parsed, _2);
                    
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
let mut _1 = pop!(self.values_stack, _1);

let __ = Program {
            classes: get_move!(_1, ClassList),
        };
SV::_2(__)
}

fn _handler2(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _1);

let mut ret = _1;
        get_ref!(ret, ClassList).push(get_move!(_2, ClassDef));
        let __ = ret;
SV::_1(__)
}

fn _handler3(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::ClassList(vec![get_move!(_1, ClassDef)]),
        };
SV::_1(__)
}

fn _handler4(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _6 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _4 = pop!(self.values_stack, _1);
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: _2.get_loc(),
            value: SemValue::ClassDef(ClassDef {
                loc: _2.get_loc(),
                name: get_move!(_3, Identifier),
                parent: match _4.value {
                    SemValue::Identifier(name) => Some(name),
                    SemValue::None => None,
                    _ => unreachable!(),
                },
                fields: get_move!(_6, FieldList),
                sealed: get_move!(_1, Sealed),
            })
        };
SV::_1(__)
}

fn _handler5(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();

let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::Sealed(true),
        };
SV::_1(__)
}

fn _handler6(&mut self) -> SV {
// Semantic values prologue.


let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::Sealed(false),
        };
SV::_1(__)
}

fn _handler7(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
self.values_stack.pop();

let __ = Sem {
            loc: _2.loc,
            value: SemValue::Identifier(get_move!(_2, Identifier)),
        };
SV::_1(__)
}

fn _handler8(&mut self) -> SV {
// Semantic values prologue.


let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::None,
        };
SV::_1(__)
}

fn _handler9(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _1);

let mut ret = _1;
        get_ref!(ret, FieldList).push(FieldDef::VarDef(get_move!(_2, VarDef)));
        let __ = ret;
SV::_1(__)
}

fn _handler10(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _1);

let mut ret = _1;
        get_ref!(ret, FieldList).push(FieldDef::MethodDef(get_move!(_2, MethodDef)));
        let __ = ret;
SV::_1(__)
}

fn _handler11(&mut self) -> SV {
// Semantic values prologue.


let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::FieldList(Vec::new())
        };
SV::_1(__)
}

fn _handler12(&mut self) -> SV {
// Semantic values prologue.
let mut _7 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _5 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _3.loc,
            value: SemValue::MethodDef(MethodDef {
                loc: _3.loc,
                name: get_move!(_3, Identifier),
                return_type: get_move!(_2, Type),
                parameters: get_move!(_5, VarDefList),
                static_: true,
                body: get_move!(_7, Block),
            })
        };
SV::_1(__)
}

fn _handler13(&mut self) -> SV {
// Semantic values prologue.
let mut _6 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _4 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: _2.loc,
            value: SemValue::MethodDef(MethodDef {
                loc: _2.loc,
                name: get_move!(_2, Identifier),
                return_type: get_move!(_1, Type),
                parameters: get_move!(_4, VarDefList),
                static_: false,
                body: get_move!(_6, Block),
            })
        };
SV::_1(__)
}

fn _handler14(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler15(&mut self) -> SV {
// Semantic values prologue.


let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::VarDefList(Vec::new()),
        };
SV::_1(__)
}

fn _handler16(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _1);

let mut ret = _1;
        get_ref!(ret, VarDefList).push(get_move!(_3, VarDef));
        let __ = ret;
SV::_1(__)
}

fn _handler17(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::VarDefList(vec!(get_move!(_1, VarDef))),
        };
SV::_1(__)
}

fn _handler18(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Block(Block {
                loc: _1.get_loc(),
                statements: get_move!(_2, StatementList),
            }),
        };
SV::_1(__)
}

fn _handler19(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _1);

let mut ret = _1;
        get_ref!(ret, StatementList).push(get_move!(_2, Statement));
        let __ = ret;
SV::_1(__)
}

fn _handler20(&mut self) -> SV {
// Semantic values prologue.


let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::StatementList(Vec::new()),
        };
SV::_1(__)
}

fn _handler21(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: _1.loc,
            value: SemValue::Statement(Statement::VarDef(get_move!(_1, VarDef))),
        };
SV::_1(__)
}

fn _handler22(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler23(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler24(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler25(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler26(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler27(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler28(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler29(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler30(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler31(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler32(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: _1.loc,
            value: SemValue::Statement(Statement::Block(get_move!(_1, Block))),
        };
SV::_1(__)
}

fn _handler33(&mut self) -> SV {
// Semantic values prologue.
let mut _5 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Statement(Statement::While(While {
                loc: _1.get_loc(),
                cond: get_move!(_3, Expr),
                body: Box::new(get_move!(_5, Statement)),
            })),
        };
SV::_1(__)
}

fn _handler34(&mut self) -> SV {
// Semantic values prologue.
let mut _9 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _7 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _5 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Statement(Statement::For(For {
                loc: _1.get_loc(),
                init: match get_move!(_3, Statement) {
                    Statement::Simple(simple) => simple,
                    _ => unreachable!(),
                },
                cond: get_move!(_5, Expr),
                update: match get_move!(_7, Statement) {
                    Statement::Simple(simple) => simple,
                    _ => unreachable!(),
                },
                body: Box::new(get_move!(_9, Statement)),
            })),
        };
SV::_1(__)
}

fn _handler35(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Statement(Statement::Break(Break {
                loc: _1.get_loc(),
            })),
        };
SV::_1(__)
}

fn _handler36(&mut self) -> SV {
// Semantic values prologue.
let mut _6 = pop!(self.values_stack, _1);
let mut _5 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Statement(Statement::If(If {
                loc: _1.get_loc(),
                cond: get_move!(_3, Expr),
                on_true: Box::new(get_move!(_5, Statement)),
                on_false: match _6.value {
                    SemValue::Statement(statement) => {Some(Box::new(statement))}
                    SemValue::None => {None}
                    _ => unreachable!(),
                },
            })),
        };
SV::_1(__)
}

fn _handler37(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Statement(get_move!(_2, Statement)),
        };
SV::_1(__)
}

fn _handler38(&mut self) -> SV {
// Semantic values prologue.


let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::None,
        };
SV::_1(__)
}

fn _handler39(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _5 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Statement(Statement::ObjectCopy(ObjectCopy {
                loc: _1.get_loc(),
                dst: get_move!(_3, Identifier),
                src: get_move!(_5, Expr),
            })),
        };
SV::_1(__)
}

fn _handler40(&mut self) -> SV {
// Semantic values prologue.
let mut _9 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _7 = pop!(self.values_stack, _1);
let mut _6 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _4 = pop!(self.values_stack, _1);
let mut _3 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Statement(Statement::Foreach(Foreach {
                loc: _1.get_loc(),
                type_: get_move!(_3, Type),
                name: get_move!(_4, Identifier),
                array: get_move!(_6, Expr),
                cond: match _7.value {
                    SemValue::Expr(expr) => Some(expr),
                    SemValue::None => None,
                    _ => unreachable!(),
                },
                body: Box::new(get_move!(_9, Statement)),
            })),
        };
SV::_1(__)
}

fn _handler41(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Type(Type::Var),
        };
SV::_1(__)
}

fn _handler42(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: _1.loc,
            value: SemValue::Type(get_move!(_1, Type)),
        };
SV::_1(__)
}

fn _handler43(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Expr(get_move!(_2, Expr)),
        };
SV::_1(__)
}

fn _handler44(&mut self) -> SV {
// Semantic values prologue.


let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::None,
        };
SV::_1(__)
}

fn _handler45(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Statement(Statement::Guarded(Guarded {
                loc: _1.get_loc(),
                guarded: get_move!(_3, GuardedList),
            }))
        };
SV::_1(__)
}

fn _handler46(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler47(&mut self) -> SV {
// Semantic values prologue.


let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::GuardedList(Vec::new()),
        };
SV::_1(__)
}

fn _handler48(&mut self) -> SV {
// Semantic values prologue.
let mut _5 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _1);

let mut ret = _1;
        get_ref!(ret, GuardedList).push((get_move!(_3, Expr), get_move!(_5, Statement)));
        let __ = ret;
SV::_1(__)
}

fn _handler49(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::GuardedList(vec![(get_move!(_1, Expr), get_move!(_3, Statement))]),
        };
SV::_1(__)
}

fn _handler50(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Statement(Statement::Return(Return {
                loc: _1.get_loc(),
                expr: Some(get_move!(_2, Expr)),
            }))
        };
SV::_1(__)
}

fn _handler51(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Statement(Statement::Return(Return {
                loc: _1.get_loc(),
                expr: None,
            }))
        };
SV::_1(__)
}

fn _handler52(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Statement(Statement::Print(Print {
                loc: _1.get_loc(),
                print: get_move!(_3, ExprList),
            }))
        };
SV::_1(__)
}

fn _handler53(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _1);

let mut ret = _1;
        get_ref!(ret, ExprList).push(get_move!(_3, Expr));
        let __ = ret;
SV::_1(__)
}

fn _handler54(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::ExprList(vec!(get_move!(_1, Expr))),
        };
SV::_1(__)
}

fn _handler55(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: _2.get_loc(),
            value: SemValue::Statement(Statement::Simple(Simple::Assign(Assign {
                loc: _2.get_loc(),
                dst: get_move!(_1, LValue),
                src: get_move!(_3, Expr),
            }))),
        };
SV::_1(__)
}

fn _handler56(&mut self) -> SV {
// Semantic values prologue.
let mut _4 = pop!(self.values_stack, _1);
let mut _3 = pop!(self.values_stack, _0);
let mut _2 = pop!(self.values_stack, _1);
self.values_stack.pop();

let __ = Sem {
            loc: _3.get_loc(),
            value: SemValue::Statement(Statement::Simple(Simple::VarAssign(VarAssign {
                loc: _3.get_loc(),
                name: get_move!(_2, Identifier),
                src: get_move!(_4, Expr),
            }))),
        };
SV::_1(__)
}

fn _handler57(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: _1.loc,
            value: SemValue::Statement(Statement::Simple(Simple::Expr(get_move!(_1, Expr)))),
        };
SV::_1(__)
}

fn _handler58(&mut self) -> SV {
// Semantic values prologue.


let __ = Sem {
            loc: self.get_loc(),
            value: SemValue::Statement(Statement::Simple(Simple::Skip(Skip {
                loc: self.get_loc(),
            }))),
        };
SV::_1(__)
}

fn _handler59(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: _1.loc,
            value: SemValue::Expr(Expr::LValue(get_move!(_1, LValue))),
        };
SV::_1(__)
}

fn _handler60(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Add);
SV::_1(__)
}

fn _handler61(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Sub);
SV::_1(__)
}

fn _handler62(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Mul);
SV::_1(__)
}

fn _handler63(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Mod);
SV::_1(__)
}

fn _handler64(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: _2.loc,
            value: SemValue::LValue(LValue::Identifier(Identifier {
                loc: _2.loc,
                owner: match _1.value {
                    SemValue::Expr(expr) => {Some(Box::new(expr))}
                    SemValue::None => {None}
                    _ => unreachable!(),
                },
                name: get_move!(_2, Identifier),
            })),
        };
SV::_1(__)
}

fn _handler65(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: _1.loc,
            value: SemValue::LValue(LValue::Indexed(Indexed {
                loc: _1.loc,
                array: Box::new(get_move!(_1, Expr)),
                index: Box::new(get_move!(_3, Expr)),
            }))
        };
SV::_1(__)
}

fn _handler66(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler67(&mut self) -> SV {
// Semantic values prologue.


let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::None,
        };
SV::_1(__)
}

fn _handler68(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = self.values_stack.pop().unwrap();

let __ = _1;
__
}

fn _handler69(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: _2.loc,
            value: SemValue::VarDef(VarDef {
                loc: _2.loc,
                name: get_move!(_2, Identifier),
                type_: get_move!(_1, Type),
            })
        };
SV::_1(__)
}

fn _handler70(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Type(Type::Basic("int")),
        };
SV::_1(__)
}

fn _handler71(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Type(Type::Basic("void")),
        };
SV::_1(__)
}

fn _handler72(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Type(Type::Basic("bool")),
        };
SV::_1(__)
}

fn _handler73(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Type(Type::Basic("string")),
        };
SV::_1(__)
}

fn _handler74(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Type(Type::Class(get_move!(_2, Identifier))),
        };
SV::_1(__)
}

fn _handler75(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: _1.loc,
            value: SemValue::Type(Type::Array(Some(Box::new(get_move!(_1, Type))))),
        };
SV::_1(__)
}

fn _handler76(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            // self.tokenizer.yytext.to_string() return s the current name
            value: SemValue::Identifier(self.tokenizer.yytext.to_string()),
        };
SV::_1(__)
}
}
