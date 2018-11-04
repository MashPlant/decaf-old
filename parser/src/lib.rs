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
static LEX_RULES: [&'static str; 79] = [
    r"^void",
    r"^int",
    r"^bool",
    r"^string",
    r"^new",
    r"^null",
    r"^true",
    r"^false",
    r"^class",
    r"^extends",
    r"^this",
    r"^while",
    r"^foreach",
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
    r"^\|\|\|",
    r"^<=",
    r"^>=",
    r"^==",
    r"^!=",
    r"^&&",
    r"^\|\|",
    r"^%%",
    r"^\+\+",
    r"^\+",
    r"^-",
    r"^\*",
    r"^/",
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
    "^\"[^\"]*\"",
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
    r"^/",
    r"^%",
    r"^<",
    r"^>",
    r"^\[",
    r"^\]",
    r"^!",
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
static PRODUCTIONS : [[i32; 2]; 117] = [
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
    [28, 1],
    [28, 1],
    [28, 3],
    [28, 3],
    [28, 3],
    [28, 3],
    [28, 3],
    [28, 3],
    [28, 3],
    [28, 3],
    [28, 3],
    [28, 3],
    [28, 3],
    [28, 3],
    [28, 3],
    [28, 3],
    [28, 3],
    [28, 6],
    [28, 6],
    [28, 7],
    [28, 9],
    [28, 3],
    [28, 2],
    [28, 2],
    [28, 3],
    [28, 3],
    [28, 1],
    [28, 4],
    [28, 5],
    [28, 6],
    [28, 5],
    [29, 2],
    [29, 4],
    [30, 2],
    [30, 0],
    [31, 5],
    [32, 1],
    [32, 1],
    [32, 1],
    [32, 1],
    [32, 1],
    [32, 1],
    [33, 3],
    [33, 2],
    [34, 3],
    [34, 1],
    [35, 1],
    [35, 0],
    [36, 2],
    [37, 2],
    [38, 1],
    [38, 1],
    [38, 1],
    [38, 1],
    [38, 2],
    [38, 3],
    [39, 1]
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
    static ref LEX_RULES_BY_START_CONDITIONS: HashMap<&'static str, Vec<i32>> = hashmap! { "INITIAL" => vec! [ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78 ] };

    /**
     * Maps a string name of a token type to its encoded number (the first
     * token number starts after all numbers for non-terminal).
     */
    static ref TOKENS_MAP: HashMap<&'static str, i32> = hashmap! { "CLASS" => 40, "SEALED" => 41, "EXTENDS" => 42, "STATIC" => 43, "WHILE" => 44, "FOR" => 45, "BREAK" => 46, "IF" => 47, "ELSE" => 48, "SCOPY" => 49, "FOREACH" => 50, "IN" => 51, "VAR" => 52, "GUARD_SPLIT" => 53, "RETURN" => 54, "PRINT" => 55, "EQUAL" => 56, "NOT_EQUAL" => 57, "LESS_EQUAL" => 58, "GREATER_EQUAL" => 59, "AND" => 60, "OR" => 61, "REPEAT" => 62, "CONCAT" => 63, "DEFAULT" => 64, "READ_INTEGER" => 65, "READ_LINE" => 66, "THIS" => 67, "NEW" => 68, "INSTANCEOF" => 69, "INT_CONST" => 70, "TRUE" => 71, "FALSE" => 72, "STRING_CONST" => 73, "NULL" => 74, "INT" => 75, "VOID" => 76, "BOOL" => 77, "STRING" => 78, "IDENTIFIER" => 79, "'{'" => 80, "'}'" => 81, "'('" => 82, "')'" => 83, "','" => 84, "';'" => 85, "':'" => 86, "'='" => 87, "'+'" => 88, "'-'" => 89, "'*'" => 90, "'/'" => 91, "'%'" => 92, "'<'" => 93, "'>'" => 94, "'['" => 95, "']'" => 96, "'!'" => 97, "'.'" => 98, "$" => 99 };

    static ref REGEX_RULES: Vec<Regex> = LEX_RULES.iter().map(|rule| Regex::new(rule).unwrap()).collect();
    /**
     * Parsing table.
     *
     * Vector index is the state number, value is a map
     * from an encoded symbol to table entry (TE).
     */
    static ref TABLE: Vec<HashMap<i32, TE>>= vec![
    hashmap! { 0 => TE::Transit(1), 1 => TE::Transit(2), 2 => TE::Transit(3), 3 => TE::Transit(4), 40 => TE::Reduce(6), 41 => TE::Shift(5) },
    hashmap! { 99 => TE::Accept },
    hashmap! { 2 => TE::Transit(6), 3 => TE::Transit(4), 40 => TE::Reduce(6), 41 => TE::Shift(5), 99 => TE::Reduce(1) },
    hashmap! { 40 => TE::Reduce(3), 41 => TE::Reduce(3), 99 => TE::Reduce(3) },
    hashmap! { 40 => TE::Shift(7) },
    hashmap! { 40 => TE::Reduce(5) },
    hashmap! { 40 => TE::Reduce(2), 41 => TE::Reduce(2), 99 => TE::Reduce(2) },
    hashmap! { 39 => TE::Transit(8), 79 => TE::Shift(9) },
    hashmap! { 4 => TE::Transit(10), 42 => TE::Shift(11), 80 => TE::Reduce(8) },
    hashmap! { 42 => TE::Reduce(116), 44 => TE::Reduce(116), 45 => TE::Reduce(116), 47 => TE::Reduce(116), 51 => TE::Reduce(116), 56 => TE::Reduce(116), 57 => TE::Reduce(116), 58 => TE::Reduce(116), 59 => TE::Reduce(116), 60 => TE::Reduce(116), 61 => TE::Reduce(116), 62 => TE::Reduce(116), 63 => TE::Reduce(116), 79 => TE::Reduce(116), 80 => TE::Reduce(116), 82 => TE::Reduce(116), 83 => TE::Reduce(116), 84 => TE::Reduce(116), 85 => TE::Reduce(116), 86 => TE::Reduce(116), 87 => TE::Reduce(116), 88 => TE::Reduce(116), 89 => TE::Reduce(116), 90 => TE::Reduce(116), 91 => TE::Reduce(116), 92 => TE::Reduce(116), 93 => TE::Reduce(116), 94 => TE::Reduce(116), 95 => TE::Reduce(116), 96 => TE::Reduce(116), 98 => TE::Reduce(116) },
    hashmap! { 80 => TE::Shift(12) },
    hashmap! { 39 => TE::Transit(241), 79 => TE::Shift(9) },
    hashmap! { 5 => TE::Transit(13), 40 => TE::Reduce(11), 43 => TE::Reduce(11), 75 => TE::Reduce(11), 76 => TE::Reduce(11), 77 => TE::Reduce(11), 78 => TE::Reduce(11), 81 => TE::Reduce(11) },
    hashmap! { 6 => TE::Transit(16), 36 => TE::Transit(15), 37 => TE::Transit(17), 38 => TE::Transit(18), 40 => TE::Shift(23), 43 => TE::Shift(24), 75 => TE::Shift(19), 76 => TE::Shift(20), 77 => TE::Shift(21), 78 => TE::Shift(22), 81 => TE::Shift(14) },
    hashmap! { 40 => TE::Reduce(4), 41 => TE::Reduce(4), 99 => TE::Reduce(4) },
    hashmap! { 40 => TE::Reduce(9), 43 => TE::Reduce(9), 75 => TE::Reduce(9), 76 => TE::Reduce(9), 77 => TE::Reduce(9), 78 => TE::Reduce(9), 81 => TE::Reduce(9) },
    hashmap! { 40 => TE::Reduce(10), 43 => TE::Reduce(10), 75 => TE::Reduce(10), 76 => TE::Reduce(10), 77 => TE::Reduce(10), 78 => TE::Reduce(10), 81 => TE::Reduce(10) },
    hashmap! { 85 => TE::Shift(25) },
    hashmap! { 39 => TE::Transit(26), 79 => TE::Shift(9), 95 => TE::Shift(27) },
    hashmap! { 79 => TE::Reduce(110), 95 => TE::Reduce(110) },
    hashmap! { 79 => TE::Reduce(111), 95 => TE::Reduce(111) },
    hashmap! { 79 => TE::Reduce(112), 95 => TE::Reduce(112) },
    hashmap! { 79 => TE::Reduce(113), 95 => TE::Reduce(113) },
    hashmap! { 39 => TE::Transit(87), 79 => TE::Shift(9) },
    hashmap! { 38 => TE::Transit(235), 40 => TE::Shift(23), 75 => TE::Shift(19), 76 => TE::Shift(20), 77 => TE::Shift(21), 78 => TE::Shift(22) },
    hashmap! { 40 => TE::Reduce(108), 43 => TE::Reduce(108), 44 => TE::Reduce(108), 45 => TE::Reduce(108), 46 => TE::Reduce(108), 47 => TE::Reduce(108), 48 => TE::Reduce(108), 49 => TE::Reduce(108), 50 => TE::Reduce(108), 52 => TE::Reduce(108), 53 => TE::Reduce(108), 54 => TE::Reduce(108), 55 => TE::Reduce(108), 65 => TE::Reduce(108), 66 => TE::Reduce(108), 67 => TE::Reduce(108), 68 => TE::Reduce(108), 69 => TE::Reduce(108), 70 => TE::Reduce(108), 71 => TE::Reduce(108), 72 => TE::Reduce(108), 73 => TE::Reduce(108), 74 => TE::Reduce(108), 75 => TE::Reduce(108), 76 => TE::Reduce(108), 77 => TE::Reduce(108), 78 => TE::Reduce(108), 79 => TE::Reduce(108), 80 => TE::Reduce(108), 81 => TE::Reduce(108), 82 => TE::Reduce(108), 85 => TE::Reduce(108), 89 => TE::Reduce(108), 95 => TE::Reduce(108), 97 => TE::Reduce(108) },
    hashmap! { 82 => TE::Shift(28), 85 => TE::Reduce(109) },
    hashmap! { 96 => TE::Shift(86) },
    hashmap! { 7 => TE::Transit(29), 8 => TE::Transit(30), 37 => TE::Transit(31), 38 => TE::Transit(32), 40 => TE::Shift(23), 75 => TE::Shift(19), 76 => TE::Shift(20), 77 => TE::Shift(21), 78 => TE::Shift(22), 83 => TE::Reduce(15) },
    hashmap! { 83 => TE::Shift(33) },
    hashmap! { 83 => TE::Reduce(14), 84 => TE::Shift(233) },
    hashmap! { 83 => TE::Reduce(17), 84 => TE::Reduce(17) },
    hashmap! { 39 => TE::Transit(85), 79 => TE::Shift(9), 95 => TE::Shift(27) },
    hashmap! { 9 => TE::Transit(34), 80 => TE::Shift(35) },
    hashmap! { 40 => TE::Reduce(13), 43 => TE::Reduce(13), 75 => TE::Reduce(13), 76 => TE::Reduce(13), 77 => TE::Reduce(13), 78 => TE::Reduce(13), 81 => TE::Reduce(13) },
    hashmap! { 10 => TE::Transit(36), 40 => TE::Reduce(20), 44 => TE::Reduce(20), 45 => TE::Reduce(20), 46 => TE::Reduce(20), 47 => TE::Reduce(20), 49 => TE::Reduce(20), 50 => TE::Reduce(20), 52 => TE::Reduce(20), 54 => TE::Reduce(20), 55 => TE::Reduce(20), 65 => TE::Reduce(20), 66 => TE::Reduce(20), 67 => TE::Reduce(20), 68 => TE::Reduce(20), 69 => TE::Reduce(20), 70 => TE::Reduce(20), 71 => TE::Reduce(20), 72 => TE::Reduce(20), 73 => TE::Reduce(20), 74 => TE::Reduce(20), 75 => TE::Reduce(20), 76 => TE::Reduce(20), 77 => TE::Reduce(20), 78 => TE::Reduce(20), 79 => TE::Reduce(20), 80 => TE::Reduce(20), 81 => TE::Reduce(20), 82 => TE::Reduce(20), 85 => TE::Reduce(20), 89 => TE::Reduce(20), 95 => TE::Reduce(20), 97 => TE::Reduce(20) },
    hashmap! { 9 => TE::Transit(50), 11 => TE::Transit(38), 12 => TE::Transit(42), 13 => TE::Transit(43), 14 => TE::Transit(46), 15 => TE::Transit(41), 17 => TE::Transit(47), 18 => TE::Transit(48), 21 => TE::Transit(49), 24 => TE::Transit(44), 25 => TE::Transit(45), 27 => TE::Transit(40), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 36 => TE::Transit(39), 37 => TE::Transit(17), 38 => TE::Transit(32), 40 => TE::Shift(23), 44 => TE::Shift(73), 45 => TE::Shift(74), 46 => TE::Shift(77), 47 => TE::Shift(72), 49 => TE::Shift(78), 50 => TE::Shift(79), 52 => TE::Shift(52), 54 => TE::Shift(75), 55 => TE::Shift(76), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 75 => TE::Shift(19), 76 => TE::Shift(20), 77 => TE::Shift(21), 78 => TE::Shift(22), 79 => TE::Reduce(94), 80 => TE::Shift(35), 81 => TE::Shift(37), 82 => TE::Shift(58), 85 => TE::Reduce(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 40 => TE::Reduce(18), 43 => TE::Reduce(18), 44 => TE::Reduce(18), 45 => TE::Reduce(18), 46 => TE::Reduce(18), 47 => TE::Reduce(18), 48 => TE::Reduce(18), 49 => TE::Reduce(18), 50 => TE::Reduce(18), 52 => TE::Reduce(18), 53 => TE::Reduce(18), 54 => TE::Reduce(18), 55 => TE::Reduce(18), 65 => TE::Reduce(18), 66 => TE::Reduce(18), 67 => TE::Reduce(18), 68 => TE::Reduce(18), 69 => TE::Reduce(18), 70 => TE::Reduce(18), 71 => TE::Reduce(18), 72 => TE::Reduce(18), 73 => TE::Reduce(18), 74 => TE::Reduce(18), 75 => TE::Reduce(18), 76 => TE::Reduce(18), 77 => TE::Reduce(18), 78 => TE::Reduce(18), 79 => TE::Reduce(18), 80 => TE::Reduce(18), 81 => TE::Reduce(18), 82 => TE::Reduce(18), 85 => TE::Reduce(18), 89 => TE::Reduce(18), 95 => TE::Reduce(18), 97 => TE::Reduce(18) },
    hashmap! { 40 => TE::Reduce(19), 44 => TE::Reduce(19), 45 => TE::Reduce(19), 46 => TE::Reduce(19), 47 => TE::Reduce(19), 49 => TE::Reduce(19), 50 => TE::Reduce(19), 52 => TE::Reduce(19), 54 => TE::Reduce(19), 55 => TE::Reduce(19), 65 => TE::Reduce(19), 66 => TE::Reduce(19), 67 => TE::Reduce(19), 68 => TE::Reduce(19), 69 => TE::Reduce(19), 70 => TE::Reduce(19), 71 => TE::Reduce(19), 72 => TE::Reduce(19), 73 => TE::Reduce(19), 74 => TE::Reduce(19), 75 => TE::Reduce(19), 76 => TE::Reduce(19), 77 => TE::Reduce(19), 78 => TE::Reduce(19), 79 => TE::Reduce(19), 80 => TE::Reduce(19), 81 => TE::Reduce(19), 82 => TE::Reduce(19), 85 => TE::Reduce(19), 89 => TE::Reduce(19), 95 => TE::Reduce(19), 97 => TE::Reduce(19) },
    hashmap! { 40 => TE::Reduce(21), 44 => TE::Reduce(21), 45 => TE::Reduce(21), 46 => TE::Reduce(21), 47 => TE::Reduce(21), 48 => TE::Reduce(21), 49 => TE::Reduce(21), 50 => TE::Reduce(21), 52 => TE::Reduce(21), 53 => TE::Reduce(21), 54 => TE::Reduce(21), 55 => TE::Reduce(21), 65 => TE::Reduce(21), 66 => TE::Reduce(21), 67 => TE::Reduce(21), 68 => TE::Reduce(21), 69 => TE::Reduce(21), 70 => TE::Reduce(21), 71 => TE::Reduce(21), 72 => TE::Reduce(21), 73 => TE::Reduce(21), 74 => TE::Reduce(21), 75 => TE::Reduce(21), 76 => TE::Reduce(21), 77 => TE::Reduce(21), 78 => TE::Reduce(21), 79 => TE::Reduce(21), 80 => TE::Reduce(21), 81 => TE::Reduce(21), 82 => TE::Reduce(21), 85 => TE::Reduce(21), 89 => TE::Reduce(21), 95 => TE::Reduce(21), 97 => TE::Reduce(21) },
    hashmap! { 85 => TE::Shift(80) },
    hashmap! { 40 => TE::Reduce(23), 44 => TE::Reduce(23), 45 => TE::Reduce(23), 46 => TE::Reduce(23), 47 => TE::Reduce(23), 48 => TE::Reduce(23), 49 => TE::Reduce(23), 50 => TE::Reduce(23), 52 => TE::Reduce(23), 53 => TE::Reduce(23), 54 => TE::Reduce(23), 55 => TE::Reduce(23), 65 => TE::Reduce(23), 66 => TE::Reduce(23), 67 => TE::Reduce(23), 68 => TE::Reduce(23), 69 => TE::Reduce(23), 70 => TE::Reduce(23), 71 => TE::Reduce(23), 72 => TE::Reduce(23), 73 => TE::Reduce(23), 74 => TE::Reduce(23), 75 => TE::Reduce(23), 76 => TE::Reduce(23), 77 => TE::Reduce(23), 78 => TE::Reduce(23), 79 => TE::Reduce(23), 80 => TE::Reduce(23), 81 => TE::Reduce(23), 82 => TE::Reduce(23), 85 => TE::Reduce(23), 89 => TE::Reduce(23), 95 => TE::Reduce(23), 97 => TE::Reduce(23) },
    hashmap! { 40 => TE::Reduce(24), 44 => TE::Reduce(24), 45 => TE::Reduce(24), 46 => TE::Reduce(24), 47 => TE::Reduce(24), 48 => TE::Reduce(24), 49 => TE::Reduce(24), 50 => TE::Reduce(24), 52 => TE::Reduce(24), 53 => TE::Reduce(24), 54 => TE::Reduce(24), 55 => TE::Reduce(24), 65 => TE::Reduce(24), 66 => TE::Reduce(24), 67 => TE::Reduce(24), 68 => TE::Reduce(24), 69 => TE::Reduce(24), 70 => TE::Reduce(24), 71 => TE::Reduce(24), 72 => TE::Reduce(24), 73 => TE::Reduce(24), 74 => TE::Reduce(24), 75 => TE::Reduce(24), 76 => TE::Reduce(24), 77 => TE::Reduce(24), 78 => TE::Reduce(24), 79 => TE::Reduce(24), 80 => TE::Reduce(24), 81 => TE::Reduce(24), 82 => TE::Reduce(24), 85 => TE::Reduce(24), 89 => TE::Reduce(24), 95 => TE::Reduce(24), 97 => TE::Reduce(24) },
    hashmap! { 40 => TE::Reduce(25), 44 => TE::Reduce(25), 45 => TE::Reduce(25), 46 => TE::Reduce(25), 47 => TE::Reduce(25), 48 => TE::Reduce(25), 49 => TE::Reduce(25), 50 => TE::Reduce(25), 52 => TE::Reduce(25), 53 => TE::Reduce(25), 54 => TE::Reduce(25), 55 => TE::Reduce(25), 65 => TE::Reduce(25), 66 => TE::Reduce(25), 67 => TE::Reduce(25), 68 => TE::Reduce(25), 69 => TE::Reduce(25), 70 => TE::Reduce(25), 71 => TE::Reduce(25), 72 => TE::Reduce(25), 73 => TE::Reduce(25), 74 => TE::Reduce(25), 75 => TE::Reduce(25), 76 => TE::Reduce(25), 77 => TE::Reduce(25), 78 => TE::Reduce(25), 79 => TE::Reduce(25), 80 => TE::Reduce(25), 81 => TE::Reduce(25), 82 => TE::Reduce(25), 85 => TE::Reduce(25), 89 => TE::Reduce(25), 95 => TE::Reduce(25), 97 => TE::Reduce(25) },
    hashmap! { 85 => TE::Shift(81) },
    hashmap! { 85 => TE::Shift(82) },
    hashmap! { 85 => TE::Shift(83) },
    hashmap! { 85 => TE::Shift(84) },
    hashmap! { 40 => TE::Reduce(30), 44 => TE::Reduce(30), 45 => TE::Reduce(30), 46 => TE::Reduce(30), 47 => TE::Reduce(30), 48 => TE::Reduce(30), 49 => TE::Reduce(30), 50 => TE::Reduce(30), 52 => TE::Reduce(30), 53 => TE::Reduce(30), 54 => TE::Reduce(30), 55 => TE::Reduce(30), 65 => TE::Reduce(30), 66 => TE::Reduce(30), 67 => TE::Reduce(30), 68 => TE::Reduce(30), 69 => TE::Reduce(30), 70 => TE::Reduce(30), 71 => TE::Reduce(30), 72 => TE::Reduce(30), 73 => TE::Reduce(30), 74 => TE::Reduce(30), 75 => TE::Reduce(30), 76 => TE::Reduce(30), 77 => TE::Reduce(30), 78 => TE::Reduce(30), 79 => TE::Reduce(30), 80 => TE::Reduce(30), 81 => TE::Reduce(30), 82 => TE::Reduce(30), 85 => TE::Reduce(30), 89 => TE::Reduce(30), 95 => TE::Reduce(30), 97 => TE::Reduce(30) },
    hashmap! { 40 => TE::Reduce(31), 44 => TE::Reduce(31), 45 => TE::Reduce(31), 46 => TE::Reduce(31), 47 => TE::Reduce(31), 48 => TE::Reduce(31), 49 => TE::Reduce(31), 50 => TE::Reduce(31), 52 => TE::Reduce(31), 53 => TE::Reduce(31), 54 => TE::Reduce(31), 55 => TE::Reduce(31), 65 => TE::Reduce(31), 66 => TE::Reduce(31), 67 => TE::Reduce(31), 68 => TE::Reduce(31), 69 => TE::Reduce(31), 70 => TE::Reduce(31), 71 => TE::Reduce(31), 72 => TE::Reduce(31), 73 => TE::Reduce(31), 74 => TE::Reduce(31), 75 => TE::Reduce(31), 76 => TE::Reduce(31), 77 => TE::Reduce(31), 78 => TE::Reduce(31), 79 => TE::Reduce(31), 80 => TE::Reduce(31), 81 => TE::Reduce(31), 82 => TE::Reduce(31), 85 => TE::Reduce(31), 89 => TE::Reduce(31), 95 => TE::Reduce(31), 97 => TE::Reduce(31) },
    hashmap! { 40 => TE::Reduce(32), 44 => TE::Reduce(32), 45 => TE::Reduce(32), 46 => TE::Reduce(32), 47 => TE::Reduce(32), 48 => TE::Reduce(32), 49 => TE::Reduce(32), 50 => TE::Reduce(32), 52 => TE::Reduce(32), 53 => TE::Reduce(32), 54 => TE::Reduce(32), 55 => TE::Reduce(32), 65 => TE::Reduce(32), 66 => TE::Reduce(32), 67 => TE::Reduce(32), 68 => TE::Reduce(32), 69 => TE::Reduce(32), 70 => TE::Reduce(32), 71 => TE::Reduce(32), 72 => TE::Reduce(32), 73 => TE::Reduce(32), 74 => TE::Reduce(32), 75 => TE::Reduce(32), 76 => TE::Reduce(32), 77 => TE::Reduce(32), 78 => TE::Reduce(32), 79 => TE::Reduce(32), 80 => TE::Reduce(32), 81 => TE::Reduce(32), 82 => TE::Reduce(32), 85 => TE::Reduce(32), 89 => TE::Reduce(32), 95 => TE::Reduce(32), 97 => TE::Reduce(32) },
    hashmap! { 56 => TE::Reduce(59), 57 => TE::Reduce(59), 58 => TE::Reduce(59), 59 => TE::Reduce(59), 60 => TE::Reduce(59), 61 => TE::Reduce(59), 62 => TE::Reduce(59), 63 => TE::Reduce(59), 83 => TE::Reduce(59), 85 => TE::Reduce(59), 87 => TE::Shift(88), 88 => TE::Reduce(59), 89 => TE::Reduce(59), 90 => TE::Reduce(59), 91 => TE::Reduce(59), 92 => TE::Reduce(59), 93 => TE::Reduce(59), 94 => TE::Reduce(59), 95 => TE::Reduce(59), 98 => TE::Reduce(59) },
    hashmap! { 39 => TE::Transit(179), 79 => TE::Shift(9) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Reduce(57), 85 => TE::Reduce(57), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 39 => TE::Transit(164), 79 => TE::Shift(9) },
    hashmap! { 44 => TE::Reduce(60), 45 => TE::Reduce(60), 47 => TE::Reduce(60), 56 => TE::Reduce(60), 57 => TE::Reduce(60), 58 => TE::Reduce(60), 59 => TE::Reduce(60), 60 => TE::Reduce(60), 61 => TE::Reduce(60), 62 => TE::Reduce(60), 63 => TE::Reduce(60), 83 => TE::Reduce(60), 84 => TE::Reduce(60), 85 => TE::Reduce(60), 86 => TE::Reduce(60), 88 => TE::Reduce(60), 89 => TE::Reduce(60), 90 => TE::Reduce(60), 91 => TE::Reduce(60), 92 => TE::Reduce(60), 93 => TE::Reduce(60), 94 => TE::Reduce(60), 95 => TE::Reduce(60), 96 => TE::Reduce(60), 98 => TE::Reduce(60) },
    hashmap! { 44 => TE::Reduce(61), 45 => TE::Reduce(61), 47 => TE::Reduce(61), 56 => TE::Reduce(61), 57 => TE::Reduce(61), 58 => TE::Reduce(61), 59 => TE::Reduce(61), 60 => TE::Reduce(61), 61 => TE::Reduce(61), 62 => TE::Reduce(61), 63 => TE::Reduce(61), 83 => TE::Reduce(61), 84 => TE::Reduce(61), 85 => TE::Reduce(61), 86 => TE::Reduce(61), 88 => TE::Reduce(61), 89 => TE::Reduce(61), 90 => TE::Reduce(61), 91 => TE::Reduce(61), 92 => TE::Reduce(61), 93 => TE::Reduce(61), 94 => TE::Reduce(61), 95 => TE::Reduce(61), 96 => TE::Reduce(61), 98 => TE::Reduce(61) },
    hashmap! { 28 => TE::Transit(128), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(131), 33 => TE::Transit(70), 34 => TE::Transit(129), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 96 => TE::Shift(130), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(140), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 40 => TE::Shift(141), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(146), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(147), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 82 => TE::Shift(148) },
    hashmap! { 82 => TE::Shift(150) },
    hashmap! { 44 => TE::Reduce(86), 45 => TE::Reduce(86), 47 => TE::Reduce(86), 56 => TE::Reduce(86), 57 => TE::Reduce(86), 58 => TE::Reduce(86), 59 => TE::Reduce(86), 60 => TE::Reduce(86), 61 => TE::Reduce(86), 62 => TE::Reduce(86), 63 => TE::Reduce(86), 83 => TE::Reduce(86), 84 => TE::Reduce(86), 85 => TE::Reduce(86), 86 => TE::Reduce(86), 88 => TE::Reduce(86), 89 => TE::Reduce(86), 90 => TE::Reduce(86), 91 => TE::Reduce(86), 92 => TE::Reduce(86), 93 => TE::Reduce(86), 94 => TE::Reduce(86), 95 => TE::Reduce(86), 96 => TE::Reduce(86), 98 => TE::Reduce(86) },
    hashmap! { 38 => TE::Transit(153), 39 => TE::Transit(152), 40 => TE::Shift(23), 75 => TE::Shift(19), 76 => TE::Shift(20), 77 => TE::Shift(21), 78 => TE::Shift(22), 79 => TE::Shift(9) },
    hashmap! { 82 => TE::Shift(159) },
    hashmap! { 44 => TE::Reduce(96), 45 => TE::Reduce(96), 47 => TE::Reduce(96), 56 => TE::Reduce(96), 57 => TE::Reduce(96), 58 => TE::Reduce(96), 59 => TE::Reduce(96), 60 => TE::Reduce(96), 61 => TE::Reduce(96), 62 => TE::Reduce(96), 63 => TE::Reduce(96), 83 => TE::Reduce(96), 84 => TE::Reduce(96), 85 => TE::Reduce(96), 86 => TE::Reduce(96), 88 => TE::Reduce(96), 89 => TE::Reduce(96), 90 => TE::Reduce(96), 91 => TE::Reduce(96), 92 => TE::Reduce(96), 93 => TE::Reduce(96), 94 => TE::Reduce(96), 95 => TE::Reduce(96), 96 => TE::Reduce(96), 98 => TE::Reduce(96) },
    hashmap! { 44 => TE::Reduce(97), 45 => TE::Reduce(97), 47 => TE::Reduce(97), 56 => TE::Reduce(97), 57 => TE::Reduce(97), 58 => TE::Reduce(97), 59 => TE::Reduce(97), 60 => TE::Reduce(97), 61 => TE::Reduce(97), 62 => TE::Reduce(97), 63 => TE::Reduce(97), 83 => TE::Reduce(97), 84 => TE::Reduce(97), 85 => TE::Reduce(97), 86 => TE::Reduce(97), 88 => TE::Reduce(97), 89 => TE::Reduce(97), 90 => TE::Reduce(97), 91 => TE::Reduce(97), 92 => TE::Reduce(97), 93 => TE::Reduce(97), 94 => TE::Reduce(97), 95 => TE::Reduce(97), 96 => TE::Reduce(97), 98 => TE::Reduce(97) },
    hashmap! { 44 => TE::Reduce(98), 45 => TE::Reduce(98), 47 => TE::Reduce(98), 56 => TE::Reduce(98), 57 => TE::Reduce(98), 58 => TE::Reduce(98), 59 => TE::Reduce(98), 60 => TE::Reduce(98), 61 => TE::Reduce(98), 62 => TE::Reduce(98), 63 => TE::Reduce(98), 83 => TE::Reduce(98), 84 => TE::Reduce(98), 85 => TE::Reduce(98), 86 => TE::Reduce(98), 88 => TE::Reduce(98), 89 => TE::Reduce(98), 90 => TE::Reduce(98), 91 => TE::Reduce(98), 92 => TE::Reduce(98), 93 => TE::Reduce(98), 94 => TE::Reduce(98), 95 => TE::Reduce(98), 96 => TE::Reduce(98), 98 => TE::Reduce(98) },
    hashmap! { 44 => TE::Reduce(99), 45 => TE::Reduce(99), 47 => TE::Reduce(99), 56 => TE::Reduce(99), 57 => TE::Reduce(99), 58 => TE::Reduce(99), 59 => TE::Reduce(99), 60 => TE::Reduce(99), 61 => TE::Reduce(99), 62 => TE::Reduce(99), 63 => TE::Reduce(99), 83 => TE::Reduce(99), 84 => TE::Reduce(99), 85 => TE::Reduce(99), 86 => TE::Reduce(99), 88 => TE::Reduce(99), 89 => TE::Reduce(99), 90 => TE::Reduce(99), 91 => TE::Reduce(99), 92 => TE::Reduce(99), 93 => TE::Reduce(99), 94 => TE::Reduce(99), 95 => TE::Reduce(99), 96 => TE::Reduce(99), 98 => TE::Reduce(99) },
    hashmap! { 44 => TE::Reduce(100), 45 => TE::Reduce(100), 47 => TE::Reduce(100), 56 => TE::Reduce(100), 57 => TE::Reduce(100), 58 => TE::Reduce(100), 59 => TE::Reduce(100), 60 => TE::Reduce(100), 61 => TE::Reduce(100), 62 => TE::Reduce(100), 63 => TE::Reduce(100), 83 => TE::Reduce(100), 84 => TE::Reduce(100), 85 => TE::Reduce(100), 86 => TE::Reduce(100), 88 => TE::Reduce(100), 89 => TE::Reduce(100), 90 => TE::Reduce(100), 91 => TE::Reduce(100), 92 => TE::Reduce(100), 93 => TE::Reduce(100), 94 => TE::Reduce(100), 95 => TE::Reduce(100), 96 => TE::Reduce(100), 98 => TE::Reduce(100) },
    hashmap! { 44 => TE::Reduce(101), 45 => TE::Reduce(101), 47 => TE::Reduce(101), 56 => TE::Reduce(101), 57 => TE::Reduce(101), 58 => TE::Reduce(101), 59 => TE::Reduce(101), 60 => TE::Reduce(101), 61 => TE::Reduce(101), 62 => TE::Reduce(101), 63 => TE::Reduce(101), 83 => TE::Reduce(101), 84 => TE::Reduce(101), 85 => TE::Reduce(101), 86 => TE::Reduce(101), 88 => TE::Reduce(101), 89 => TE::Reduce(101), 90 => TE::Reduce(101), 91 => TE::Reduce(101), 92 => TE::Reduce(101), 93 => TE::Reduce(101), 94 => TE::Reduce(101), 95 => TE::Reduce(101), 96 => TE::Reduce(101), 98 => TE::Reduce(101) },
    hashmap! { 80 => TE::Shift(183), 82 => TE::Shift(182) },
    hashmap! { 82 => TE::Shift(190) },
    hashmap! { 82 => TE::Shift(194) },
    hashmap! { 28 => TE::Transit(202), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 85 => TE::Reduce(51), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 82 => TE::Shift(203) },
    hashmap! { 85 => TE::Reduce(35) },
    hashmap! { 82 => TE::Shift(206) },
    hashmap! { 82 => TE::Shift(211) },
    hashmap! { 40 => TE::Reduce(22), 44 => TE::Reduce(22), 45 => TE::Reduce(22), 46 => TE::Reduce(22), 47 => TE::Reduce(22), 48 => TE::Reduce(22), 49 => TE::Reduce(22), 50 => TE::Reduce(22), 52 => TE::Reduce(22), 53 => TE::Reduce(22), 54 => TE::Reduce(22), 55 => TE::Reduce(22), 65 => TE::Reduce(22), 66 => TE::Reduce(22), 67 => TE::Reduce(22), 68 => TE::Reduce(22), 69 => TE::Reduce(22), 70 => TE::Reduce(22), 71 => TE::Reduce(22), 72 => TE::Reduce(22), 73 => TE::Reduce(22), 74 => TE::Reduce(22), 75 => TE::Reduce(22), 76 => TE::Reduce(22), 77 => TE::Reduce(22), 78 => TE::Reduce(22), 79 => TE::Reduce(22), 80 => TE::Reduce(22), 81 => TE::Reduce(22), 82 => TE::Reduce(22), 85 => TE::Reduce(22), 89 => TE::Reduce(22), 95 => TE::Reduce(22), 97 => TE::Reduce(22) },
    hashmap! { 40 => TE::Reduce(26), 44 => TE::Reduce(26), 45 => TE::Reduce(26), 46 => TE::Reduce(26), 47 => TE::Reduce(26), 48 => TE::Reduce(26), 49 => TE::Reduce(26), 50 => TE::Reduce(26), 52 => TE::Reduce(26), 53 => TE::Reduce(26), 54 => TE::Reduce(26), 55 => TE::Reduce(26), 65 => TE::Reduce(26), 66 => TE::Reduce(26), 67 => TE::Reduce(26), 68 => TE::Reduce(26), 69 => TE::Reduce(26), 70 => TE::Reduce(26), 71 => TE::Reduce(26), 72 => TE::Reduce(26), 73 => TE::Reduce(26), 74 => TE::Reduce(26), 75 => TE::Reduce(26), 76 => TE::Reduce(26), 77 => TE::Reduce(26), 78 => TE::Reduce(26), 79 => TE::Reduce(26), 80 => TE::Reduce(26), 81 => TE::Reduce(26), 82 => TE::Reduce(26), 85 => TE::Reduce(26), 89 => TE::Reduce(26), 95 => TE::Reduce(26), 97 => TE::Reduce(26) },
    hashmap! { 40 => TE::Reduce(27), 44 => TE::Reduce(27), 45 => TE::Reduce(27), 46 => TE::Reduce(27), 47 => TE::Reduce(27), 48 => TE::Reduce(27), 49 => TE::Reduce(27), 50 => TE::Reduce(27), 52 => TE::Reduce(27), 53 => TE::Reduce(27), 54 => TE::Reduce(27), 55 => TE::Reduce(27), 65 => TE::Reduce(27), 66 => TE::Reduce(27), 67 => TE::Reduce(27), 68 => TE::Reduce(27), 69 => TE::Reduce(27), 70 => TE::Reduce(27), 71 => TE::Reduce(27), 72 => TE::Reduce(27), 73 => TE::Reduce(27), 74 => TE::Reduce(27), 75 => TE::Reduce(27), 76 => TE::Reduce(27), 77 => TE::Reduce(27), 78 => TE::Reduce(27), 79 => TE::Reduce(27), 80 => TE::Reduce(27), 81 => TE::Reduce(27), 82 => TE::Reduce(27), 85 => TE::Reduce(27), 89 => TE::Reduce(27), 95 => TE::Reduce(27), 97 => TE::Reduce(27) },
    hashmap! { 40 => TE::Reduce(28), 44 => TE::Reduce(28), 45 => TE::Reduce(28), 46 => TE::Reduce(28), 47 => TE::Reduce(28), 48 => TE::Reduce(28), 49 => TE::Reduce(28), 50 => TE::Reduce(28), 52 => TE::Reduce(28), 53 => TE::Reduce(28), 54 => TE::Reduce(28), 55 => TE::Reduce(28), 65 => TE::Reduce(28), 66 => TE::Reduce(28), 67 => TE::Reduce(28), 68 => TE::Reduce(28), 69 => TE::Reduce(28), 70 => TE::Reduce(28), 71 => TE::Reduce(28), 72 => TE::Reduce(28), 73 => TE::Reduce(28), 74 => TE::Reduce(28), 75 => TE::Reduce(28), 76 => TE::Reduce(28), 77 => TE::Reduce(28), 78 => TE::Reduce(28), 79 => TE::Reduce(28), 80 => TE::Reduce(28), 81 => TE::Reduce(28), 82 => TE::Reduce(28), 85 => TE::Reduce(28), 89 => TE::Reduce(28), 95 => TE::Reduce(28), 97 => TE::Reduce(28) },
    hashmap! { 40 => TE::Reduce(29), 44 => TE::Reduce(29), 45 => TE::Reduce(29), 46 => TE::Reduce(29), 47 => TE::Reduce(29), 48 => TE::Reduce(29), 49 => TE::Reduce(29), 50 => TE::Reduce(29), 52 => TE::Reduce(29), 53 => TE::Reduce(29), 54 => TE::Reduce(29), 55 => TE::Reduce(29), 65 => TE::Reduce(29), 66 => TE::Reduce(29), 67 => TE::Reduce(29), 68 => TE::Reduce(29), 69 => TE::Reduce(29), 70 => TE::Reduce(29), 71 => TE::Reduce(29), 72 => TE::Reduce(29), 73 => TE::Reduce(29), 74 => TE::Reduce(29), 75 => TE::Reduce(29), 76 => TE::Reduce(29), 77 => TE::Reduce(29), 78 => TE::Reduce(29), 79 => TE::Reduce(29), 80 => TE::Reduce(29), 81 => TE::Reduce(29), 82 => TE::Reduce(29), 85 => TE::Reduce(29), 89 => TE::Reduce(29), 95 => TE::Reduce(29), 97 => TE::Reduce(29) },
    hashmap! { 83 => TE::Reduce(109), 84 => TE::Reduce(109), 85 => TE::Reduce(109) },
    hashmap! { 79 => TE::Reduce(115), 95 => TE::Reduce(115) },
    hashmap! { 79 => TE::Reduce(114), 95 => TE::Reduce(114) },
    hashmap! { 28 => TE::Transit(89), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Reduce(55), 85 => TE::Reduce(55), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(59), 45 => TE::Reduce(59), 47 => TE::Reduce(59), 56 => TE::Reduce(59), 57 => TE::Reduce(59), 58 => TE::Reduce(59), 59 => TE::Reduce(59), 60 => TE::Reduce(59), 61 => TE::Reduce(59), 62 => TE::Reduce(59), 63 => TE::Reduce(59), 83 => TE::Reduce(59), 84 => TE::Reduce(59), 85 => TE::Reduce(59), 86 => TE::Reduce(59), 88 => TE::Reduce(59), 89 => TE::Reduce(59), 90 => TE::Reduce(59), 91 => TE::Reduce(59), 92 => TE::Reduce(59), 93 => TE::Reduce(59), 94 => TE::Reduce(59), 95 => TE::Reduce(59), 96 => TE::Reduce(59), 98 => TE::Reduce(59) },
    hashmap! { 28 => TE::Transit(108), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(109), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(110), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(111), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(112), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(113), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(114), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(115), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(116), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(117), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(118), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(119), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(120), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(121), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(122), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 28 => TE::Transit(123), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 79 => TE::Reduce(93) },
    hashmap! { 44 => TE::Reduce(62), 45 => TE::Reduce(62), 47 => TE::Reduce(62), 56 => TE::Reduce(62), 57 => TE::Reduce(62), 58 => TE::Reduce(62), 59 => TE::Reduce(62), 60 => TE::Reduce(62), 61 => TE::Reduce(62), 62 => TE::Reduce(62), 63 => TE::Reduce(62), 83 => TE::Reduce(62), 84 => TE::Reduce(62), 85 => TE::Reduce(62), 86 => TE::Reduce(62), 88 => TE::Reduce(62), 89 => TE::Reduce(62), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Reduce(62), 94 => TE::Reduce(62), 95 => TE::Shift(106), 96 => TE::Reduce(62), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(63), 45 => TE::Reduce(63), 47 => TE::Reduce(63), 56 => TE::Reduce(63), 57 => TE::Reduce(63), 58 => TE::Reduce(63), 59 => TE::Reduce(63), 60 => TE::Reduce(63), 61 => TE::Reduce(63), 62 => TE::Reduce(63), 63 => TE::Reduce(63), 83 => TE::Reduce(63), 84 => TE::Reduce(63), 85 => TE::Reduce(63), 86 => TE::Reduce(63), 88 => TE::Reduce(63), 89 => TE::Reduce(63), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Reduce(63), 94 => TE::Reduce(63), 95 => TE::Shift(106), 96 => TE::Reduce(63), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(64), 45 => TE::Reduce(64), 47 => TE::Reduce(64), 56 => TE::Reduce(64), 57 => TE::Reduce(64), 58 => TE::Reduce(64), 59 => TE::Reduce(64), 60 => TE::Reduce(64), 61 => TE::Reduce(64), 62 => TE::Reduce(64), 63 => TE::Reduce(64), 83 => TE::Reduce(64), 84 => TE::Reduce(64), 85 => TE::Reduce(64), 86 => TE::Reduce(64), 88 => TE::Reduce(64), 89 => TE::Reduce(64), 90 => TE::Reduce(64), 91 => TE::Reduce(64), 92 => TE::Reduce(64), 93 => TE::Reduce(64), 94 => TE::Reduce(64), 95 => TE::Shift(106), 96 => TE::Reduce(64), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(65), 45 => TE::Reduce(65), 47 => TE::Reduce(65), 56 => TE::Reduce(65), 57 => TE::Reduce(65), 58 => TE::Reduce(65), 59 => TE::Reduce(65), 60 => TE::Reduce(65), 61 => TE::Reduce(65), 62 => TE::Reduce(65), 63 => TE::Reduce(65), 83 => TE::Reduce(65), 84 => TE::Reduce(65), 85 => TE::Reduce(65), 86 => TE::Reduce(65), 88 => TE::Reduce(65), 89 => TE::Reduce(65), 90 => TE::Reduce(65), 91 => TE::Reduce(65), 92 => TE::Reduce(65), 93 => TE::Reduce(65), 94 => TE::Reduce(65), 95 => TE::Shift(106), 96 => TE::Reduce(65), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(66), 45 => TE::Reduce(66), 47 => TE::Reduce(66), 56 => TE::Reduce(66), 57 => TE::Reduce(66), 58 => TE::Reduce(66), 59 => TE::Reduce(66), 60 => TE::Reduce(66), 61 => TE::Reduce(66), 62 => TE::Reduce(66), 63 => TE::Reduce(66), 83 => TE::Reduce(66), 84 => TE::Reduce(66), 85 => TE::Reduce(66), 86 => TE::Reduce(66), 88 => TE::Reduce(66), 89 => TE::Reduce(66), 90 => TE::Reduce(66), 91 => TE::Reduce(66), 92 => TE::Reduce(66), 93 => TE::Reduce(66), 94 => TE::Reduce(66), 95 => TE::Shift(106), 96 => TE::Reduce(66), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(67), 45 => TE::Reduce(67), 47 => TE::Reduce(67), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Reduce(67), 61 => TE::Reduce(67), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Reduce(67), 84 => TE::Reduce(67), 85 => TE::Reduce(67), 86 => TE::Reduce(67), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 96 => TE::Reduce(67), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(68), 45 => TE::Reduce(68), 47 => TE::Reduce(68), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Reduce(68), 61 => TE::Reduce(68), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Reduce(68), 84 => TE::Reduce(68), 85 => TE::Reduce(68), 86 => TE::Reduce(68), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 96 => TE::Reduce(68), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(69), 45 => TE::Reduce(69), 47 => TE::Reduce(69), 56 => TE::Reduce(69), 57 => TE::Reduce(69), 60 => TE::Reduce(69), 61 => TE::Reduce(69), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Reduce(69), 84 => TE::Reduce(69), 85 => TE::Reduce(69), 86 => TE::Reduce(69), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 95 => TE::Shift(106), 96 => TE::Reduce(69), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(70), 45 => TE::Reduce(70), 47 => TE::Reduce(70), 56 => TE::Reduce(70), 57 => TE::Reduce(70), 60 => TE::Reduce(70), 61 => TE::Reduce(70), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Reduce(70), 84 => TE::Reduce(70), 85 => TE::Reduce(70), 86 => TE::Reduce(70), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 95 => TE::Shift(106), 96 => TE::Reduce(70), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(71), 45 => TE::Reduce(71), 47 => TE::Reduce(71), 56 => TE::Reduce(71), 57 => TE::Reduce(71), 60 => TE::Reduce(71), 61 => TE::Reduce(71), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Reduce(71), 84 => TE::Reduce(71), 85 => TE::Reduce(71), 86 => TE::Reduce(71), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 95 => TE::Shift(106), 96 => TE::Reduce(71), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(72), 45 => TE::Reduce(72), 47 => TE::Reduce(72), 56 => TE::Reduce(72), 57 => TE::Reduce(72), 60 => TE::Reduce(72), 61 => TE::Reduce(72), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Reduce(72), 84 => TE::Reduce(72), 85 => TE::Reduce(72), 86 => TE::Reduce(72), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 95 => TE::Shift(106), 96 => TE::Reduce(72), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(73), 45 => TE::Reduce(73), 47 => TE::Reduce(73), 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Reduce(73), 61 => TE::Reduce(73), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Reduce(73), 84 => TE::Reduce(73), 85 => TE::Reduce(73), 86 => TE::Reduce(73), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 96 => TE::Reduce(73), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(74), 45 => TE::Reduce(74), 47 => TE::Reduce(74), 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Reduce(74), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Reduce(74), 84 => TE::Reduce(74), 85 => TE::Reduce(74), 86 => TE::Reduce(74), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 96 => TE::Reduce(74), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(75), 45 => TE::Reduce(75), 47 => TE::Reduce(75), 56 => TE::Reduce(75), 57 => TE::Reduce(75), 58 => TE::Reduce(75), 59 => TE::Reduce(75), 60 => TE::Reduce(75), 61 => TE::Reduce(75), 62 => TE::Reduce(75), 63 => TE::Reduce(75), 83 => TE::Reduce(75), 84 => TE::Reduce(75), 85 => TE::Reduce(75), 86 => TE::Reduce(75), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Reduce(75), 94 => TE::Reduce(75), 95 => TE::Shift(106), 96 => TE::Reduce(75), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(76), 45 => TE::Reduce(76), 47 => TE::Reduce(76), 56 => TE::Reduce(76), 57 => TE::Reduce(76), 58 => TE::Reduce(76), 59 => TE::Reduce(76), 60 => TE::Reduce(76), 61 => TE::Reduce(76), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Reduce(76), 84 => TE::Reduce(76), 85 => TE::Reduce(76), 86 => TE::Reduce(76), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Reduce(76), 94 => TE::Reduce(76), 95 => TE::Shift(106), 96 => TE::Reduce(76), 98 => TE::Shift(107) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 86 => TE::Shift(124), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 96 => TE::Shift(125), 98 => TE::Shift(107) },
    hashmap! { 28 => TE::Transit(126), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 44 => TE::Reduce(92), 45 => TE::Reduce(92), 47 => TE::Reduce(92), 56 => TE::Reduce(92), 57 => TE::Reduce(92), 58 => TE::Reduce(92), 59 => TE::Reduce(92), 60 => TE::Reduce(92), 61 => TE::Reduce(92), 62 => TE::Reduce(92), 63 => TE::Reduce(92), 64 => TE::Shift(177), 83 => TE::Reduce(92), 84 => TE::Reduce(92), 85 => TE::Reduce(92), 86 => TE::Reduce(92), 87 => TE::Reduce(92), 88 => TE::Reduce(92), 89 => TE::Reduce(92), 90 => TE::Reduce(92), 91 => TE::Reduce(92), 92 => TE::Reduce(92), 93 => TE::Reduce(92), 94 => TE::Reduce(92), 95 => TE::Reduce(92), 96 => TE::Reduce(92), 98 => TE::Reduce(92) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 96 => TE::Shift(127), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(77), 45 => TE::Reduce(77), 47 => TE::Reduce(77), 56 => TE::Reduce(77), 57 => TE::Reduce(77), 58 => TE::Reduce(77), 59 => TE::Reduce(77), 60 => TE::Reduce(77), 61 => TE::Reduce(77), 62 => TE::Reduce(77), 63 => TE::Reduce(77), 83 => TE::Reduce(77), 84 => TE::Reduce(77), 85 => TE::Reduce(77), 86 => TE::Reduce(77), 88 => TE::Reduce(77), 89 => TE::Reduce(77), 90 => TE::Reduce(77), 91 => TE::Reduce(77), 92 => TE::Reduce(77), 93 => TE::Reduce(77), 94 => TE::Reduce(77), 95 => TE::Reduce(77), 96 => TE::Reduce(77), 98 => TE::Reduce(77) },
    hashmap! { 45 => TE::Shift(132), 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 84 => TE::Shift(173), 96 => TE::Shift(172) },
    hashmap! { 44 => TE::Reduce(103), 45 => TE::Reduce(103), 47 => TE::Reduce(103), 56 => TE::Reduce(103), 57 => TE::Reduce(103), 58 => TE::Reduce(103), 59 => TE::Reduce(103), 60 => TE::Reduce(103), 61 => TE::Reduce(103), 62 => TE::Reduce(103), 63 => TE::Reduce(103), 83 => TE::Reduce(103), 84 => TE::Reduce(103), 85 => TE::Reduce(103), 86 => TE::Reduce(103), 88 => TE::Reduce(103), 89 => TE::Reduce(103), 90 => TE::Reduce(103), 91 => TE::Reduce(103), 92 => TE::Reduce(103), 93 => TE::Reduce(103), 94 => TE::Reduce(103), 95 => TE::Reduce(103), 96 => TE::Reduce(103), 98 => TE::Reduce(103) },
    hashmap! { 45 => TE::Reduce(61), 56 => TE::Reduce(61), 57 => TE::Reduce(61), 58 => TE::Reduce(61), 59 => TE::Reduce(61), 60 => TE::Reduce(61), 61 => TE::Reduce(61), 62 => TE::Reduce(61), 63 => TE::Reduce(61), 84 => TE::Reduce(105), 88 => TE::Reduce(61), 89 => TE::Reduce(61), 90 => TE::Reduce(61), 91 => TE::Reduce(61), 92 => TE::Reduce(61), 93 => TE::Reduce(61), 94 => TE::Reduce(61), 95 => TE::Reduce(61), 96 => TE::Reduce(105), 98 => TE::Reduce(61) },
    hashmap! { 39 => TE::Transit(133), 79 => TE::Shift(9) },
    hashmap! { 51 => TE::Shift(134) },
    hashmap! { 28 => TE::Transit(135), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 47 => TE::Shift(137), 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 96 => TE::Shift(136), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(79), 45 => TE::Reduce(79), 47 => TE::Reduce(79), 56 => TE::Reduce(79), 57 => TE::Reduce(79), 58 => TE::Reduce(79), 59 => TE::Reduce(79), 60 => TE::Reduce(79), 61 => TE::Reduce(79), 62 => TE::Reduce(79), 63 => TE::Reduce(79), 83 => TE::Reduce(79), 84 => TE::Reduce(79), 85 => TE::Reduce(79), 86 => TE::Reduce(79), 88 => TE::Reduce(79), 89 => TE::Reduce(79), 90 => TE::Reduce(79), 91 => TE::Reduce(79), 92 => TE::Reduce(79), 93 => TE::Reduce(79), 94 => TE::Reduce(79), 95 => TE::Reduce(79), 96 => TE::Reduce(79), 98 => TE::Reduce(79) },
    hashmap! { 28 => TE::Transit(138), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 96 => TE::Shift(139), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(80), 45 => TE::Reduce(80), 47 => TE::Reduce(80), 56 => TE::Reduce(80), 57 => TE::Reduce(80), 58 => TE::Reduce(80), 59 => TE::Reduce(80), 60 => TE::Reduce(80), 61 => TE::Reduce(80), 62 => TE::Reduce(80), 63 => TE::Reduce(80), 83 => TE::Reduce(80), 84 => TE::Reduce(80), 85 => TE::Reduce(80), 86 => TE::Reduce(80), 88 => TE::Reduce(80), 89 => TE::Reduce(80), 90 => TE::Reduce(80), 91 => TE::Reduce(80), 92 => TE::Reduce(80), 93 => TE::Reduce(80), 94 => TE::Reduce(80), 95 => TE::Reduce(80), 96 => TE::Reduce(80), 98 => TE::Reduce(80) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Shift(142), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 39 => TE::Transit(143), 79 => TE::Shift(9) },
    hashmap! { 44 => TE::Reduce(81), 45 => TE::Reduce(81), 47 => TE::Reduce(81), 56 => TE::Reduce(81), 57 => TE::Reduce(81), 58 => TE::Reduce(81), 59 => TE::Reduce(81), 60 => TE::Reduce(81), 61 => TE::Reduce(81), 62 => TE::Reduce(81), 63 => TE::Reduce(81), 83 => TE::Reduce(81), 84 => TE::Reduce(81), 85 => TE::Reduce(81), 86 => TE::Reduce(81), 88 => TE::Reduce(81), 89 => TE::Reduce(81), 90 => TE::Reduce(81), 91 => TE::Reduce(81), 92 => TE::Reduce(81), 93 => TE::Reduce(81), 94 => TE::Reduce(81), 95 => TE::Reduce(81), 96 => TE::Reduce(81), 98 => TE::Reduce(81) },
    hashmap! { 83 => TE::Shift(144) },
    hashmap! { 28 => TE::Transit(145), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 44 => TE::Reduce(90), 45 => TE::Reduce(90), 47 => TE::Reduce(90), 56 => TE::Reduce(90), 57 => TE::Reduce(90), 58 => TE::Reduce(90), 59 => TE::Reduce(90), 60 => TE::Reduce(90), 61 => TE::Reduce(90), 62 => TE::Reduce(90), 63 => TE::Reduce(90), 83 => TE::Reduce(90), 84 => TE::Reduce(90), 85 => TE::Reduce(90), 86 => TE::Reduce(90), 88 => TE::Reduce(90), 89 => TE::Reduce(90), 90 => TE::Reduce(90), 91 => TE::Reduce(90), 92 => TE::Reduce(90), 93 => TE::Reduce(90), 94 => TE::Reduce(90), 95 => TE::Reduce(90), 96 => TE::Reduce(90), 98 => TE::Reduce(90) },
    hashmap! { 44 => TE::Reduce(82), 45 => TE::Reduce(82), 47 => TE::Reduce(82), 56 => TE::Reduce(82), 57 => TE::Reduce(82), 58 => TE::Reduce(82), 59 => TE::Reduce(82), 60 => TE::Reduce(82), 61 => TE::Reduce(82), 62 => TE::Reduce(82), 63 => TE::Reduce(82), 83 => TE::Reduce(82), 84 => TE::Reduce(82), 85 => TE::Reduce(82), 86 => TE::Reduce(82), 88 => TE::Reduce(82), 89 => TE::Reduce(82), 90 => TE::Reduce(82), 91 => TE::Reduce(82), 92 => TE::Reduce(82), 93 => TE::Reduce(82), 94 => TE::Reduce(82), 95 => TE::Shift(106), 96 => TE::Reduce(82), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(83), 45 => TE::Reduce(83), 47 => TE::Reduce(83), 56 => TE::Reduce(83), 57 => TE::Reduce(83), 58 => TE::Reduce(83), 59 => TE::Reduce(83), 60 => TE::Reduce(83), 61 => TE::Reduce(83), 62 => TE::Reduce(83), 63 => TE::Reduce(83), 83 => TE::Reduce(83), 84 => TE::Reduce(83), 85 => TE::Reduce(83), 86 => TE::Reduce(83), 88 => TE::Reduce(83), 89 => TE::Reduce(83), 90 => TE::Reduce(83), 91 => TE::Reduce(83), 92 => TE::Reduce(83), 93 => TE::Reduce(83), 94 => TE::Reduce(83), 95 => TE::Shift(106), 96 => TE::Reduce(83), 98 => TE::Shift(107) },
    hashmap! { 83 => TE::Shift(149) },
    hashmap! { 44 => TE::Reduce(84), 45 => TE::Reduce(84), 47 => TE::Reduce(84), 56 => TE::Reduce(84), 57 => TE::Reduce(84), 58 => TE::Reduce(84), 59 => TE::Reduce(84), 60 => TE::Reduce(84), 61 => TE::Reduce(84), 62 => TE::Reduce(84), 63 => TE::Reduce(84), 83 => TE::Reduce(84), 84 => TE::Reduce(84), 85 => TE::Reduce(84), 86 => TE::Reduce(84), 88 => TE::Reduce(84), 89 => TE::Reduce(84), 90 => TE::Reduce(84), 91 => TE::Reduce(84), 92 => TE::Reduce(84), 93 => TE::Reduce(84), 94 => TE::Reduce(84), 95 => TE::Reduce(84), 96 => TE::Reduce(84), 98 => TE::Reduce(84) },
    hashmap! { 83 => TE::Shift(151) },
    hashmap! { 44 => TE::Reduce(85), 45 => TE::Reduce(85), 47 => TE::Reduce(85), 56 => TE::Reduce(85), 57 => TE::Reduce(85), 58 => TE::Reduce(85), 59 => TE::Reduce(85), 60 => TE::Reduce(85), 61 => TE::Reduce(85), 62 => TE::Reduce(85), 63 => TE::Reduce(85), 83 => TE::Reduce(85), 84 => TE::Reduce(85), 85 => TE::Reduce(85), 86 => TE::Reduce(85), 88 => TE::Reduce(85), 89 => TE::Reduce(85), 90 => TE::Reduce(85), 91 => TE::Reduce(85), 92 => TE::Reduce(85), 93 => TE::Reduce(85), 94 => TE::Reduce(85), 95 => TE::Reduce(85), 96 => TE::Reduce(85), 98 => TE::Reduce(85) },
    hashmap! { 82 => TE::Shift(154) },
    hashmap! { 95 => TE::Shift(156) },
    hashmap! { 83 => TE::Shift(155) },
    hashmap! { 44 => TE::Reduce(87), 45 => TE::Reduce(87), 47 => TE::Reduce(87), 56 => TE::Reduce(87), 57 => TE::Reduce(87), 58 => TE::Reduce(87), 59 => TE::Reduce(87), 60 => TE::Reduce(87), 61 => TE::Reduce(87), 62 => TE::Reduce(87), 63 => TE::Reduce(87), 83 => TE::Reduce(87), 84 => TE::Reduce(87), 85 => TE::Reduce(87), 86 => TE::Reduce(87), 88 => TE::Reduce(87), 89 => TE::Reduce(87), 90 => TE::Reduce(87), 91 => TE::Reduce(87), 92 => TE::Reduce(87), 93 => TE::Reduce(87), 94 => TE::Reduce(87), 95 => TE::Reduce(87), 96 => TE::Reduce(87), 98 => TE::Reduce(87) },
    hashmap! { 28 => TE::Transit(157), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 96 => TE::Shift(86), 97 => TE::Shift(60) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 96 => TE::Shift(158), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(88), 45 => TE::Reduce(88), 47 => TE::Reduce(88), 56 => TE::Reduce(88), 57 => TE::Reduce(88), 58 => TE::Reduce(88), 59 => TE::Reduce(88), 60 => TE::Reduce(88), 61 => TE::Reduce(88), 62 => TE::Reduce(88), 63 => TE::Reduce(88), 83 => TE::Reduce(88), 84 => TE::Reduce(88), 85 => TE::Reduce(88), 86 => TE::Reduce(88), 88 => TE::Reduce(88), 89 => TE::Reduce(88), 90 => TE::Reduce(88), 91 => TE::Reduce(88), 92 => TE::Reduce(88), 93 => TE::Reduce(88), 94 => TE::Reduce(88), 95 => TE::Reduce(88), 96 => TE::Reduce(88), 98 => TE::Reduce(88) },
    hashmap! { 28 => TE::Transit(160), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 84 => TE::Shift(161), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 39 => TE::Transit(162), 79 => TE::Shift(9) },
    hashmap! { 83 => TE::Shift(163) },
    hashmap! { 44 => TE::Reduce(89), 45 => TE::Reduce(89), 47 => TE::Reduce(89), 56 => TE::Reduce(89), 57 => TE::Reduce(89), 58 => TE::Reduce(89), 59 => TE::Reduce(89), 60 => TE::Reduce(89), 61 => TE::Reduce(89), 62 => TE::Reduce(89), 63 => TE::Reduce(89), 83 => TE::Reduce(89), 84 => TE::Reduce(89), 85 => TE::Reduce(89), 86 => TE::Reduce(89), 88 => TE::Reduce(89), 89 => TE::Reduce(89), 90 => TE::Reduce(89), 91 => TE::Reduce(89), 92 => TE::Reduce(89), 93 => TE::Reduce(89), 94 => TE::Reduce(89), 95 => TE::Reduce(89), 96 => TE::Reduce(89), 98 => TE::Reduce(89) },
    hashmap! { 44 => TE::Reduce(91), 45 => TE::Reduce(91), 47 => TE::Reduce(91), 56 => TE::Reduce(91), 57 => TE::Reduce(91), 58 => TE::Reduce(91), 59 => TE::Reduce(91), 60 => TE::Reduce(91), 61 => TE::Reduce(91), 62 => TE::Reduce(91), 63 => TE::Reduce(91), 82 => TE::Shift(165), 83 => TE::Reduce(91), 84 => TE::Reduce(91), 85 => TE::Reduce(91), 86 => TE::Reduce(91), 87 => TE::Reduce(91), 88 => TE::Reduce(91), 89 => TE::Reduce(91), 90 => TE::Reduce(91), 91 => TE::Reduce(91), 92 => TE::Reduce(91), 93 => TE::Reduce(91), 94 => TE::Reduce(91), 95 => TE::Reduce(91), 96 => TE::Reduce(91), 98 => TE::Reduce(91) },
    hashmap! { 26 => TE::Transit(167), 28 => TE::Transit(168), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 35 => TE::Transit(166), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 83 => TE::Reduce(107), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 83 => TE::Shift(169) },
    hashmap! { 83 => TE::Reduce(106), 84 => TE::Shift(170) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Reduce(54), 84 => TE::Reduce(54), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(95), 45 => TE::Reduce(95), 47 => TE::Reduce(95), 56 => TE::Reduce(95), 57 => TE::Reduce(95), 58 => TE::Reduce(95), 59 => TE::Reduce(95), 60 => TE::Reduce(95), 61 => TE::Reduce(95), 62 => TE::Reduce(95), 63 => TE::Reduce(95), 83 => TE::Reduce(95), 84 => TE::Reduce(95), 85 => TE::Reduce(95), 86 => TE::Reduce(95), 88 => TE::Reduce(95), 89 => TE::Reduce(95), 90 => TE::Reduce(95), 91 => TE::Reduce(95), 92 => TE::Reduce(95), 93 => TE::Reduce(95), 94 => TE::Reduce(95), 95 => TE::Reduce(95), 96 => TE::Reduce(95), 98 => TE::Reduce(95) },
    hashmap! { 28 => TE::Transit(171), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Reduce(53), 84 => TE::Reduce(53), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 44 => TE::Reduce(102), 45 => TE::Reduce(102), 47 => TE::Reduce(102), 56 => TE::Reduce(102), 57 => TE::Reduce(102), 58 => TE::Reduce(102), 59 => TE::Reduce(102), 60 => TE::Reduce(102), 61 => TE::Reduce(102), 62 => TE::Reduce(102), 63 => TE::Reduce(102), 83 => TE::Reduce(102), 84 => TE::Reduce(102), 85 => TE::Reduce(102), 86 => TE::Reduce(102), 88 => TE::Reduce(102), 89 => TE::Reduce(102), 90 => TE::Reduce(102), 91 => TE::Reduce(102), 92 => TE::Reduce(102), 93 => TE::Reduce(102), 94 => TE::Reduce(102), 95 => TE::Reduce(102), 96 => TE::Reduce(102), 98 => TE::Reduce(102) },
    hashmap! { 32 => TE::Transit(174), 33 => TE::Transit(70), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 95 => TE::Shift(175) },
    hashmap! { 84 => TE::Reduce(104), 96 => TE::Reduce(104) },
    hashmap! { 32 => TE::Transit(176), 33 => TE::Transit(70), 34 => TE::Transit(129), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 95 => TE::Shift(175), 96 => TE::Shift(130) },
    hashmap! { 84 => TE::Reduce(105), 96 => TE::Reduce(105) },
    hashmap! { 28 => TE::Transit(178), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 44 => TE::Reduce(78), 45 => TE::Reduce(78), 47 => TE::Reduce(78), 56 => TE::Reduce(78), 57 => TE::Reduce(78), 58 => TE::Reduce(78), 59 => TE::Reduce(78), 60 => TE::Reduce(78), 61 => TE::Reduce(78), 62 => TE::Reduce(78), 63 => TE::Reduce(78), 83 => TE::Reduce(78), 84 => TE::Reduce(78), 85 => TE::Reduce(78), 86 => TE::Reduce(78), 88 => TE::Reduce(78), 89 => TE::Reduce(78), 90 => TE::Reduce(78), 91 => TE::Reduce(78), 92 => TE::Reduce(78), 93 => TE::Reduce(78), 94 => TE::Reduce(78), 96 => TE::Reduce(78) },
    hashmap! { 87 => TE::Shift(180) },
    hashmap! { 28 => TE::Transit(181), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Reduce(56), 85 => TE::Reduce(56), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 28 => TE::Transit(184), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 22 => TE::Transit(223), 23 => TE::Transit(224), 28 => TE::Transit(225), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 81 => TE::Reduce(47), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Shift(185), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 9 => TE::Transit(50), 11 => TE::Transit(186), 12 => TE::Transit(42), 13 => TE::Transit(43), 14 => TE::Transit(46), 15 => TE::Transit(41), 17 => TE::Transit(47), 18 => TE::Transit(48), 21 => TE::Transit(49), 24 => TE::Transit(44), 25 => TE::Transit(45), 27 => TE::Transit(40), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 36 => TE::Transit(39), 37 => TE::Transit(17), 38 => TE::Transit(32), 40 => TE::Shift(23), 44 => TE::Shift(73), 45 => TE::Shift(74), 46 => TE::Shift(77), 47 => TE::Shift(72), 49 => TE::Shift(78), 50 => TE::Shift(79), 52 => TE::Shift(52), 54 => TE::Shift(75), 55 => TE::Shift(76), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 75 => TE::Shift(19), 76 => TE::Shift(20), 77 => TE::Shift(21), 78 => TE::Shift(22), 79 => TE::Reduce(94), 80 => TE::Shift(35), 82 => TE::Shift(58), 85 => TE::Reduce(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 16 => TE::Transit(187), 40 => TE::Reduce(38), 44 => TE::Reduce(38), 45 => TE::Reduce(38), 46 => TE::Reduce(38), 47 => TE::Reduce(38), 48 => TE::Shift(188), 49 => TE::Reduce(38), 50 => TE::Reduce(38), 52 => TE::Reduce(38), 53 => TE::Reduce(38), 54 => TE::Reduce(38), 55 => TE::Reduce(38), 65 => TE::Reduce(38), 66 => TE::Reduce(38), 67 => TE::Reduce(38), 68 => TE::Reduce(38), 69 => TE::Reduce(38), 70 => TE::Reduce(38), 71 => TE::Reduce(38), 72 => TE::Reduce(38), 73 => TE::Reduce(38), 74 => TE::Reduce(38), 75 => TE::Reduce(38), 76 => TE::Reduce(38), 77 => TE::Reduce(38), 78 => TE::Reduce(38), 79 => TE::Reduce(38), 80 => TE::Reduce(38), 81 => TE::Reduce(38), 82 => TE::Reduce(38), 85 => TE::Reduce(38), 89 => TE::Reduce(38), 95 => TE::Reduce(38), 97 => TE::Reduce(38) },
    hashmap! { 40 => TE::Reduce(36), 44 => TE::Reduce(36), 45 => TE::Reduce(36), 46 => TE::Reduce(36), 47 => TE::Reduce(36), 48 => TE::Reduce(36), 49 => TE::Reduce(36), 50 => TE::Reduce(36), 52 => TE::Reduce(36), 53 => TE::Reduce(36), 54 => TE::Reduce(36), 55 => TE::Reduce(36), 65 => TE::Reduce(36), 66 => TE::Reduce(36), 67 => TE::Reduce(36), 68 => TE::Reduce(36), 69 => TE::Reduce(36), 70 => TE::Reduce(36), 71 => TE::Reduce(36), 72 => TE::Reduce(36), 73 => TE::Reduce(36), 74 => TE::Reduce(36), 75 => TE::Reduce(36), 76 => TE::Reduce(36), 77 => TE::Reduce(36), 78 => TE::Reduce(36), 79 => TE::Reduce(36), 80 => TE::Reduce(36), 81 => TE::Reduce(36), 82 => TE::Reduce(36), 85 => TE::Reduce(36), 89 => TE::Reduce(36), 95 => TE::Reduce(36), 97 => TE::Reduce(36) },
    hashmap! { 9 => TE::Transit(50), 11 => TE::Transit(189), 12 => TE::Transit(42), 13 => TE::Transit(43), 14 => TE::Transit(46), 15 => TE::Transit(41), 17 => TE::Transit(47), 18 => TE::Transit(48), 21 => TE::Transit(49), 24 => TE::Transit(44), 25 => TE::Transit(45), 27 => TE::Transit(40), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 36 => TE::Transit(39), 37 => TE::Transit(17), 38 => TE::Transit(32), 40 => TE::Shift(23), 44 => TE::Shift(73), 45 => TE::Shift(74), 46 => TE::Shift(77), 47 => TE::Shift(72), 49 => TE::Shift(78), 50 => TE::Shift(79), 52 => TE::Shift(52), 54 => TE::Shift(75), 55 => TE::Shift(76), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 75 => TE::Shift(19), 76 => TE::Shift(20), 77 => TE::Shift(21), 78 => TE::Shift(22), 79 => TE::Reduce(94), 80 => TE::Shift(35), 82 => TE::Shift(58), 85 => TE::Reduce(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 40 => TE::Reduce(37), 44 => TE::Reduce(37), 45 => TE::Reduce(37), 46 => TE::Reduce(37), 47 => TE::Reduce(37), 48 => TE::Reduce(37), 49 => TE::Reduce(37), 50 => TE::Reduce(37), 52 => TE::Reduce(37), 53 => TE::Reduce(37), 54 => TE::Reduce(37), 55 => TE::Reduce(37), 65 => TE::Reduce(37), 66 => TE::Reduce(37), 67 => TE::Reduce(37), 68 => TE::Reduce(37), 69 => TE::Reduce(37), 70 => TE::Reduce(37), 71 => TE::Reduce(37), 72 => TE::Reduce(37), 73 => TE::Reduce(37), 74 => TE::Reduce(37), 75 => TE::Reduce(37), 76 => TE::Reduce(37), 77 => TE::Reduce(37), 78 => TE::Reduce(37), 79 => TE::Reduce(37), 80 => TE::Reduce(37), 81 => TE::Reduce(37), 82 => TE::Reduce(37), 85 => TE::Reduce(37), 89 => TE::Reduce(37), 95 => TE::Reduce(37), 97 => TE::Reduce(37) },
    hashmap! { 28 => TE::Transit(191), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Shift(192), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 9 => TE::Transit(50), 11 => TE::Transit(193), 12 => TE::Transit(42), 13 => TE::Transit(43), 14 => TE::Transit(46), 15 => TE::Transit(41), 17 => TE::Transit(47), 18 => TE::Transit(48), 21 => TE::Transit(49), 24 => TE::Transit(44), 25 => TE::Transit(45), 27 => TE::Transit(40), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 36 => TE::Transit(39), 37 => TE::Transit(17), 38 => TE::Transit(32), 40 => TE::Shift(23), 44 => TE::Shift(73), 45 => TE::Shift(74), 46 => TE::Shift(77), 47 => TE::Shift(72), 49 => TE::Shift(78), 50 => TE::Shift(79), 52 => TE::Shift(52), 54 => TE::Shift(75), 55 => TE::Shift(76), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 75 => TE::Shift(19), 76 => TE::Shift(20), 77 => TE::Shift(21), 78 => TE::Shift(22), 79 => TE::Reduce(94), 80 => TE::Shift(35), 82 => TE::Shift(58), 85 => TE::Reduce(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 40 => TE::Reduce(33), 44 => TE::Reduce(33), 45 => TE::Reduce(33), 46 => TE::Reduce(33), 47 => TE::Reduce(33), 48 => TE::Reduce(33), 49 => TE::Reduce(33), 50 => TE::Reduce(33), 52 => TE::Reduce(33), 53 => TE::Reduce(33), 54 => TE::Reduce(33), 55 => TE::Reduce(33), 65 => TE::Reduce(33), 66 => TE::Reduce(33), 67 => TE::Reduce(33), 68 => TE::Reduce(33), 69 => TE::Reduce(33), 70 => TE::Reduce(33), 71 => TE::Reduce(33), 72 => TE::Reduce(33), 73 => TE::Reduce(33), 74 => TE::Reduce(33), 75 => TE::Reduce(33), 76 => TE::Reduce(33), 77 => TE::Reduce(33), 78 => TE::Reduce(33), 79 => TE::Reduce(33), 80 => TE::Reduce(33), 81 => TE::Reduce(33), 82 => TE::Reduce(33), 85 => TE::Reduce(33), 89 => TE::Reduce(33), 95 => TE::Reduce(33), 97 => TE::Reduce(33) },
    hashmap! { 27 => TE::Transit(195), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 52 => TE::Shift(52), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 85 => TE::Reduce(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 85 => TE::Shift(196) },
    hashmap! { 28 => TE::Transit(197), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 85 => TE::Shift(198), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 27 => TE::Transit(199), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 52 => TE::Shift(52), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 83 => TE::Reduce(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 83 => TE::Shift(200) },
    hashmap! { 9 => TE::Transit(50), 11 => TE::Transit(201), 12 => TE::Transit(42), 13 => TE::Transit(43), 14 => TE::Transit(46), 15 => TE::Transit(41), 17 => TE::Transit(47), 18 => TE::Transit(48), 21 => TE::Transit(49), 24 => TE::Transit(44), 25 => TE::Transit(45), 27 => TE::Transit(40), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 36 => TE::Transit(39), 37 => TE::Transit(17), 38 => TE::Transit(32), 40 => TE::Shift(23), 44 => TE::Shift(73), 45 => TE::Shift(74), 46 => TE::Shift(77), 47 => TE::Shift(72), 49 => TE::Shift(78), 50 => TE::Shift(79), 52 => TE::Shift(52), 54 => TE::Shift(75), 55 => TE::Shift(76), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 75 => TE::Shift(19), 76 => TE::Shift(20), 77 => TE::Shift(21), 78 => TE::Shift(22), 79 => TE::Reduce(94), 80 => TE::Shift(35), 82 => TE::Shift(58), 85 => TE::Reduce(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 40 => TE::Reduce(34), 44 => TE::Reduce(34), 45 => TE::Reduce(34), 46 => TE::Reduce(34), 47 => TE::Reduce(34), 48 => TE::Reduce(34), 49 => TE::Reduce(34), 50 => TE::Reduce(34), 52 => TE::Reduce(34), 53 => TE::Reduce(34), 54 => TE::Reduce(34), 55 => TE::Reduce(34), 65 => TE::Reduce(34), 66 => TE::Reduce(34), 67 => TE::Reduce(34), 68 => TE::Reduce(34), 69 => TE::Reduce(34), 70 => TE::Reduce(34), 71 => TE::Reduce(34), 72 => TE::Reduce(34), 73 => TE::Reduce(34), 74 => TE::Reduce(34), 75 => TE::Reduce(34), 76 => TE::Reduce(34), 77 => TE::Reduce(34), 78 => TE::Reduce(34), 79 => TE::Reduce(34), 80 => TE::Reduce(34), 81 => TE::Reduce(34), 82 => TE::Reduce(34), 85 => TE::Reduce(34), 89 => TE::Reduce(34), 95 => TE::Reduce(34), 97 => TE::Reduce(34) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 85 => TE::Reduce(50), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 26 => TE::Transit(204), 28 => TE::Transit(168), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 83 => TE::Shift(205), 84 => TE::Shift(170) },
    hashmap! { 85 => TE::Reduce(52) },
    hashmap! { 39 => TE::Transit(207), 79 => TE::Shift(9) },
    hashmap! { 84 => TE::Shift(208) },
    hashmap! { 28 => TE::Transit(209), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Shift(210), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 85 => TE::Reduce(39) },
    hashmap! { 19 => TE::Transit(212), 38 => TE::Transit(214), 40 => TE::Shift(23), 52 => TE::Shift(213), 75 => TE::Shift(19), 76 => TE::Shift(20), 77 => TE::Shift(21), 78 => TE::Shift(22) },
    hashmap! { 39 => TE::Transit(215), 79 => TE::Shift(9) },
    hashmap! { 79 => TE::Reduce(41) },
    hashmap! { 79 => TE::Reduce(42), 95 => TE::Shift(27) },
    hashmap! { 51 => TE::Shift(216) },
    hashmap! { 28 => TE::Transit(217), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 20 => TE::Transit(218), 44 => TE::Shift(219), 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Reduce(44), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 83 => TE::Shift(220) },
    hashmap! { 28 => TE::Transit(222), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 9 => TE::Transit(50), 11 => TE::Transit(221), 12 => TE::Transit(42), 13 => TE::Transit(43), 14 => TE::Transit(46), 15 => TE::Transit(41), 17 => TE::Transit(47), 18 => TE::Transit(48), 21 => TE::Transit(49), 24 => TE::Transit(44), 25 => TE::Transit(45), 27 => TE::Transit(40), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 36 => TE::Transit(39), 37 => TE::Transit(17), 38 => TE::Transit(32), 40 => TE::Shift(23), 44 => TE::Shift(73), 45 => TE::Shift(74), 46 => TE::Shift(77), 47 => TE::Shift(72), 49 => TE::Shift(78), 50 => TE::Shift(79), 52 => TE::Shift(52), 54 => TE::Shift(75), 55 => TE::Shift(76), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 75 => TE::Shift(19), 76 => TE::Shift(20), 77 => TE::Shift(21), 78 => TE::Shift(22), 79 => TE::Reduce(94), 80 => TE::Shift(35), 82 => TE::Shift(58), 85 => TE::Reduce(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 40 => TE::Reduce(40), 44 => TE::Reduce(40), 45 => TE::Reduce(40), 46 => TE::Reduce(40), 47 => TE::Reduce(40), 48 => TE::Reduce(40), 49 => TE::Reduce(40), 50 => TE::Reduce(40), 52 => TE::Reduce(40), 53 => TE::Reduce(40), 54 => TE::Reduce(40), 55 => TE::Reduce(40), 65 => TE::Reduce(40), 66 => TE::Reduce(40), 67 => TE::Reduce(40), 68 => TE::Reduce(40), 69 => TE::Reduce(40), 70 => TE::Reduce(40), 71 => TE::Reduce(40), 72 => TE::Reduce(40), 73 => TE::Reduce(40), 74 => TE::Reduce(40), 75 => TE::Reduce(40), 76 => TE::Reduce(40), 77 => TE::Reduce(40), 78 => TE::Reduce(40), 79 => TE::Reduce(40), 80 => TE::Reduce(40), 81 => TE::Reduce(40), 82 => TE::Reduce(40), 85 => TE::Reduce(40), 89 => TE::Reduce(40), 95 => TE::Reduce(40), 97 => TE::Reduce(40) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 83 => TE::Reduce(43), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 81 => TE::Shift(226) },
    hashmap! { 53 => TE::Shift(227), 81 => TE::Reduce(46) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 86 => TE::Shift(231), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 40 => TE::Reduce(45), 44 => TE::Reduce(45), 45 => TE::Reduce(45), 46 => TE::Reduce(45), 47 => TE::Reduce(45), 48 => TE::Reduce(45), 49 => TE::Reduce(45), 50 => TE::Reduce(45), 52 => TE::Reduce(45), 53 => TE::Reduce(45), 54 => TE::Reduce(45), 55 => TE::Reduce(45), 65 => TE::Reduce(45), 66 => TE::Reduce(45), 67 => TE::Reduce(45), 68 => TE::Reduce(45), 69 => TE::Reduce(45), 70 => TE::Reduce(45), 71 => TE::Reduce(45), 72 => TE::Reduce(45), 73 => TE::Reduce(45), 74 => TE::Reduce(45), 75 => TE::Reduce(45), 76 => TE::Reduce(45), 77 => TE::Reduce(45), 78 => TE::Reduce(45), 79 => TE::Reduce(45), 80 => TE::Reduce(45), 81 => TE::Reduce(45), 82 => TE::Reduce(45), 85 => TE::Reduce(45), 89 => TE::Reduce(45), 95 => TE::Reduce(45), 97 => TE::Reduce(45) },
    hashmap! { 28 => TE::Transit(228), 29 => TE::Transit(90), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 79 => TE::Reduce(94), 82 => TE::Shift(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 56 => TE::Shift(96), 57 => TE::Shift(97), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 63 => TE::Shift(105), 86 => TE::Shift(229), 88 => TE::Shift(91), 89 => TE::Shift(92), 90 => TE::Shift(93), 91 => TE::Shift(94), 92 => TE::Shift(95), 93 => TE::Shift(98), 94 => TE::Shift(99), 95 => TE::Shift(106), 98 => TE::Shift(107) },
    hashmap! { 9 => TE::Transit(50), 11 => TE::Transit(230), 12 => TE::Transit(42), 13 => TE::Transit(43), 14 => TE::Transit(46), 15 => TE::Transit(41), 17 => TE::Transit(47), 18 => TE::Transit(48), 21 => TE::Transit(49), 24 => TE::Transit(44), 25 => TE::Transit(45), 27 => TE::Transit(40), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 36 => TE::Transit(39), 37 => TE::Transit(17), 38 => TE::Transit(32), 40 => TE::Shift(23), 44 => TE::Shift(73), 45 => TE::Shift(74), 46 => TE::Shift(77), 47 => TE::Shift(72), 49 => TE::Shift(78), 50 => TE::Shift(79), 52 => TE::Shift(52), 54 => TE::Shift(75), 55 => TE::Shift(76), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 75 => TE::Shift(19), 76 => TE::Shift(20), 77 => TE::Shift(21), 78 => TE::Shift(22), 79 => TE::Reduce(94), 80 => TE::Shift(35), 82 => TE::Shift(58), 85 => TE::Reduce(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 53 => TE::Reduce(48), 81 => TE::Reduce(48) },
    hashmap! { 9 => TE::Transit(50), 11 => TE::Transit(232), 12 => TE::Transit(42), 13 => TE::Transit(43), 14 => TE::Transit(46), 15 => TE::Transit(41), 17 => TE::Transit(47), 18 => TE::Transit(48), 21 => TE::Transit(49), 24 => TE::Transit(44), 25 => TE::Transit(45), 27 => TE::Transit(40), 28 => TE::Transit(53), 29 => TE::Transit(51), 30 => TE::Transit(54), 31 => TE::Transit(55), 32 => TE::Transit(56), 33 => TE::Transit(70), 36 => TE::Transit(39), 37 => TE::Transit(17), 38 => TE::Transit(32), 40 => TE::Shift(23), 44 => TE::Shift(73), 45 => TE::Shift(74), 46 => TE::Shift(77), 47 => TE::Shift(72), 49 => TE::Shift(78), 50 => TE::Shift(79), 52 => TE::Shift(52), 54 => TE::Shift(75), 55 => TE::Shift(76), 65 => TE::Shift(61), 66 => TE::Shift(62), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(71), 75 => TE::Shift(19), 76 => TE::Shift(20), 77 => TE::Shift(21), 78 => TE::Shift(22), 79 => TE::Reduce(94), 80 => TE::Shift(35), 82 => TE::Shift(58), 85 => TE::Reduce(58), 89 => TE::Shift(59), 95 => TE::Shift(57), 97 => TE::Shift(60) },
    hashmap! { 53 => TE::Reduce(49), 81 => TE::Reduce(49) },
    hashmap! { 37 => TE::Transit(234), 38 => TE::Transit(32), 40 => TE::Shift(23), 75 => TE::Shift(19), 76 => TE::Shift(20), 77 => TE::Shift(21), 78 => TE::Shift(22) },
    hashmap! { 83 => TE::Reduce(16), 84 => TE::Reduce(16) },
    hashmap! { 39 => TE::Transit(236), 79 => TE::Shift(9), 95 => TE::Shift(27) },
    hashmap! { 82 => TE::Shift(237) },
    hashmap! { 7 => TE::Transit(238), 8 => TE::Transit(30), 37 => TE::Transit(31), 38 => TE::Transit(32), 40 => TE::Shift(23), 75 => TE::Shift(19), 76 => TE::Shift(20), 77 => TE::Shift(21), 78 => TE::Shift(22), 83 => TE::Reduce(15) },
    hashmap! { 83 => TE::Shift(239) },
    hashmap! { 9 => TE::Transit(240), 80 => TE::Shift(35) },
    hashmap! { 40 => TE::Reduce(12), 43 => TE::Reduce(12), 75 => TE::Reduce(12), 76 => TE::Reduce(12), 77 => TE::Reduce(12), 78 => TE::Reduce(12), 81 => TE::Reduce(12) },
    hashmap! { 80 => TE::Reduce(7) }
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

fn gen_unary(opt: Token, mut opr: Sem, kind: Operator) -> Sem {
    Sem {
        loc: opt.get_loc(),
        value: SemValue::Expr(Expr::Unary(Unary {
            loc: opt.get_loc(),
            opt: kind,
            opr: Box::new(get_move!(opr, Expr)),
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
    Const(Const),
    Sealed(bool),
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

    handlers: [fn(&mut Tokenizer) -> &'static str; 79],
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
    Tokenizer::_lex_rule70,
    Tokenizer::_lex_rule71,
    Tokenizer::_lex_rule72,
    Tokenizer::_lex_rule73,
    Tokenizer::_lex_rule74,
    Tokenizer::_lex_rule75,
    Tokenizer::_lex_rule76,
    Tokenizer::_lex_rule77,
    Tokenizer::_lex_rule78
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
            if let Some(matched) = self._match(str_slice, &REGEX_RULES[i]) {

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
return "TRUE";
}

fn _lex_rule7(&mut self) -> &'static str {
return "FALSE";
}

fn _lex_rule8(&mut self) -> &'static str {
return "CLASS";
}

fn _lex_rule9(&mut self) -> &'static str {
return "EXTENDS";
}

fn _lex_rule10(&mut self) -> &'static str {
return "THIS";
}

fn _lex_rule11(&mut self) -> &'static str {
return "WHILE";
}

fn _lex_rule12(&mut self) -> &'static str {
return "FOREACH";
}

fn _lex_rule13(&mut self) -> &'static str {
return "FOR";
}

fn _lex_rule14(&mut self) -> &'static str {
return "IF";
}

fn _lex_rule15(&mut self) -> &'static str {
return "ELSE";
}

fn _lex_rule16(&mut self) -> &'static str {
return "RETURN";
}

fn _lex_rule17(&mut self) -> &'static str {
return "BREAK";
}

fn _lex_rule18(&mut self) -> &'static str {
return "PRINT";
}

fn _lex_rule19(&mut self) -> &'static str {
return "READ_INTEGER";
}

fn _lex_rule20(&mut self) -> &'static str {
return "READ_LINE";
}

fn _lex_rule21(&mut self) -> &'static str {
return "STATIC";
}

fn _lex_rule22(&mut self) -> &'static str {
return "INSTANCEOF";
}

fn _lex_rule23(&mut self) -> &'static str {
return "SCOPY";
}

fn _lex_rule24(&mut self) -> &'static str {
return "SEALED";
}

fn _lex_rule25(&mut self) -> &'static str {
return "VAR";
}

fn _lex_rule26(&mut self) -> &'static str {
return "DEFAULT";
}

fn _lex_rule27(&mut self) -> &'static str {
return "IN";
}

fn _lex_rule28(&mut self) -> &'static str {
return "GUARD_SPLIT";
}

fn _lex_rule29(&mut self) -> &'static str {
return "LESS_EQUAL";
}

fn _lex_rule30(&mut self) -> &'static str {
return "GREATER_EQUAL";
}

fn _lex_rule31(&mut self) -> &'static str {
return "EQUAL";
}

fn _lex_rule32(&mut self) -> &'static str {
return "NOT_EQUAL";
}

fn _lex_rule33(&mut self) -> &'static str {
return "AND";
}

fn _lex_rule34(&mut self) -> &'static str {
return "OR";
}

fn _lex_rule35(&mut self) -> &'static str {
return "REPEAT";
}

fn _lex_rule36(&mut self) -> &'static str {
return "CONCAT";
}

fn _lex_rule37(&mut self) -> &'static str {
return "'+'";
}

fn _lex_rule38(&mut self) -> &'static str {
return "'-'";
}

fn _lex_rule39(&mut self) -> &'static str {
return "'*'";
}

fn _lex_rule40(&mut self) -> &'static str {
return "'/'";
}

fn _lex_rule41(&mut self) -> &'static str {
return "'%'";
}

fn _lex_rule42(&mut self) -> &'static str {
return "'='";
}

fn _lex_rule43(&mut self) -> &'static str {
return "'<'";
}

fn _lex_rule44(&mut self) -> &'static str {
return "'>'";
}

fn _lex_rule45(&mut self) -> &'static str {
return "'.'";
}

fn _lex_rule46(&mut self) -> &'static str {
return "','";
}

fn _lex_rule47(&mut self) -> &'static str {
return "';'";
}

fn _lex_rule48(&mut self) -> &'static str {
return "'!'";
}

fn _lex_rule49(&mut self) -> &'static str {
return "'('";
}

fn _lex_rule50(&mut self) -> &'static str {
return "')'";
}

fn _lex_rule51(&mut self) -> &'static str {
return "'['";
}

fn _lex_rule52(&mut self) -> &'static str {
return "']'";
}

fn _lex_rule53(&mut self) -> &'static str {
return "'{'";
}

fn _lex_rule54(&mut self) -> &'static str {
return "'}'";
}

fn _lex_rule55(&mut self) -> &'static str {
return "':'";
}

fn _lex_rule56(&mut self) -> &'static str {
return "STRING_CONST";
}

fn _lex_rule57(&mut self) -> &'static str {
return "";
}

fn _lex_rule58(&mut self) -> &'static str {
return "INT_CONST";
}

fn _lex_rule59(&mut self) -> &'static str {
return "IDENTIFIER";
}

fn _lex_rule60(&mut self) -> &'static str {
return "'{'";
}

fn _lex_rule61(&mut self) -> &'static str {
return "'}'";
}

fn _lex_rule62(&mut self) -> &'static str {
return "'('";
}

fn _lex_rule63(&mut self) -> &'static str {
return "')'";
}

fn _lex_rule64(&mut self) -> &'static str {
return "','";
}

fn _lex_rule65(&mut self) -> &'static str {
return "';'";
}

fn _lex_rule66(&mut self) -> &'static str {
return "':'";
}

fn _lex_rule67(&mut self) -> &'static str {
return "'='";
}

fn _lex_rule68(&mut self) -> &'static str {
return "'+'";
}

fn _lex_rule69(&mut self) -> &'static str {
return "'-'";
}

fn _lex_rule70(&mut self) -> &'static str {
return "'*'";
}

fn _lex_rule71(&mut self) -> &'static str {
return "'/'";
}

fn _lex_rule72(&mut self) -> &'static str {
return "'%'";
}

fn _lex_rule73(&mut self) -> &'static str {
return "'<'";
}

fn _lex_rule74(&mut self) -> &'static str {
return "'>'";
}

fn _lex_rule75(&mut self) -> &'static str {
return "'['";
}

fn _lex_rule76(&mut self) -> &'static str {
return "']'";
}

fn _lex_rule77(&mut self) -> &'static str {
return "'!'";
}

fn _lex_rule78(&mut self) -> &'static str {
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
    handlers: [fn(&mut Parser) -> SV; 117],
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
    Parser::_handler76,
    Parser::_handler77,
    Parser::_handler78,
    Parser::_handler79,
    Parser::_handler80,
    Parser::_handler81,
    Parser::_handler82,
    Parser::_handler83,
    Parser::_handler84,
    Parser::_handler85,
    Parser::_handler86,
    Parser::_handler87,
    Parser::_handler88,
    Parser::_handler89,
    Parser::_handler90,
    Parser::_handler91,
    Parser::_handler92,
    Parser::_handler93,
    Parser::_handler94,
    Parser::_handler95,
    Parser::_handler96,
    Parser::_handler97,
    Parser::_handler98,
    Parser::_handler99,
    Parser::_handler100,
    Parser::_handler101,
    Parser::_handler102,
    Parser::_handler103,
    Parser::_handler104,
    Parser::_handler105,
    Parser::_handler106,
    Parser::_handler107,
    Parser::_handler108,
    Parser::_handler109,
    Parser::_handler110,
    Parser::_handler111,
    Parser::_handler112,
    Parser::_handler113,
    Parser::_handler114,
    Parser::_handler115,
    Parser::_handler116
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
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler61(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: _1.loc,
            value: SemValue::Expr(Expr::Const(get_move!(_1, Const))),
        };
SV::_1(__)
}

fn _handler62(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Add);
SV::_1(__)
}

fn _handler63(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Sub);
SV::_1(__)
}

fn _handler64(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Mul);
SV::_1(__)
}

fn _handler65(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Div);
SV::_1(__)
}

fn _handler66(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Mod);
SV::_1(__)
}

fn _handler67(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Eq);
SV::_1(__)
}

fn _handler68(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Ne);
SV::_1(__)
}

fn _handler69(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Lt);
SV::_1(__)
}

fn _handler70(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Gt);
SV::_1(__)
}

fn _handler71(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Le);
SV::_1(__)
}

fn _handler72(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Ge);
SV::_1(__)
}

fn _handler73(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::And);
SV::_1(__)
}

fn _handler74(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Or);
SV::_1(__)
}

fn _handler75(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Repeat);
SV::_1(__)
}

fn _handler76(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = gen_binary(_1, _2, _3, Operator::Concat);
SV::_1(__)
}

fn _handler77(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _5 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: _2.get_loc(),
            value: SemValue::Expr(Expr::Range(Range {
                loc: _2.get_loc(),
                array: Box::new(get_move!(_1, Expr)),
                lower: Box::new(get_move!(_3, Expr)),
                upper: Box::new(get_move!(_5, Expr)),
            }))
        };
SV::_1(__)
}

fn _handler78(&mut self) -> SV {
// Semantic values prologue.
let mut _6 = pop!(self.values_stack, _1);
self.values_stack.pop();
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _1);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: _2.get_loc(),
            value: SemValue::Expr(Expr::Default(Default {
                loc: _2.get_loc(),
                array: Box::new(get_move!(_1, Expr)),
                index: Box::new(get_move!(_3, Expr)),
                default: Box::new(get_move!(_6, Expr)),
            }))
        };
SV::_1(__)
}

fn _handler79(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _6 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _4 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Expr(Expr::Comprehension(Comprehension {
                loc: _1.get_loc(),
                expr: Box::new(get_move!(_2, Expr)),
                name: get_move!(_4, Identifier),
                array: Box::new(get_move!(_6, Expr)),
                cond: None,
            })),
        };
SV::_1(__)
}

fn _handler80(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _8 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _6 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _4 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Expr(Expr::Comprehension(Comprehension {
                loc: _1.get_loc(),
                expr: Box::new(get_move!(_2, Expr)),
                name: get_move!(_4, Identifier),
                array: Box::new(get_move!(_6, Expr)),
                cond: Some(Box::new(get_move!(_8, Expr))),
            })),
        };
SV::_1(__)
}

fn _handler81(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _1);
self.values_stack.pop();

let __ = _2;
SV::_1(__)
}

fn _handler82(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _0);

let __ = gen_unary(_1, _2, Operator::Neg);
SV::_1(__)
}

fn _handler83(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _0);

let __ = gen_unary(_1, _2, Operator::Not);
SV::_1(__)
}

fn _handler84(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Expr(Expr::ReadInt(ReadInt {
                loc: _1.get_loc(),
            }))
        };
SV::_1(__)
}

fn _handler85(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Expr(Expr::ReadLine(ReadLine {
                loc: _1.get_loc(),
            }))
        };
SV::_1(__)
}

fn _handler86(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Expr(Expr::This(This {
                loc: _1.get_loc(),
            }))
        };
SV::_1(__)
}

fn _handler87(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Expr(Expr::NewClass(NewClass {
                loc: _1.get_loc(),
                name: get_move!(_2, Identifier),
            }))
        };
SV::_1(__)
}

fn _handler88(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _4 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Expr(Expr::NewArray(NewArray {
                loc: _1.get_loc(),
                type_: get_move!(_2, Type),
                len: Box::new(get_move!(_4, Expr)),
            }))
        };
SV::_1(__)
}

fn _handler89(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _5 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Expr(Expr::TypeTest(TypeTest {
                loc: _1.get_loc(),
                expr: Box::new(get_move!(_3, Expr)),
                name: get_move!(_5, Identifier),
            }))
        };
SV::_1(__)
}

fn _handler90(&mut self) -> SV {
// Semantic values prologue.
let mut _5 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _1);
self.values_stack.pop();
self.values_stack.pop();

let __ = Sem {
            loc: _3.loc,
            value: SemValue::Expr(Expr::TypeCast(TypeCast {
                loc: _3.loc,
                name: get_move!(_3, Identifier),
                expr: Box::new(get_move!(_5, Expr)),
            }))
        };
SV::_1(__)
}

fn _handler91(&mut self) -> SV {
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

fn _handler92(&mut self) -> SV {
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

fn _handler93(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler94(&mut self) -> SV {
// Semantic values prologue.


let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::None,
        };
SV::_1(__)
}

fn _handler95(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _4 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: _2.loc,
            value: SemValue::Expr(Expr::Call(Call {
                loc: _2.loc,
                receiver: match _1.value {
                    SemValue::Expr(expr) => Some(Box::new(expr)),
                    SemValue::None => None,
                    _ => unreachable!(),
                },
                name: get_move!(_2, Identifier),
                arguments: get_move!(_4, ExprList),
            })),
        };
SV::_1(__)
}

fn _handler96(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Const(Const::IntConst(IntConst {
                loc: _1.get_loc(),
                value: self.tokenizer.yytext.parse::<i32>().unwrap(),
            })),
        };
SV::_1(__)
}

fn _handler97(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Const(Const::BoolConst(BoolConst {
                loc: _1.get_loc(),
                value: true,
            })),
        };
SV::_1(__)
}

fn _handler98(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Const(Const::BoolConst(BoolConst {
                loc: _1.get_loc(),
                value: false,
            })),
        };
SV::_1(__)
}

fn _handler99(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Const(Const::StringConst(StringConst {
                loc: _1.get_loc(),
                value: (&self.tokenizer.yytext[1..self.tokenizer.yytext.len() - 1]).to_string(),
            })),
        };
SV::_1(__)
}

fn _handler100(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler101(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Const(Const::Null(Null {
                loc: _1.get_loc(),
            })),
        };
SV::_1(__)
}

fn _handler102(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _1);
self.values_stack.pop();

let __ = _2;
SV::_1(__)
}

fn _handler103(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
self.values_stack.pop();

let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::Const(Const::ArrayConst(ArrayConst {
                loc: NO_LOCATION,
                value: Vec::new(),
            })),
        };
SV::_1(__)
}

fn _handler104(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _1);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _1);

let mut ret = _1;
        match get_ref!(ret, Const) {
            Const::ArrayConst(array_const) => array_const.value.push(get_move!(_3, Const)),
            _ => unreachable!(),
        };
        let __ = ret;
SV::_1(__)
}

fn _handler105(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::Const(Const::ArrayConst(ArrayConst {
                loc: NO_LOCATION,
                value: vec![get_move!(_1, Const)],
            })),
        };
SV::_1(__)
}

fn _handler106(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let __ = _1;
SV::_1(__)
}

fn _handler107(&mut self) -> SV {
// Semantic values prologue.


let __ = Sem {
            loc: NO_LOCATION,
            value: SemValue::ExprList(Vec::new()),
        };
SV::_1(__)
}

fn _handler108(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = self.values_stack.pop().unwrap();

let __ = _1;
__
}

fn _handler109(&mut self) -> SV {
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

fn _handler110(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Type(Type::Basic("int")),
        };
SV::_1(__)
}

fn _handler111(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Type(Type::Basic("void")),
        };
SV::_1(__)
}

fn _handler112(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Type(Type::Basic("bool")),
        };
SV::_1(__)
}

fn _handler113(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Type(Type::Basic("string")),
        };
SV::_1(__)
}

fn _handler114(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _1);
let mut _1 = pop!(self.values_stack, _0);

let __ = Sem {
            loc: _1.get_loc(),
            value: SemValue::Type(Type::Class(get_move!(_2, Identifier))),
        };
SV::_1(__)
}

fn _handler115(&mut self) -> SV {
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

fn _handler116(&mut self) -> SV {
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
