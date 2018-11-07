#![allow(dead_code)]
#![allow(unused_mut)]

extern crate regex;

#[macro_use]
extern crate lazy_static;

use regex::Regex;
use std::collections::HashMap;

// Stack value.
enum SV {
    Undefined,
    _0(Token),
    _1(ClassList),
    _2(Result<Program, Vec<Error>>),
    _3(ClassDef),
    _4(Flag),
    _5(Option<String>),
    _6(FieldList),
    _7(VarDef),
    _8(MethodDef),
    _9(Type),
    _10(VarDefList),
    _11(Block),
    _12(StatementList),
    _13(Statement),
    _14(Simple),
    _15(Expr),
    _16(Option<Statement>),
    _17(Option<Expr>),
    _18(GuardedList),
    _19(ExprList),
    _20(LValue),
    _21(Const),
    _22(ConstList)
}

// Lex rules.
static LEX_RULES: [&'static str; 89] = [
    r##########"^void"##########,
    r##########"^int"##########,
    r##########"^bool"##########,
    r##########"^string"##########,
    r##########"^new"##########,
    r##########"^null"##########,
    r##########"^true"##########,
    r##########"^false"##########,
    r##########"^class"##########,
    r##########"^extends"##########,
    r##########"^this"##########,
    r##########"^while"##########,
    r##########"^foreach"##########,
    r##########"^for"##########,
    r##########"^if"##########,
    r##########"^else"##########,
    r##########"^return"##########,
    r##########"^break"##########,
    r##########"^Print"##########,
    r##########"^ReadInteger"##########,
    r##########"^ReadLine"##########,
    r##########"^static"##########,
    r##########"^instanceof"##########,
    r##########"^scopy"##########,
    r##########"^sealed"##########,
    r##########"^var"##########,
    r##########"^default"##########,
    r##########"^in"##########,
    r##########"^\|\|\|"##########,
    r##########"^<="##########,
    r##########"^>="##########,
    r##########"^=="##########,
    r##########"^!="##########,
    r##########"^&&"##########,
    r##########"^\|\|"##########,
    r##########"^%%"##########,
    r##########"^\+\+"##########,
    r##########"^\+"##########,
    r##########"^-"##########,
    r##########"^\*"##########,
    r##########"^/"##########,
    r##########"^%"##########,
    r##########"^="##########,
    r##########"^<"##########,
    r##########"^>"##########,
    r##########"^\."##########,
    r##########"^,"##########,
    r##########"^;"##########,
    r##########"^!"##########,
    r##########"^\("##########,
    r##########"^\)"##########,
    r##########"^\["##########,
    r##########"^\]"##########,
    r##########"^\{"##########,
    r##########"^\}"##########,
    r##########"^:"##########,
    r##########"^""##########,
    r##########"^\n"##########,
    r##########"^\r"##########,
    r##########"^$"##########,
    r##########"^""##########,
    r##########"^\\n"##########,
    r##########"^\\t"##########,
    r##########"^\\\u0022"##########,
    r##########"^\\\\"##########,
    r##########"^."##########,
    r##########"^\u002f\u002f[^\n]*"##########,
    r##########"^\s+"##########,
    r##########"^\d+"##########,
    r##########"^[A-Za-z][_0-9A-Za-z]*"##########,
    r##########"^\{"##########,
    r##########"^\}"##########,
    r##########"^;"##########,
    r##########"^\("##########,
    r##########"^\)"##########,
    r##########"^,"##########,
    r##########"^:"##########,
    r##########"^="##########,
    r##########"^\+"##########,
    r##########"^\-"##########,
    r##########"^\*"##########,
    r##########"^/"##########,
    r##########"^%"##########,
    r##########"^<"##########,
    r##########"^>"##########,
    r##########"^\["##########,
    r##########"^\]"##########,
    r##########"^!"##########,
    r##########"^\."##########
];

// EOF value.
static EOF: &'static str = "$";

// A macro for map literals.
// usage: hashmap!{ 1 => "one", 2 => "two" };
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

// Unwraps a SV for the result. The result type is known from the grammar.
macro_rules! get_result {
    ($r:expr, $ty:ident) => (match $r { SV::$ty(v) => v, _ => unreachable!() });
}

// Pops a SV with needed enum value.
macro_rules! pop {
    ($s:expr, $ty:ident) => (get_result!($s.pop().unwrap(), $ty));
}

// Productions data.
// 0 - encoded non-terminal, 1 - length of RHS to pop from the stack
static PRODUCTIONS : [[i32; 2]; 115] = [
    [-1, 1],
    [0, 1],
    [1, 2],
    [1, 1],
    [2, 7],
    [3, 1],
    [3, 0],
    [4, 2],
    [4, 0],
    [5, 3],
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
    [11, 2],
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
    [37, 1],
    [37, 1],
    [37, 1],
    [37, 1],
    [37, 2],
    [37, 3]
];

// Table entry.
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
    // Lexical rules grouped by lexer state (by start condition).
    static ref LEX_RULES_BY_START_CONDITIONS: HashMap<&'static str, Vec<i32>> = hashmap! { "INITIAL" => vec! [ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88 ], "S" => vec! [ 57, 58, 59, 60, 61, 62, 63, 64, 65 ] };

    // Maps a string name of a token type to its encoded number (the first
    // token number starts after all numbers for non-terminal).
    static ref TOKENS_MAP: HashMap<&'static str, i32> = hashmap! { "CLASS" => 38, "IDENTIFIER" => 39, "SEALED" => 40, "EXTENDS" => 41, "STATIC" => 42, "WHILE" => 43, "FOR" => 44, "BREAK" => 45, "IF" => 46, "ELSE" => 47, "SCOPY" => 48, "FOREACH" => 49, "IN" => 50, "VAR" => 51, "GUARD_SPLIT" => 52, "RETURN" => 53, "PRINT" => 54, "EQUAL" => 55, "NOT_EQUAL" => 56, "LESS_EQUAL" => 57, "GREATER_EQUAL" => 58, "AND" => 59, "OR" => 60, "REPEAT" => 61, "CONCAT" => 62, "DEFAULT" => 63, "READ_INTEGER" => 64, "READ_LINE" => 65, "THIS" => 66, "NEW" => 67, "INSTANCEOF" => 68, "INT_CONST" => 69, "TRUE" => 70, "FALSE" => 71, "STRING_CONST" => 72, "NULL" => 73, "INT" => 74, "VOID" => 75, "BOOL" => 76, "STRING" => 77, "'{'" => 78, "'}'" => 79, "';'" => 80, "'('" => 81, "')'" => 82, "','" => 83, "':'" => 84, "'='" => 85, "'+'" => 86, "'-'" => 87, "'*'" => 88, "'/'" => 89, "'%'" => 90, "'<'" => 91, "'>'" => 92, "'['" => 93, "']'" => 94, "'!'" => 95, "'.'" => 96, "$" => 97 };

    // Parsing table.
    // Vector index is the state number, value is a map
    // from an encoded symbol to table entry (TE).
    static ref TABLE: Vec<HashMap<i32, TE>>= vec![
    hashmap! { 0 => TE::Transit(1), 1 => TE::Transit(2), 2 => TE::Transit(3), 3 => TE::Transit(4), 38 => TE::Reduce(6), 40 => TE::Shift(5) },
    hashmap! { 97 => TE::Accept },
    hashmap! { 2 => TE::Transit(6), 3 => TE::Transit(4), 38 => TE::Reduce(6), 40 => TE::Shift(5), 97 => TE::Reduce(1) },
    hashmap! { 38 => TE::Reduce(3), 40 => TE::Reduce(3), 97 => TE::Reduce(3) },
    hashmap! { 38 => TE::Shift(7) },
    hashmap! { 38 => TE::Reduce(5) },
    hashmap! { 38 => TE::Reduce(2), 40 => TE::Reduce(2), 97 => TE::Reduce(2) },
    hashmap! { 39 => TE::Shift(8) },
    hashmap! { 4 => TE::Transit(9), 41 => TE::Shift(10), 78 => TE::Reduce(8) },
    hashmap! { 78 => TE::Shift(11) },
    hashmap! { 39 => TE::Shift(240) },
    hashmap! { 5 => TE::Transit(12), 38 => TE::Reduce(11), 42 => TE::Reduce(11), 74 => TE::Reduce(11), 75 => TE::Reduce(11), 76 => TE::Reduce(11), 77 => TE::Reduce(11), 79 => TE::Reduce(11) },
    hashmap! { 6 => TE::Transit(15), 36 => TE::Transit(14), 37 => TE::Transit(16), 38 => TE::Shift(21), 42 => TE::Shift(22), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 79 => TE::Shift(13) },
    hashmap! { 38 => TE::Reduce(4), 40 => TE::Reduce(4), 97 => TE::Reduce(4) },
    hashmap! { 80 => TE::Shift(23) },
    hashmap! { 38 => TE::Reduce(10), 42 => TE::Reduce(10), 74 => TE::Reduce(10), 75 => TE::Reduce(10), 76 => TE::Reduce(10), 77 => TE::Reduce(10), 79 => TE::Reduce(10) },
    hashmap! { 39 => TE::Shift(24), 93 => TE::Shift(25) },
    hashmap! { 39 => TE::Reduce(109), 93 => TE::Reduce(109) },
    hashmap! { 39 => TE::Reduce(110), 93 => TE::Reduce(110) },
    hashmap! { 39 => TE::Reduce(111), 93 => TE::Reduce(111) },
    hashmap! { 39 => TE::Reduce(112), 93 => TE::Reduce(112) },
    hashmap! { 39 => TE::Shift(86) },
    hashmap! { 37 => TE::Transit(234), 38 => TE::Shift(21), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20) },
    hashmap! { 38 => TE::Reduce(9), 42 => TE::Reduce(9), 74 => TE::Reduce(9), 75 => TE::Reduce(9), 76 => TE::Reduce(9), 77 => TE::Reduce(9), 79 => TE::Reduce(9) },
    hashmap! { 80 => TE::Reduce(108), 81 => TE::Shift(26) },
    hashmap! { 94 => TE::Shift(85) },
    hashmap! { 7 => TE::Transit(27), 8 => TE::Transit(28), 36 => TE::Transit(29), 37 => TE::Transit(30), 38 => TE::Shift(21), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 82 => TE::Reduce(15) },
    hashmap! { 82 => TE::Shift(31) },
    hashmap! { 82 => TE::Reduce(14), 83 => TE::Shift(232) },
    hashmap! { 82 => TE::Reduce(17), 83 => TE::Reduce(17) },
    hashmap! { 39 => TE::Shift(84), 93 => TE::Shift(25) },
    hashmap! { 9 => TE::Transit(32), 78 => TE::Shift(33) },
    hashmap! { 38 => TE::Reduce(13), 42 => TE::Reduce(13), 74 => TE::Reduce(13), 75 => TE::Reduce(13), 76 => TE::Reduce(13), 77 => TE::Reduce(13), 79 => TE::Reduce(13) },
    hashmap! { 10 => TE::Transit(34), 38 => TE::Reduce(20), 39 => TE::Reduce(20), 43 => TE::Reduce(20), 44 => TE::Reduce(20), 45 => TE::Reduce(20), 46 => TE::Reduce(20), 48 => TE::Reduce(20), 49 => TE::Reduce(20), 51 => TE::Reduce(20), 53 => TE::Reduce(20), 54 => TE::Reduce(20), 64 => TE::Reduce(20), 65 => TE::Reduce(20), 66 => TE::Reduce(20), 67 => TE::Reduce(20), 68 => TE::Reduce(20), 69 => TE::Reduce(20), 70 => TE::Reduce(20), 71 => TE::Reduce(20), 72 => TE::Reduce(20), 73 => TE::Reduce(20), 74 => TE::Reduce(20), 75 => TE::Reduce(20), 76 => TE::Reduce(20), 77 => TE::Reduce(20), 78 => TE::Reduce(20), 79 => TE::Reduce(20), 80 => TE::Reduce(20), 81 => TE::Reduce(20), 87 => TE::Reduce(20), 93 => TE::Reduce(20), 95 => TE::Reduce(20) },
    hashmap! { 9 => TE::Transit(48), 11 => TE::Transit(36), 12 => TE::Transit(40), 13 => TE::Transit(41), 14 => TE::Transit(44), 15 => TE::Transit(39), 17 => TE::Transit(45), 18 => TE::Transit(46), 21 => TE::Transit(47), 24 => TE::Transit(42), 25 => TE::Transit(43), 27 => TE::Transit(38), 28 => TE::Transit(51), 29 => TE::Transit(49), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 36 => TE::Transit(37), 37 => TE::Transit(30), 38 => TE::Shift(21), 39 => TE::Reduce(94), 43 => TE::Shift(71), 44 => TE::Shift(72), 45 => TE::Shift(75), 46 => TE::Shift(70), 48 => TE::Shift(76), 49 => TE::Shift(77), 51 => TE::Shift(50), 53 => TE::Shift(73), 54 => TE::Shift(74), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 78 => TE::Shift(33), 79 => TE::Shift(35), 80 => TE::Reduce(58), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 38 => TE::Reduce(18), 39 => TE::Reduce(18), 42 => TE::Reduce(18), 43 => TE::Reduce(18), 44 => TE::Reduce(18), 45 => TE::Reduce(18), 46 => TE::Reduce(18), 47 => TE::Reduce(18), 48 => TE::Reduce(18), 49 => TE::Reduce(18), 51 => TE::Reduce(18), 52 => TE::Reduce(18), 53 => TE::Reduce(18), 54 => TE::Reduce(18), 64 => TE::Reduce(18), 65 => TE::Reduce(18), 66 => TE::Reduce(18), 67 => TE::Reduce(18), 68 => TE::Reduce(18), 69 => TE::Reduce(18), 70 => TE::Reduce(18), 71 => TE::Reduce(18), 72 => TE::Reduce(18), 73 => TE::Reduce(18), 74 => TE::Reduce(18), 75 => TE::Reduce(18), 76 => TE::Reduce(18), 77 => TE::Reduce(18), 78 => TE::Reduce(18), 79 => TE::Reduce(18), 80 => TE::Reduce(18), 81 => TE::Reduce(18), 87 => TE::Reduce(18), 93 => TE::Reduce(18), 95 => TE::Reduce(18) },
    hashmap! { 38 => TE::Reduce(19), 39 => TE::Reduce(19), 43 => TE::Reduce(19), 44 => TE::Reduce(19), 45 => TE::Reduce(19), 46 => TE::Reduce(19), 48 => TE::Reduce(19), 49 => TE::Reduce(19), 51 => TE::Reduce(19), 53 => TE::Reduce(19), 54 => TE::Reduce(19), 64 => TE::Reduce(19), 65 => TE::Reduce(19), 66 => TE::Reduce(19), 67 => TE::Reduce(19), 68 => TE::Reduce(19), 69 => TE::Reduce(19), 70 => TE::Reduce(19), 71 => TE::Reduce(19), 72 => TE::Reduce(19), 73 => TE::Reduce(19), 74 => TE::Reduce(19), 75 => TE::Reduce(19), 76 => TE::Reduce(19), 77 => TE::Reduce(19), 78 => TE::Reduce(19), 79 => TE::Reduce(19), 80 => TE::Reduce(19), 81 => TE::Reduce(19), 87 => TE::Reduce(19), 93 => TE::Reduce(19), 95 => TE::Reduce(19) },
    hashmap! { 80 => TE::Shift(78) },
    hashmap! { 80 => TE::Shift(79) },
    hashmap! { 38 => TE::Reduce(23), 39 => TE::Reduce(23), 43 => TE::Reduce(23), 44 => TE::Reduce(23), 45 => TE::Reduce(23), 46 => TE::Reduce(23), 47 => TE::Reduce(23), 48 => TE::Reduce(23), 49 => TE::Reduce(23), 51 => TE::Reduce(23), 52 => TE::Reduce(23), 53 => TE::Reduce(23), 54 => TE::Reduce(23), 64 => TE::Reduce(23), 65 => TE::Reduce(23), 66 => TE::Reduce(23), 67 => TE::Reduce(23), 68 => TE::Reduce(23), 69 => TE::Reduce(23), 70 => TE::Reduce(23), 71 => TE::Reduce(23), 72 => TE::Reduce(23), 73 => TE::Reduce(23), 74 => TE::Reduce(23), 75 => TE::Reduce(23), 76 => TE::Reduce(23), 77 => TE::Reduce(23), 78 => TE::Reduce(23), 79 => TE::Reduce(23), 80 => TE::Reduce(23), 81 => TE::Reduce(23), 87 => TE::Reduce(23), 93 => TE::Reduce(23), 95 => TE::Reduce(23) },
    hashmap! { 38 => TE::Reduce(24), 39 => TE::Reduce(24), 43 => TE::Reduce(24), 44 => TE::Reduce(24), 45 => TE::Reduce(24), 46 => TE::Reduce(24), 47 => TE::Reduce(24), 48 => TE::Reduce(24), 49 => TE::Reduce(24), 51 => TE::Reduce(24), 52 => TE::Reduce(24), 53 => TE::Reduce(24), 54 => TE::Reduce(24), 64 => TE::Reduce(24), 65 => TE::Reduce(24), 66 => TE::Reduce(24), 67 => TE::Reduce(24), 68 => TE::Reduce(24), 69 => TE::Reduce(24), 70 => TE::Reduce(24), 71 => TE::Reduce(24), 72 => TE::Reduce(24), 73 => TE::Reduce(24), 74 => TE::Reduce(24), 75 => TE::Reduce(24), 76 => TE::Reduce(24), 77 => TE::Reduce(24), 78 => TE::Reduce(24), 79 => TE::Reduce(24), 80 => TE::Reduce(24), 81 => TE::Reduce(24), 87 => TE::Reduce(24), 93 => TE::Reduce(24), 95 => TE::Reduce(24) },
    hashmap! { 38 => TE::Reduce(25), 39 => TE::Reduce(25), 43 => TE::Reduce(25), 44 => TE::Reduce(25), 45 => TE::Reduce(25), 46 => TE::Reduce(25), 47 => TE::Reduce(25), 48 => TE::Reduce(25), 49 => TE::Reduce(25), 51 => TE::Reduce(25), 52 => TE::Reduce(25), 53 => TE::Reduce(25), 54 => TE::Reduce(25), 64 => TE::Reduce(25), 65 => TE::Reduce(25), 66 => TE::Reduce(25), 67 => TE::Reduce(25), 68 => TE::Reduce(25), 69 => TE::Reduce(25), 70 => TE::Reduce(25), 71 => TE::Reduce(25), 72 => TE::Reduce(25), 73 => TE::Reduce(25), 74 => TE::Reduce(25), 75 => TE::Reduce(25), 76 => TE::Reduce(25), 77 => TE::Reduce(25), 78 => TE::Reduce(25), 79 => TE::Reduce(25), 80 => TE::Reduce(25), 81 => TE::Reduce(25), 87 => TE::Reduce(25), 93 => TE::Reduce(25), 95 => TE::Reduce(25) },
    hashmap! { 80 => TE::Shift(80) },
    hashmap! { 80 => TE::Shift(81) },
    hashmap! { 80 => TE::Shift(82) },
    hashmap! { 80 => TE::Shift(83) },
    hashmap! { 38 => TE::Reduce(30), 39 => TE::Reduce(30), 43 => TE::Reduce(30), 44 => TE::Reduce(30), 45 => TE::Reduce(30), 46 => TE::Reduce(30), 47 => TE::Reduce(30), 48 => TE::Reduce(30), 49 => TE::Reduce(30), 51 => TE::Reduce(30), 52 => TE::Reduce(30), 53 => TE::Reduce(30), 54 => TE::Reduce(30), 64 => TE::Reduce(30), 65 => TE::Reduce(30), 66 => TE::Reduce(30), 67 => TE::Reduce(30), 68 => TE::Reduce(30), 69 => TE::Reduce(30), 70 => TE::Reduce(30), 71 => TE::Reduce(30), 72 => TE::Reduce(30), 73 => TE::Reduce(30), 74 => TE::Reduce(30), 75 => TE::Reduce(30), 76 => TE::Reduce(30), 77 => TE::Reduce(30), 78 => TE::Reduce(30), 79 => TE::Reduce(30), 80 => TE::Reduce(30), 81 => TE::Reduce(30), 87 => TE::Reduce(30), 93 => TE::Reduce(30), 95 => TE::Reduce(30) },
    hashmap! { 38 => TE::Reduce(31), 39 => TE::Reduce(31), 43 => TE::Reduce(31), 44 => TE::Reduce(31), 45 => TE::Reduce(31), 46 => TE::Reduce(31), 47 => TE::Reduce(31), 48 => TE::Reduce(31), 49 => TE::Reduce(31), 51 => TE::Reduce(31), 52 => TE::Reduce(31), 53 => TE::Reduce(31), 54 => TE::Reduce(31), 64 => TE::Reduce(31), 65 => TE::Reduce(31), 66 => TE::Reduce(31), 67 => TE::Reduce(31), 68 => TE::Reduce(31), 69 => TE::Reduce(31), 70 => TE::Reduce(31), 71 => TE::Reduce(31), 72 => TE::Reduce(31), 73 => TE::Reduce(31), 74 => TE::Reduce(31), 75 => TE::Reduce(31), 76 => TE::Reduce(31), 77 => TE::Reduce(31), 78 => TE::Reduce(31), 79 => TE::Reduce(31), 80 => TE::Reduce(31), 81 => TE::Reduce(31), 87 => TE::Reduce(31), 93 => TE::Reduce(31), 95 => TE::Reduce(31) },
    hashmap! { 38 => TE::Reduce(32), 39 => TE::Reduce(32), 43 => TE::Reduce(32), 44 => TE::Reduce(32), 45 => TE::Reduce(32), 46 => TE::Reduce(32), 47 => TE::Reduce(32), 48 => TE::Reduce(32), 49 => TE::Reduce(32), 51 => TE::Reduce(32), 52 => TE::Reduce(32), 53 => TE::Reduce(32), 54 => TE::Reduce(32), 64 => TE::Reduce(32), 65 => TE::Reduce(32), 66 => TE::Reduce(32), 67 => TE::Reduce(32), 68 => TE::Reduce(32), 69 => TE::Reduce(32), 70 => TE::Reduce(32), 71 => TE::Reduce(32), 72 => TE::Reduce(32), 73 => TE::Reduce(32), 74 => TE::Reduce(32), 75 => TE::Reduce(32), 76 => TE::Reduce(32), 77 => TE::Reduce(32), 78 => TE::Reduce(32), 79 => TE::Reduce(32), 80 => TE::Reduce(32), 81 => TE::Reduce(32), 87 => TE::Reduce(32), 93 => TE::Reduce(32), 95 => TE::Reduce(32) },
    hashmap! { 55 => TE::Reduce(59), 56 => TE::Reduce(59), 57 => TE::Reduce(59), 58 => TE::Reduce(59), 59 => TE::Reduce(59), 60 => TE::Reduce(59), 61 => TE::Reduce(59), 62 => TE::Reduce(59), 80 => TE::Reduce(59), 82 => TE::Reduce(59), 85 => TE::Shift(87), 86 => TE::Reduce(59), 87 => TE::Reduce(59), 88 => TE::Reduce(59), 89 => TE::Reduce(59), 90 => TE::Reduce(59), 91 => TE::Reduce(59), 92 => TE::Reduce(59), 93 => TE::Reduce(59), 96 => TE::Reduce(59) },
    hashmap! { 39 => TE::Shift(178) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 80 => TE::Reduce(57), 82 => TE::Reduce(57), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 39 => TE::Shift(163) },
    hashmap! { 43 => TE::Reduce(60), 44 => TE::Reduce(60), 46 => TE::Reduce(60), 55 => TE::Reduce(60), 56 => TE::Reduce(60), 57 => TE::Reduce(60), 58 => TE::Reduce(60), 59 => TE::Reduce(60), 60 => TE::Reduce(60), 61 => TE::Reduce(60), 62 => TE::Reduce(60), 80 => TE::Reduce(60), 82 => TE::Reduce(60), 83 => TE::Reduce(60), 84 => TE::Reduce(60), 86 => TE::Reduce(60), 87 => TE::Reduce(60), 88 => TE::Reduce(60), 89 => TE::Reduce(60), 90 => TE::Reduce(60), 91 => TE::Reduce(60), 92 => TE::Reduce(60), 93 => TE::Reduce(60), 94 => TE::Reduce(60), 96 => TE::Reduce(60) },
    hashmap! { 43 => TE::Reduce(61), 44 => TE::Reduce(61), 46 => TE::Reduce(61), 55 => TE::Reduce(61), 56 => TE::Reduce(61), 57 => TE::Reduce(61), 58 => TE::Reduce(61), 59 => TE::Reduce(61), 60 => TE::Reduce(61), 61 => TE::Reduce(61), 62 => TE::Reduce(61), 80 => TE::Reduce(61), 82 => TE::Reduce(61), 83 => TE::Reduce(61), 84 => TE::Reduce(61), 86 => TE::Reduce(61), 87 => TE::Reduce(61), 88 => TE::Reduce(61), 89 => TE::Reduce(61), 90 => TE::Reduce(61), 91 => TE::Reduce(61), 92 => TE::Reduce(61), 93 => TE::Reduce(61), 94 => TE::Reduce(61), 96 => TE::Reduce(61) },
    hashmap! { 28 => TE::Transit(127), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(130), 33 => TE::Transit(68), 34 => TE::Transit(128), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 94 => TE::Shift(129), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(139), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 38 => TE::Shift(140), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(145), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(146), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 81 => TE::Shift(147) },
    hashmap! { 81 => TE::Shift(149) },
    hashmap! { 43 => TE::Reduce(86), 44 => TE::Reduce(86), 46 => TE::Reduce(86), 55 => TE::Reduce(86), 56 => TE::Reduce(86), 57 => TE::Reduce(86), 58 => TE::Reduce(86), 59 => TE::Reduce(86), 60 => TE::Reduce(86), 61 => TE::Reduce(86), 62 => TE::Reduce(86), 80 => TE::Reduce(86), 82 => TE::Reduce(86), 83 => TE::Reduce(86), 84 => TE::Reduce(86), 86 => TE::Reduce(86), 87 => TE::Reduce(86), 88 => TE::Reduce(86), 89 => TE::Reduce(86), 90 => TE::Reduce(86), 91 => TE::Reduce(86), 92 => TE::Reduce(86), 93 => TE::Reduce(86), 94 => TE::Reduce(86), 96 => TE::Reduce(86) },
    hashmap! { 37 => TE::Transit(152), 38 => TE::Shift(21), 39 => TE::Shift(151), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20) },
    hashmap! { 81 => TE::Shift(158) },
    hashmap! { 43 => TE::Reduce(96), 44 => TE::Reduce(96), 46 => TE::Reduce(96), 55 => TE::Reduce(96), 56 => TE::Reduce(96), 57 => TE::Reduce(96), 58 => TE::Reduce(96), 59 => TE::Reduce(96), 60 => TE::Reduce(96), 61 => TE::Reduce(96), 62 => TE::Reduce(96), 80 => TE::Reduce(96), 82 => TE::Reduce(96), 83 => TE::Reduce(96), 84 => TE::Reduce(96), 86 => TE::Reduce(96), 87 => TE::Reduce(96), 88 => TE::Reduce(96), 89 => TE::Reduce(96), 90 => TE::Reduce(96), 91 => TE::Reduce(96), 92 => TE::Reduce(96), 93 => TE::Reduce(96), 94 => TE::Reduce(96), 96 => TE::Reduce(96) },
    hashmap! { 43 => TE::Reduce(97), 44 => TE::Reduce(97), 46 => TE::Reduce(97), 55 => TE::Reduce(97), 56 => TE::Reduce(97), 57 => TE::Reduce(97), 58 => TE::Reduce(97), 59 => TE::Reduce(97), 60 => TE::Reduce(97), 61 => TE::Reduce(97), 62 => TE::Reduce(97), 80 => TE::Reduce(97), 82 => TE::Reduce(97), 83 => TE::Reduce(97), 84 => TE::Reduce(97), 86 => TE::Reduce(97), 87 => TE::Reduce(97), 88 => TE::Reduce(97), 89 => TE::Reduce(97), 90 => TE::Reduce(97), 91 => TE::Reduce(97), 92 => TE::Reduce(97), 93 => TE::Reduce(97), 94 => TE::Reduce(97), 96 => TE::Reduce(97) },
    hashmap! { 43 => TE::Reduce(98), 44 => TE::Reduce(98), 46 => TE::Reduce(98), 55 => TE::Reduce(98), 56 => TE::Reduce(98), 57 => TE::Reduce(98), 58 => TE::Reduce(98), 59 => TE::Reduce(98), 60 => TE::Reduce(98), 61 => TE::Reduce(98), 62 => TE::Reduce(98), 80 => TE::Reduce(98), 82 => TE::Reduce(98), 83 => TE::Reduce(98), 84 => TE::Reduce(98), 86 => TE::Reduce(98), 87 => TE::Reduce(98), 88 => TE::Reduce(98), 89 => TE::Reduce(98), 90 => TE::Reduce(98), 91 => TE::Reduce(98), 92 => TE::Reduce(98), 93 => TE::Reduce(98), 94 => TE::Reduce(98), 96 => TE::Reduce(98) },
    hashmap! { 43 => TE::Reduce(99), 44 => TE::Reduce(99), 46 => TE::Reduce(99), 55 => TE::Reduce(99), 56 => TE::Reduce(99), 57 => TE::Reduce(99), 58 => TE::Reduce(99), 59 => TE::Reduce(99), 60 => TE::Reduce(99), 61 => TE::Reduce(99), 62 => TE::Reduce(99), 80 => TE::Reduce(99), 82 => TE::Reduce(99), 83 => TE::Reduce(99), 84 => TE::Reduce(99), 86 => TE::Reduce(99), 87 => TE::Reduce(99), 88 => TE::Reduce(99), 89 => TE::Reduce(99), 90 => TE::Reduce(99), 91 => TE::Reduce(99), 92 => TE::Reduce(99), 93 => TE::Reduce(99), 94 => TE::Reduce(99), 96 => TE::Reduce(99) },
    hashmap! { 43 => TE::Reduce(100), 44 => TE::Reduce(100), 46 => TE::Reduce(100), 55 => TE::Reduce(100), 56 => TE::Reduce(100), 57 => TE::Reduce(100), 58 => TE::Reduce(100), 59 => TE::Reduce(100), 60 => TE::Reduce(100), 61 => TE::Reduce(100), 62 => TE::Reduce(100), 80 => TE::Reduce(100), 82 => TE::Reduce(100), 83 => TE::Reduce(100), 84 => TE::Reduce(100), 86 => TE::Reduce(100), 87 => TE::Reduce(100), 88 => TE::Reduce(100), 89 => TE::Reduce(100), 90 => TE::Reduce(100), 91 => TE::Reduce(100), 92 => TE::Reduce(100), 93 => TE::Reduce(100), 94 => TE::Reduce(100), 96 => TE::Reduce(100) },
    hashmap! { 43 => TE::Reduce(101), 44 => TE::Reduce(101), 46 => TE::Reduce(101), 55 => TE::Reduce(101), 56 => TE::Reduce(101), 57 => TE::Reduce(101), 58 => TE::Reduce(101), 59 => TE::Reduce(101), 60 => TE::Reduce(101), 61 => TE::Reduce(101), 62 => TE::Reduce(101), 80 => TE::Reduce(101), 82 => TE::Reduce(101), 83 => TE::Reduce(101), 84 => TE::Reduce(101), 86 => TE::Reduce(101), 87 => TE::Reduce(101), 88 => TE::Reduce(101), 89 => TE::Reduce(101), 90 => TE::Reduce(101), 91 => TE::Reduce(101), 92 => TE::Reduce(101), 93 => TE::Reduce(101), 94 => TE::Reduce(101), 96 => TE::Reduce(101) },
    hashmap! { 78 => TE::Shift(182), 81 => TE::Shift(181) },
    hashmap! { 81 => TE::Shift(189) },
    hashmap! { 81 => TE::Shift(193) },
    hashmap! { 28 => TE::Transit(201), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 80 => TE::Reduce(51), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 81 => TE::Shift(202) },
    hashmap! { 80 => TE::Reduce(35) },
    hashmap! { 81 => TE::Shift(205) },
    hashmap! { 81 => TE::Shift(210) },
    hashmap! { 38 => TE::Reduce(21), 39 => TE::Reduce(21), 43 => TE::Reduce(21), 44 => TE::Reduce(21), 45 => TE::Reduce(21), 46 => TE::Reduce(21), 47 => TE::Reduce(21), 48 => TE::Reduce(21), 49 => TE::Reduce(21), 51 => TE::Reduce(21), 52 => TE::Reduce(21), 53 => TE::Reduce(21), 54 => TE::Reduce(21), 64 => TE::Reduce(21), 65 => TE::Reduce(21), 66 => TE::Reduce(21), 67 => TE::Reduce(21), 68 => TE::Reduce(21), 69 => TE::Reduce(21), 70 => TE::Reduce(21), 71 => TE::Reduce(21), 72 => TE::Reduce(21), 73 => TE::Reduce(21), 74 => TE::Reduce(21), 75 => TE::Reduce(21), 76 => TE::Reduce(21), 77 => TE::Reduce(21), 78 => TE::Reduce(21), 79 => TE::Reduce(21), 80 => TE::Reduce(21), 81 => TE::Reduce(21), 87 => TE::Reduce(21), 93 => TE::Reduce(21), 95 => TE::Reduce(21) },
    hashmap! { 38 => TE::Reduce(22), 39 => TE::Reduce(22), 43 => TE::Reduce(22), 44 => TE::Reduce(22), 45 => TE::Reduce(22), 46 => TE::Reduce(22), 47 => TE::Reduce(22), 48 => TE::Reduce(22), 49 => TE::Reduce(22), 51 => TE::Reduce(22), 52 => TE::Reduce(22), 53 => TE::Reduce(22), 54 => TE::Reduce(22), 64 => TE::Reduce(22), 65 => TE::Reduce(22), 66 => TE::Reduce(22), 67 => TE::Reduce(22), 68 => TE::Reduce(22), 69 => TE::Reduce(22), 70 => TE::Reduce(22), 71 => TE::Reduce(22), 72 => TE::Reduce(22), 73 => TE::Reduce(22), 74 => TE::Reduce(22), 75 => TE::Reduce(22), 76 => TE::Reduce(22), 77 => TE::Reduce(22), 78 => TE::Reduce(22), 79 => TE::Reduce(22), 80 => TE::Reduce(22), 81 => TE::Reduce(22), 87 => TE::Reduce(22), 93 => TE::Reduce(22), 95 => TE::Reduce(22) },
    hashmap! { 38 => TE::Reduce(26), 39 => TE::Reduce(26), 43 => TE::Reduce(26), 44 => TE::Reduce(26), 45 => TE::Reduce(26), 46 => TE::Reduce(26), 47 => TE::Reduce(26), 48 => TE::Reduce(26), 49 => TE::Reduce(26), 51 => TE::Reduce(26), 52 => TE::Reduce(26), 53 => TE::Reduce(26), 54 => TE::Reduce(26), 64 => TE::Reduce(26), 65 => TE::Reduce(26), 66 => TE::Reduce(26), 67 => TE::Reduce(26), 68 => TE::Reduce(26), 69 => TE::Reduce(26), 70 => TE::Reduce(26), 71 => TE::Reduce(26), 72 => TE::Reduce(26), 73 => TE::Reduce(26), 74 => TE::Reduce(26), 75 => TE::Reduce(26), 76 => TE::Reduce(26), 77 => TE::Reduce(26), 78 => TE::Reduce(26), 79 => TE::Reduce(26), 80 => TE::Reduce(26), 81 => TE::Reduce(26), 87 => TE::Reduce(26), 93 => TE::Reduce(26), 95 => TE::Reduce(26) },
    hashmap! { 38 => TE::Reduce(27), 39 => TE::Reduce(27), 43 => TE::Reduce(27), 44 => TE::Reduce(27), 45 => TE::Reduce(27), 46 => TE::Reduce(27), 47 => TE::Reduce(27), 48 => TE::Reduce(27), 49 => TE::Reduce(27), 51 => TE::Reduce(27), 52 => TE::Reduce(27), 53 => TE::Reduce(27), 54 => TE::Reduce(27), 64 => TE::Reduce(27), 65 => TE::Reduce(27), 66 => TE::Reduce(27), 67 => TE::Reduce(27), 68 => TE::Reduce(27), 69 => TE::Reduce(27), 70 => TE::Reduce(27), 71 => TE::Reduce(27), 72 => TE::Reduce(27), 73 => TE::Reduce(27), 74 => TE::Reduce(27), 75 => TE::Reduce(27), 76 => TE::Reduce(27), 77 => TE::Reduce(27), 78 => TE::Reduce(27), 79 => TE::Reduce(27), 80 => TE::Reduce(27), 81 => TE::Reduce(27), 87 => TE::Reduce(27), 93 => TE::Reduce(27), 95 => TE::Reduce(27) },
    hashmap! { 38 => TE::Reduce(28), 39 => TE::Reduce(28), 43 => TE::Reduce(28), 44 => TE::Reduce(28), 45 => TE::Reduce(28), 46 => TE::Reduce(28), 47 => TE::Reduce(28), 48 => TE::Reduce(28), 49 => TE::Reduce(28), 51 => TE::Reduce(28), 52 => TE::Reduce(28), 53 => TE::Reduce(28), 54 => TE::Reduce(28), 64 => TE::Reduce(28), 65 => TE::Reduce(28), 66 => TE::Reduce(28), 67 => TE::Reduce(28), 68 => TE::Reduce(28), 69 => TE::Reduce(28), 70 => TE::Reduce(28), 71 => TE::Reduce(28), 72 => TE::Reduce(28), 73 => TE::Reduce(28), 74 => TE::Reduce(28), 75 => TE::Reduce(28), 76 => TE::Reduce(28), 77 => TE::Reduce(28), 78 => TE::Reduce(28), 79 => TE::Reduce(28), 80 => TE::Reduce(28), 81 => TE::Reduce(28), 87 => TE::Reduce(28), 93 => TE::Reduce(28), 95 => TE::Reduce(28) },
    hashmap! { 38 => TE::Reduce(29), 39 => TE::Reduce(29), 43 => TE::Reduce(29), 44 => TE::Reduce(29), 45 => TE::Reduce(29), 46 => TE::Reduce(29), 47 => TE::Reduce(29), 48 => TE::Reduce(29), 49 => TE::Reduce(29), 51 => TE::Reduce(29), 52 => TE::Reduce(29), 53 => TE::Reduce(29), 54 => TE::Reduce(29), 64 => TE::Reduce(29), 65 => TE::Reduce(29), 66 => TE::Reduce(29), 67 => TE::Reduce(29), 68 => TE::Reduce(29), 69 => TE::Reduce(29), 70 => TE::Reduce(29), 71 => TE::Reduce(29), 72 => TE::Reduce(29), 73 => TE::Reduce(29), 74 => TE::Reduce(29), 75 => TE::Reduce(29), 76 => TE::Reduce(29), 77 => TE::Reduce(29), 78 => TE::Reduce(29), 79 => TE::Reduce(29), 80 => TE::Reduce(29), 81 => TE::Reduce(29), 87 => TE::Reduce(29), 93 => TE::Reduce(29), 95 => TE::Reduce(29) },
    hashmap! { 80 => TE::Reduce(108), 82 => TE::Reduce(108), 83 => TE::Reduce(108) },
    hashmap! { 39 => TE::Reduce(114), 93 => TE::Reduce(114) },
    hashmap! { 39 => TE::Reduce(113), 93 => TE::Reduce(113) },
    hashmap! { 28 => TE::Transit(88), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 80 => TE::Reduce(55), 82 => TE::Reduce(55), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(59), 44 => TE::Reduce(59), 46 => TE::Reduce(59), 55 => TE::Reduce(59), 56 => TE::Reduce(59), 57 => TE::Reduce(59), 58 => TE::Reduce(59), 59 => TE::Reduce(59), 60 => TE::Reduce(59), 61 => TE::Reduce(59), 62 => TE::Reduce(59), 80 => TE::Reduce(59), 82 => TE::Reduce(59), 83 => TE::Reduce(59), 84 => TE::Reduce(59), 86 => TE::Reduce(59), 87 => TE::Reduce(59), 88 => TE::Reduce(59), 89 => TE::Reduce(59), 90 => TE::Reduce(59), 91 => TE::Reduce(59), 92 => TE::Reduce(59), 93 => TE::Reduce(59), 94 => TE::Reduce(59), 96 => TE::Reduce(59) },
    hashmap! { 28 => TE::Transit(107), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(108), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(109), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(110), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(111), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(112), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(113), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(114), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(115), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(116), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(117), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(118), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(119), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(120), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(121), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 28 => TE::Transit(122), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 39 => TE::Reduce(93) },
    hashmap! { 43 => TE::Reduce(62), 44 => TE::Reduce(62), 46 => TE::Reduce(62), 55 => TE::Reduce(62), 56 => TE::Reduce(62), 57 => TE::Reduce(62), 58 => TE::Reduce(62), 59 => TE::Reduce(62), 60 => TE::Reduce(62), 61 => TE::Reduce(62), 62 => TE::Reduce(62), 80 => TE::Reduce(62), 82 => TE::Reduce(62), 83 => TE::Reduce(62), 84 => TE::Reduce(62), 86 => TE::Reduce(62), 87 => TE::Reduce(62), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Reduce(62), 92 => TE::Reduce(62), 93 => TE::Shift(105), 94 => TE::Reduce(62), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(63), 44 => TE::Reduce(63), 46 => TE::Reduce(63), 55 => TE::Reduce(63), 56 => TE::Reduce(63), 57 => TE::Reduce(63), 58 => TE::Reduce(63), 59 => TE::Reduce(63), 60 => TE::Reduce(63), 61 => TE::Reduce(63), 62 => TE::Reduce(63), 80 => TE::Reduce(63), 82 => TE::Reduce(63), 83 => TE::Reduce(63), 84 => TE::Reduce(63), 86 => TE::Reduce(63), 87 => TE::Reduce(63), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Reduce(63), 92 => TE::Reduce(63), 93 => TE::Shift(105), 94 => TE::Reduce(63), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(64), 44 => TE::Reduce(64), 46 => TE::Reduce(64), 55 => TE::Reduce(64), 56 => TE::Reduce(64), 57 => TE::Reduce(64), 58 => TE::Reduce(64), 59 => TE::Reduce(64), 60 => TE::Reduce(64), 61 => TE::Reduce(64), 62 => TE::Reduce(64), 80 => TE::Reduce(64), 82 => TE::Reduce(64), 83 => TE::Reduce(64), 84 => TE::Reduce(64), 86 => TE::Reduce(64), 87 => TE::Reduce(64), 88 => TE::Reduce(64), 89 => TE::Reduce(64), 90 => TE::Reduce(64), 91 => TE::Reduce(64), 92 => TE::Reduce(64), 93 => TE::Shift(105), 94 => TE::Reduce(64), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(65), 44 => TE::Reduce(65), 46 => TE::Reduce(65), 55 => TE::Reduce(65), 56 => TE::Reduce(65), 57 => TE::Reduce(65), 58 => TE::Reduce(65), 59 => TE::Reduce(65), 60 => TE::Reduce(65), 61 => TE::Reduce(65), 62 => TE::Reduce(65), 80 => TE::Reduce(65), 82 => TE::Reduce(65), 83 => TE::Reduce(65), 84 => TE::Reduce(65), 86 => TE::Reduce(65), 87 => TE::Reduce(65), 88 => TE::Reduce(65), 89 => TE::Reduce(65), 90 => TE::Reduce(65), 91 => TE::Reduce(65), 92 => TE::Reduce(65), 93 => TE::Shift(105), 94 => TE::Reduce(65), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(66), 44 => TE::Reduce(66), 46 => TE::Reduce(66), 55 => TE::Reduce(66), 56 => TE::Reduce(66), 57 => TE::Reduce(66), 58 => TE::Reduce(66), 59 => TE::Reduce(66), 60 => TE::Reduce(66), 61 => TE::Reduce(66), 62 => TE::Reduce(66), 80 => TE::Reduce(66), 82 => TE::Reduce(66), 83 => TE::Reduce(66), 84 => TE::Reduce(66), 86 => TE::Reduce(66), 87 => TE::Reduce(66), 88 => TE::Reduce(66), 89 => TE::Reduce(66), 90 => TE::Reduce(66), 91 => TE::Reduce(66), 92 => TE::Reduce(66), 93 => TE::Shift(105), 94 => TE::Reduce(66), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(67), 44 => TE::Reduce(67), 46 => TE::Reduce(67), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Reduce(67), 60 => TE::Reduce(67), 61 => TE::Shift(103), 62 => TE::Shift(104), 80 => TE::Reduce(67), 82 => TE::Reduce(67), 83 => TE::Reduce(67), 84 => TE::Reduce(67), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 94 => TE::Reduce(67), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(68), 44 => TE::Reduce(68), 46 => TE::Reduce(68), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Reduce(68), 60 => TE::Reduce(68), 61 => TE::Shift(103), 62 => TE::Shift(104), 80 => TE::Reduce(68), 82 => TE::Reduce(68), 83 => TE::Reduce(68), 84 => TE::Reduce(68), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 94 => TE::Reduce(68), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(69), 44 => TE::Reduce(69), 46 => TE::Reduce(69), 55 => TE::Reduce(69), 56 => TE::Reduce(69), 59 => TE::Reduce(69), 60 => TE::Reduce(69), 61 => TE::Shift(103), 62 => TE::Shift(104), 80 => TE::Reduce(69), 82 => TE::Reduce(69), 83 => TE::Reduce(69), 84 => TE::Reduce(69), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 93 => TE::Shift(105), 94 => TE::Reduce(69), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(70), 44 => TE::Reduce(70), 46 => TE::Reduce(70), 55 => TE::Reduce(70), 56 => TE::Reduce(70), 59 => TE::Reduce(70), 60 => TE::Reduce(70), 61 => TE::Shift(103), 62 => TE::Shift(104), 80 => TE::Reduce(70), 82 => TE::Reduce(70), 83 => TE::Reduce(70), 84 => TE::Reduce(70), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 93 => TE::Shift(105), 94 => TE::Reduce(70), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(71), 44 => TE::Reduce(71), 46 => TE::Reduce(71), 55 => TE::Reduce(71), 56 => TE::Reduce(71), 59 => TE::Reduce(71), 60 => TE::Reduce(71), 61 => TE::Shift(103), 62 => TE::Shift(104), 80 => TE::Reduce(71), 82 => TE::Reduce(71), 83 => TE::Reduce(71), 84 => TE::Reduce(71), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 93 => TE::Shift(105), 94 => TE::Reduce(71), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(72), 44 => TE::Reduce(72), 46 => TE::Reduce(72), 55 => TE::Reduce(72), 56 => TE::Reduce(72), 59 => TE::Reduce(72), 60 => TE::Reduce(72), 61 => TE::Shift(103), 62 => TE::Shift(104), 80 => TE::Reduce(72), 82 => TE::Reduce(72), 83 => TE::Reduce(72), 84 => TE::Reduce(72), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 93 => TE::Shift(105), 94 => TE::Reduce(72), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(73), 44 => TE::Reduce(73), 46 => TE::Reduce(73), 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Reduce(73), 60 => TE::Reduce(73), 61 => TE::Shift(103), 62 => TE::Shift(104), 80 => TE::Reduce(73), 82 => TE::Reduce(73), 83 => TE::Reduce(73), 84 => TE::Reduce(73), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 94 => TE::Reduce(73), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(74), 44 => TE::Reduce(74), 46 => TE::Reduce(74), 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Reduce(74), 61 => TE::Shift(103), 62 => TE::Shift(104), 80 => TE::Reduce(74), 82 => TE::Reduce(74), 83 => TE::Reduce(74), 84 => TE::Reduce(74), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 94 => TE::Reduce(74), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(75), 44 => TE::Reduce(75), 46 => TE::Reduce(75), 55 => TE::Reduce(75), 56 => TE::Reduce(75), 57 => TE::Reduce(75), 58 => TE::Reduce(75), 59 => TE::Reduce(75), 60 => TE::Reduce(75), 61 => TE::Reduce(75), 62 => TE::Reduce(75), 80 => TE::Reduce(75), 82 => TE::Reduce(75), 83 => TE::Reduce(75), 84 => TE::Reduce(75), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Reduce(75), 92 => TE::Reduce(75), 93 => TE::Shift(105), 94 => TE::Reduce(75), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(76), 44 => TE::Reduce(76), 46 => TE::Reduce(76), 55 => TE::Reduce(76), 56 => TE::Reduce(76), 57 => TE::Reduce(76), 58 => TE::Reduce(76), 59 => TE::Reduce(76), 60 => TE::Reduce(76), 61 => TE::Shift(103), 62 => TE::Shift(104), 80 => TE::Reduce(76), 82 => TE::Reduce(76), 83 => TE::Reduce(76), 84 => TE::Reduce(76), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Reduce(76), 92 => TE::Reduce(76), 93 => TE::Shift(105), 94 => TE::Reduce(76), 96 => TE::Shift(106) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 84 => TE::Shift(123), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 94 => TE::Shift(124), 96 => TE::Shift(106) },
    hashmap! { 28 => TE::Transit(125), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 43 => TE::Reduce(92), 44 => TE::Reduce(92), 46 => TE::Reduce(92), 55 => TE::Reduce(92), 56 => TE::Reduce(92), 57 => TE::Reduce(92), 58 => TE::Reduce(92), 59 => TE::Reduce(92), 60 => TE::Reduce(92), 61 => TE::Reduce(92), 62 => TE::Reduce(92), 63 => TE::Shift(176), 80 => TE::Reduce(92), 82 => TE::Reduce(92), 83 => TE::Reduce(92), 84 => TE::Reduce(92), 85 => TE::Reduce(92), 86 => TE::Reduce(92), 87 => TE::Reduce(92), 88 => TE::Reduce(92), 89 => TE::Reduce(92), 90 => TE::Reduce(92), 91 => TE::Reduce(92), 92 => TE::Reduce(92), 93 => TE::Reduce(92), 94 => TE::Reduce(92), 96 => TE::Reduce(92) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 94 => TE::Shift(126), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(77), 44 => TE::Reduce(77), 46 => TE::Reduce(77), 55 => TE::Reduce(77), 56 => TE::Reduce(77), 57 => TE::Reduce(77), 58 => TE::Reduce(77), 59 => TE::Reduce(77), 60 => TE::Reduce(77), 61 => TE::Reduce(77), 62 => TE::Reduce(77), 80 => TE::Reduce(77), 82 => TE::Reduce(77), 83 => TE::Reduce(77), 84 => TE::Reduce(77), 86 => TE::Reduce(77), 87 => TE::Reduce(77), 88 => TE::Reduce(77), 89 => TE::Reduce(77), 90 => TE::Reduce(77), 91 => TE::Reduce(77), 92 => TE::Reduce(77), 93 => TE::Reduce(77), 94 => TE::Reduce(77), 96 => TE::Reduce(77) },
    hashmap! { 44 => TE::Shift(131), 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 83 => TE::Shift(172), 94 => TE::Shift(171) },
    hashmap! { 43 => TE::Reduce(103), 44 => TE::Reduce(103), 46 => TE::Reduce(103), 55 => TE::Reduce(103), 56 => TE::Reduce(103), 57 => TE::Reduce(103), 58 => TE::Reduce(103), 59 => TE::Reduce(103), 60 => TE::Reduce(103), 61 => TE::Reduce(103), 62 => TE::Reduce(103), 80 => TE::Reduce(103), 82 => TE::Reduce(103), 83 => TE::Reduce(103), 84 => TE::Reduce(103), 86 => TE::Reduce(103), 87 => TE::Reduce(103), 88 => TE::Reduce(103), 89 => TE::Reduce(103), 90 => TE::Reduce(103), 91 => TE::Reduce(103), 92 => TE::Reduce(103), 93 => TE::Reduce(103), 94 => TE::Reduce(103), 96 => TE::Reduce(103) },
    hashmap! { 44 => TE::Reduce(61), 55 => TE::Reduce(61), 56 => TE::Reduce(61), 57 => TE::Reduce(61), 58 => TE::Reduce(61), 59 => TE::Reduce(61), 60 => TE::Reduce(61), 61 => TE::Reduce(61), 62 => TE::Reduce(61), 83 => TE::Reduce(105), 86 => TE::Reduce(61), 87 => TE::Reduce(61), 88 => TE::Reduce(61), 89 => TE::Reduce(61), 90 => TE::Reduce(61), 91 => TE::Reduce(61), 92 => TE::Reduce(61), 93 => TE::Reduce(61), 94 => TE::Reduce(105), 96 => TE::Reduce(61) },
    hashmap! { 39 => TE::Shift(132) },
    hashmap! { 50 => TE::Shift(133) },
    hashmap! { 28 => TE::Transit(134), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 46 => TE::Shift(136), 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 94 => TE::Shift(135), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(79), 44 => TE::Reduce(79), 46 => TE::Reduce(79), 55 => TE::Reduce(79), 56 => TE::Reduce(79), 57 => TE::Reduce(79), 58 => TE::Reduce(79), 59 => TE::Reduce(79), 60 => TE::Reduce(79), 61 => TE::Reduce(79), 62 => TE::Reduce(79), 80 => TE::Reduce(79), 82 => TE::Reduce(79), 83 => TE::Reduce(79), 84 => TE::Reduce(79), 86 => TE::Reduce(79), 87 => TE::Reduce(79), 88 => TE::Reduce(79), 89 => TE::Reduce(79), 90 => TE::Reduce(79), 91 => TE::Reduce(79), 92 => TE::Reduce(79), 93 => TE::Reduce(79), 94 => TE::Reduce(79), 96 => TE::Reduce(79) },
    hashmap! { 28 => TE::Transit(137), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 94 => TE::Shift(138), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(80), 44 => TE::Reduce(80), 46 => TE::Reduce(80), 55 => TE::Reduce(80), 56 => TE::Reduce(80), 57 => TE::Reduce(80), 58 => TE::Reduce(80), 59 => TE::Reduce(80), 60 => TE::Reduce(80), 61 => TE::Reduce(80), 62 => TE::Reduce(80), 80 => TE::Reduce(80), 82 => TE::Reduce(80), 83 => TE::Reduce(80), 84 => TE::Reduce(80), 86 => TE::Reduce(80), 87 => TE::Reduce(80), 88 => TE::Reduce(80), 89 => TE::Reduce(80), 90 => TE::Reduce(80), 91 => TE::Reduce(80), 92 => TE::Reduce(80), 93 => TE::Reduce(80), 94 => TE::Reduce(80), 96 => TE::Reduce(80) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 82 => TE::Shift(141), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 39 => TE::Shift(142) },
    hashmap! { 43 => TE::Reduce(81), 44 => TE::Reduce(81), 46 => TE::Reduce(81), 55 => TE::Reduce(81), 56 => TE::Reduce(81), 57 => TE::Reduce(81), 58 => TE::Reduce(81), 59 => TE::Reduce(81), 60 => TE::Reduce(81), 61 => TE::Reduce(81), 62 => TE::Reduce(81), 80 => TE::Reduce(81), 82 => TE::Reduce(81), 83 => TE::Reduce(81), 84 => TE::Reduce(81), 86 => TE::Reduce(81), 87 => TE::Reduce(81), 88 => TE::Reduce(81), 89 => TE::Reduce(81), 90 => TE::Reduce(81), 91 => TE::Reduce(81), 92 => TE::Reduce(81), 93 => TE::Reduce(81), 94 => TE::Reduce(81), 96 => TE::Reduce(81) },
    hashmap! { 82 => TE::Shift(143) },
    hashmap! { 28 => TE::Transit(144), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 43 => TE::Reduce(90), 44 => TE::Reduce(90), 46 => TE::Reduce(90), 55 => TE::Reduce(90), 56 => TE::Reduce(90), 57 => TE::Reduce(90), 58 => TE::Reduce(90), 59 => TE::Reduce(90), 60 => TE::Reduce(90), 61 => TE::Reduce(90), 62 => TE::Reduce(90), 80 => TE::Reduce(90), 82 => TE::Reduce(90), 83 => TE::Reduce(90), 84 => TE::Reduce(90), 86 => TE::Reduce(90), 87 => TE::Reduce(90), 88 => TE::Reduce(90), 89 => TE::Reduce(90), 90 => TE::Reduce(90), 91 => TE::Reduce(90), 92 => TE::Reduce(90), 93 => TE::Reduce(90), 94 => TE::Reduce(90), 96 => TE::Reduce(90) },
    hashmap! { 43 => TE::Reduce(82), 44 => TE::Reduce(82), 46 => TE::Reduce(82), 55 => TE::Reduce(82), 56 => TE::Reduce(82), 57 => TE::Reduce(82), 58 => TE::Reduce(82), 59 => TE::Reduce(82), 60 => TE::Reduce(82), 61 => TE::Reduce(82), 62 => TE::Reduce(82), 80 => TE::Reduce(82), 82 => TE::Reduce(82), 83 => TE::Reduce(82), 84 => TE::Reduce(82), 86 => TE::Reduce(82), 87 => TE::Reduce(82), 88 => TE::Reduce(82), 89 => TE::Reduce(82), 90 => TE::Reduce(82), 91 => TE::Reduce(82), 92 => TE::Reduce(82), 93 => TE::Shift(105), 94 => TE::Reduce(82), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(83), 44 => TE::Reduce(83), 46 => TE::Reduce(83), 55 => TE::Reduce(83), 56 => TE::Reduce(83), 57 => TE::Reduce(83), 58 => TE::Reduce(83), 59 => TE::Reduce(83), 60 => TE::Reduce(83), 61 => TE::Reduce(83), 62 => TE::Reduce(83), 80 => TE::Reduce(83), 82 => TE::Reduce(83), 83 => TE::Reduce(83), 84 => TE::Reduce(83), 86 => TE::Reduce(83), 87 => TE::Reduce(83), 88 => TE::Reduce(83), 89 => TE::Reduce(83), 90 => TE::Reduce(83), 91 => TE::Reduce(83), 92 => TE::Reduce(83), 93 => TE::Shift(105), 94 => TE::Reduce(83), 96 => TE::Shift(106) },
    hashmap! { 82 => TE::Shift(148) },
    hashmap! { 43 => TE::Reduce(84), 44 => TE::Reduce(84), 46 => TE::Reduce(84), 55 => TE::Reduce(84), 56 => TE::Reduce(84), 57 => TE::Reduce(84), 58 => TE::Reduce(84), 59 => TE::Reduce(84), 60 => TE::Reduce(84), 61 => TE::Reduce(84), 62 => TE::Reduce(84), 80 => TE::Reduce(84), 82 => TE::Reduce(84), 83 => TE::Reduce(84), 84 => TE::Reduce(84), 86 => TE::Reduce(84), 87 => TE::Reduce(84), 88 => TE::Reduce(84), 89 => TE::Reduce(84), 90 => TE::Reduce(84), 91 => TE::Reduce(84), 92 => TE::Reduce(84), 93 => TE::Reduce(84), 94 => TE::Reduce(84), 96 => TE::Reduce(84) },
    hashmap! { 82 => TE::Shift(150) },
    hashmap! { 43 => TE::Reduce(85), 44 => TE::Reduce(85), 46 => TE::Reduce(85), 55 => TE::Reduce(85), 56 => TE::Reduce(85), 57 => TE::Reduce(85), 58 => TE::Reduce(85), 59 => TE::Reduce(85), 60 => TE::Reduce(85), 61 => TE::Reduce(85), 62 => TE::Reduce(85), 80 => TE::Reduce(85), 82 => TE::Reduce(85), 83 => TE::Reduce(85), 84 => TE::Reduce(85), 86 => TE::Reduce(85), 87 => TE::Reduce(85), 88 => TE::Reduce(85), 89 => TE::Reduce(85), 90 => TE::Reduce(85), 91 => TE::Reduce(85), 92 => TE::Reduce(85), 93 => TE::Reduce(85), 94 => TE::Reduce(85), 96 => TE::Reduce(85) },
    hashmap! { 81 => TE::Shift(153) },
    hashmap! { 93 => TE::Shift(155) },
    hashmap! { 82 => TE::Shift(154) },
    hashmap! { 43 => TE::Reduce(87), 44 => TE::Reduce(87), 46 => TE::Reduce(87), 55 => TE::Reduce(87), 56 => TE::Reduce(87), 57 => TE::Reduce(87), 58 => TE::Reduce(87), 59 => TE::Reduce(87), 60 => TE::Reduce(87), 61 => TE::Reduce(87), 62 => TE::Reduce(87), 80 => TE::Reduce(87), 82 => TE::Reduce(87), 83 => TE::Reduce(87), 84 => TE::Reduce(87), 86 => TE::Reduce(87), 87 => TE::Reduce(87), 88 => TE::Reduce(87), 89 => TE::Reduce(87), 90 => TE::Reduce(87), 91 => TE::Reduce(87), 92 => TE::Reduce(87), 93 => TE::Reduce(87), 94 => TE::Reduce(87), 96 => TE::Reduce(87) },
    hashmap! { 28 => TE::Transit(156), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 94 => TE::Shift(85), 95 => TE::Shift(58) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 94 => TE::Shift(157), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(88), 44 => TE::Reduce(88), 46 => TE::Reduce(88), 55 => TE::Reduce(88), 56 => TE::Reduce(88), 57 => TE::Reduce(88), 58 => TE::Reduce(88), 59 => TE::Reduce(88), 60 => TE::Reduce(88), 61 => TE::Reduce(88), 62 => TE::Reduce(88), 80 => TE::Reduce(88), 82 => TE::Reduce(88), 83 => TE::Reduce(88), 84 => TE::Reduce(88), 86 => TE::Reduce(88), 87 => TE::Reduce(88), 88 => TE::Reduce(88), 89 => TE::Reduce(88), 90 => TE::Reduce(88), 91 => TE::Reduce(88), 92 => TE::Reduce(88), 93 => TE::Reduce(88), 94 => TE::Reduce(88), 96 => TE::Reduce(88) },
    hashmap! { 28 => TE::Transit(159), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 83 => TE::Shift(160), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 39 => TE::Shift(161) },
    hashmap! { 82 => TE::Shift(162) },
    hashmap! { 43 => TE::Reduce(89), 44 => TE::Reduce(89), 46 => TE::Reduce(89), 55 => TE::Reduce(89), 56 => TE::Reduce(89), 57 => TE::Reduce(89), 58 => TE::Reduce(89), 59 => TE::Reduce(89), 60 => TE::Reduce(89), 61 => TE::Reduce(89), 62 => TE::Reduce(89), 80 => TE::Reduce(89), 82 => TE::Reduce(89), 83 => TE::Reduce(89), 84 => TE::Reduce(89), 86 => TE::Reduce(89), 87 => TE::Reduce(89), 88 => TE::Reduce(89), 89 => TE::Reduce(89), 90 => TE::Reduce(89), 91 => TE::Reduce(89), 92 => TE::Reduce(89), 93 => TE::Reduce(89), 94 => TE::Reduce(89), 96 => TE::Reduce(89) },
    hashmap! { 43 => TE::Reduce(91), 44 => TE::Reduce(91), 46 => TE::Reduce(91), 55 => TE::Reduce(91), 56 => TE::Reduce(91), 57 => TE::Reduce(91), 58 => TE::Reduce(91), 59 => TE::Reduce(91), 60 => TE::Reduce(91), 61 => TE::Reduce(91), 62 => TE::Reduce(91), 80 => TE::Reduce(91), 81 => TE::Shift(164), 82 => TE::Reduce(91), 83 => TE::Reduce(91), 84 => TE::Reduce(91), 85 => TE::Reduce(91), 86 => TE::Reduce(91), 87 => TE::Reduce(91), 88 => TE::Reduce(91), 89 => TE::Reduce(91), 90 => TE::Reduce(91), 91 => TE::Reduce(91), 92 => TE::Reduce(91), 93 => TE::Reduce(91), 94 => TE::Reduce(91), 96 => TE::Reduce(91) },
    hashmap! { 26 => TE::Transit(166), 28 => TE::Transit(167), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 35 => TE::Transit(165), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 82 => TE::Reduce(107), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 82 => TE::Shift(168) },
    hashmap! { 82 => TE::Reduce(106), 83 => TE::Shift(169) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 82 => TE::Reduce(54), 83 => TE::Reduce(54), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(95), 44 => TE::Reduce(95), 46 => TE::Reduce(95), 55 => TE::Reduce(95), 56 => TE::Reduce(95), 57 => TE::Reduce(95), 58 => TE::Reduce(95), 59 => TE::Reduce(95), 60 => TE::Reduce(95), 61 => TE::Reduce(95), 62 => TE::Reduce(95), 80 => TE::Reduce(95), 82 => TE::Reduce(95), 83 => TE::Reduce(95), 84 => TE::Reduce(95), 86 => TE::Reduce(95), 87 => TE::Reduce(95), 88 => TE::Reduce(95), 89 => TE::Reduce(95), 90 => TE::Reduce(95), 91 => TE::Reduce(95), 92 => TE::Reduce(95), 93 => TE::Reduce(95), 94 => TE::Reduce(95), 96 => TE::Reduce(95) },
    hashmap! { 28 => TE::Transit(170), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 82 => TE::Reduce(53), 83 => TE::Reduce(53), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 43 => TE::Reduce(102), 44 => TE::Reduce(102), 46 => TE::Reduce(102), 55 => TE::Reduce(102), 56 => TE::Reduce(102), 57 => TE::Reduce(102), 58 => TE::Reduce(102), 59 => TE::Reduce(102), 60 => TE::Reduce(102), 61 => TE::Reduce(102), 62 => TE::Reduce(102), 80 => TE::Reduce(102), 82 => TE::Reduce(102), 83 => TE::Reduce(102), 84 => TE::Reduce(102), 86 => TE::Reduce(102), 87 => TE::Reduce(102), 88 => TE::Reduce(102), 89 => TE::Reduce(102), 90 => TE::Reduce(102), 91 => TE::Reduce(102), 92 => TE::Reduce(102), 93 => TE::Reduce(102), 94 => TE::Reduce(102), 96 => TE::Reduce(102) },
    hashmap! { 32 => TE::Transit(173), 33 => TE::Transit(68), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 93 => TE::Shift(174) },
    hashmap! { 83 => TE::Reduce(104), 94 => TE::Reduce(104) },
    hashmap! { 32 => TE::Transit(175), 33 => TE::Transit(68), 34 => TE::Transit(128), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 93 => TE::Shift(174), 94 => TE::Shift(129) },
    hashmap! { 83 => TE::Reduce(105), 94 => TE::Reduce(105) },
    hashmap! { 28 => TE::Transit(177), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 43 => TE::Reduce(78), 44 => TE::Reduce(78), 46 => TE::Reduce(78), 55 => TE::Reduce(78), 56 => TE::Reduce(78), 57 => TE::Reduce(78), 58 => TE::Reduce(78), 59 => TE::Reduce(78), 60 => TE::Reduce(78), 61 => TE::Reduce(78), 62 => TE::Reduce(78), 80 => TE::Reduce(78), 82 => TE::Reduce(78), 83 => TE::Reduce(78), 84 => TE::Reduce(78), 86 => TE::Reduce(78), 87 => TE::Reduce(78), 88 => TE::Reduce(78), 89 => TE::Reduce(78), 90 => TE::Reduce(78), 91 => TE::Reduce(78), 92 => TE::Reduce(78), 94 => TE::Reduce(78) },
    hashmap! { 85 => TE::Shift(179) },
    hashmap! { 28 => TE::Transit(180), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 80 => TE::Reduce(56), 82 => TE::Reduce(56), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 28 => TE::Transit(183), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 22 => TE::Transit(222), 23 => TE::Transit(223), 28 => TE::Transit(224), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 79 => TE::Reduce(47), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 82 => TE::Shift(184), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 9 => TE::Transit(48), 11 => TE::Transit(185), 12 => TE::Transit(40), 13 => TE::Transit(41), 14 => TE::Transit(44), 15 => TE::Transit(39), 17 => TE::Transit(45), 18 => TE::Transit(46), 21 => TE::Transit(47), 24 => TE::Transit(42), 25 => TE::Transit(43), 27 => TE::Transit(38), 28 => TE::Transit(51), 29 => TE::Transit(49), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 36 => TE::Transit(37), 37 => TE::Transit(30), 38 => TE::Shift(21), 39 => TE::Reduce(94), 43 => TE::Shift(71), 44 => TE::Shift(72), 45 => TE::Shift(75), 46 => TE::Shift(70), 48 => TE::Shift(76), 49 => TE::Shift(77), 51 => TE::Shift(50), 53 => TE::Shift(73), 54 => TE::Shift(74), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 78 => TE::Shift(33), 80 => TE::Reduce(58), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 16 => TE::Transit(186), 38 => TE::Reduce(38), 39 => TE::Reduce(38), 43 => TE::Reduce(38), 44 => TE::Reduce(38), 45 => TE::Reduce(38), 46 => TE::Reduce(38), 47 => TE::Shift(187), 48 => TE::Reduce(38), 49 => TE::Reduce(38), 51 => TE::Reduce(38), 52 => TE::Reduce(38), 53 => TE::Reduce(38), 54 => TE::Reduce(38), 64 => TE::Reduce(38), 65 => TE::Reduce(38), 66 => TE::Reduce(38), 67 => TE::Reduce(38), 68 => TE::Reduce(38), 69 => TE::Reduce(38), 70 => TE::Reduce(38), 71 => TE::Reduce(38), 72 => TE::Reduce(38), 73 => TE::Reduce(38), 74 => TE::Reduce(38), 75 => TE::Reduce(38), 76 => TE::Reduce(38), 77 => TE::Reduce(38), 78 => TE::Reduce(38), 79 => TE::Reduce(38), 80 => TE::Reduce(38), 81 => TE::Reduce(38), 87 => TE::Reduce(38), 93 => TE::Reduce(38), 95 => TE::Reduce(38) },
    hashmap! { 38 => TE::Reduce(36), 39 => TE::Reduce(36), 43 => TE::Reduce(36), 44 => TE::Reduce(36), 45 => TE::Reduce(36), 46 => TE::Reduce(36), 47 => TE::Reduce(36), 48 => TE::Reduce(36), 49 => TE::Reduce(36), 51 => TE::Reduce(36), 52 => TE::Reduce(36), 53 => TE::Reduce(36), 54 => TE::Reduce(36), 64 => TE::Reduce(36), 65 => TE::Reduce(36), 66 => TE::Reduce(36), 67 => TE::Reduce(36), 68 => TE::Reduce(36), 69 => TE::Reduce(36), 70 => TE::Reduce(36), 71 => TE::Reduce(36), 72 => TE::Reduce(36), 73 => TE::Reduce(36), 74 => TE::Reduce(36), 75 => TE::Reduce(36), 76 => TE::Reduce(36), 77 => TE::Reduce(36), 78 => TE::Reduce(36), 79 => TE::Reduce(36), 80 => TE::Reduce(36), 81 => TE::Reduce(36), 87 => TE::Reduce(36), 93 => TE::Reduce(36), 95 => TE::Reduce(36) },
    hashmap! { 9 => TE::Transit(48), 11 => TE::Transit(188), 12 => TE::Transit(40), 13 => TE::Transit(41), 14 => TE::Transit(44), 15 => TE::Transit(39), 17 => TE::Transit(45), 18 => TE::Transit(46), 21 => TE::Transit(47), 24 => TE::Transit(42), 25 => TE::Transit(43), 27 => TE::Transit(38), 28 => TE::Transit(51), 29 => TE::Transit(49), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 36 => TE::Transit(37), 37 => TE::Transit(30), 38 => TE::Shift(21), 39 => TE::Reduce(94), 43 => TE::Shift(71), 44 => TE::Shift(72), 45 => TE::Shift(75), 46 => TE::Shift(70), 48 => TE::Shift(76), 49 => TE::Shift(77), 51 => TE::Shift(50), 53 => TE::Shift(73), 54 => TE::Shift(74), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 78 => TE::Shift(33), 80 => TE::Reduce(58), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 38 => TE::Reduce(37), 39 => TE::Reduce(37), 43 => TE::Reduce(37), 44 => TE::Reduce(37), 45 => TE::Reduce(37), 46 => TE::Reduce(37), 47 => TE::Reduce(37), 48 => TE::Reduce(37), 49 => TE::Reduce(37), 51 => TE::Reduce(37), 52 => TE::Reduce(37), 53 => TE::Reduce(37), 54 => TE::Reduce(37), 64 => TE::Reduce(37), 65 => TE::Reduce(37), 66 => TE::Reduce(37), 67 => TE::Reduce(37), 68 => TE::Reduce(37), 69 => TE::Reduce(37), 70 => TE::Reduce(37), 71 => TE::Reduce(37), 72 => TE::Reduce(37), 73 => TE::Reduce(37), 74 => TE::Reduce(37), 75 => TE::Reduce(37), 76 => TE::Reduce(37), 77 => TE::Reduce(37), 78 => TE::Reduce(37), 79 => TE::Reduce(37), 80 => TE::Reduce(37), 81 => TE::Reduce(37), 87 => TE::Reduce(37), 93 => TE::Reduce(37), 95 => TE::Reduce(37) },
    hashmap! { 28 => TE::Transit(190), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 82 => TE::Shift(191), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 9 => TE::Transit(48), 11 => TE::Transit(192), 12 => TE::Transit(40), 13 => TE::Transit(41), 14 => TE::Transit(44), 15 => TE::Transit(39), 17 => TE::Transit(45), 18 => TE::Transit(46), 21 => TE::Transit(47), 24 => TE::Transit(42), 25 => TE::Transit(43), 27 => TE::Transit(38), 28 => TE::Transit(51), 29 => TE::Transit(49), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 36 => TE::Transit(37), 37 => TE::Transit(30), 38 => TE::Shift(21), 39 => TE::Reduce(94), 43 => TE::Shift(71), 44 => TE::Shift(72), 45 => TE::Shift(75), 46 => TE::Shift(70), 48 => TE::Shift(76), 49 => TE::Shift(77), 51 => TE::Shift(50), 53 => TE::Shift(73), 54 => TE::Shift(74), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 78 => TE::Shift(33), 80 => TE::Reduce(58), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 38 => TE::Reduce(33), 39 => TE::Reduce(33), 43 => TE::Reduce(33), 44 => TE::Reduce(33), 45 => TE::Reduce(33), 46 => TE::Reduce(33), 47 => TE::Reduce(33), 48 => TE::Reduce(33), 49 => TE::Reduce(33), 51 => TE::Reduce(33), 52 => TE::Reduce(33), 53 => TE::Reduce(33), 54 => TE::Reduce(33), 64 => TE::Reduce(33), 65 => TE::Reduce(33), 66 => TE::Reduce(33), 67 => TE::Reduce(33), 68 => TE::Reduce(33), 69 => TE::Reduce(33), 70 => TE::Reduce(33), 71 => TE::Reduce(33), 72 => TE::Reduce(33), 73 => TE::Reduce(33), 74 => TE::Reduce(33), 75 => TE::Reduce(33), 76 => TE::Reduce(33), 77 => TE::Reduce(33), 78 => TE::Reduce(33), 79 => TE::Reduce(33), 80 => TE::Reduce(33), 81 => TE::Reduce(33), 87 => TE::Reduce(33), 93 => TE::Reduce(33), 95 => TE::Reduce(33) },
    hashmap! { 27 => TE::Transit(194), 28 => TE::Transit(51), 29 => TE::Transit(49), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 51 => TE::Shift(50), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 80 => TE::Reduce(58), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 80 => TE::Shift(195) },
    hashmap! { 28 => TE::Transit(196), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 80 => TE::Shift(197), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 27 => TE::Transit(198), 28 => TE::Transit(51), 29 => TE::Transit(49), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 51 => TE::Shift(50), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 82 => TE::Reduce(58), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 82 => TE::Shift(199) },
    hashmap! { 9 => TE::Transit(48), 11 => TE::Transit(200), 12 => TE::Transit(40), 13 => TE::Transit(41), 14 => TE::Transit(44), 15 => TE::Transit(39), 17 => TE::Transit(45), 18 => TE::Transit(46), 21 => TE::Transit(47), 24 => TE::Transit(42), 25 => TE::Transit(43), 27 => TE::Transit(38), 28 => TE::Transit(51), 29 => TE::Transit(49), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 36 => TE::Transit(37), 37 => TE::Transit(30), 38 => TE::Shift(21), 39 => TE::Reduce(94), 43 => TE::Shift(71), 44 => TE::Shift(72), 45 => TE::Shift(75), 46 => TE::Shift(70), 48 => TE::Shift(76), 49 => TE::Shift(77), 51 => TE::Shift(50), 53 => TE::Shift(73), 54 => TE::Shift(74), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 78 => TE::Shift(33), 80 => TE::Reduce(58), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 38 => TE::Reduce(34), 39 => TE::Reduce(34), 43 => TE::Reduce(34), 44 => TE::Reduce(34), 45 => TE::Reduce(34), 46 => TE::Reduce(34), 47 => TE::Reduce(34), 48 => TE::Reduce(34), 49 => TE::Reduce(34), 51 => TE::Reduce(34), 52 => TE::Reduce(34), 53 => TE::Reduce(34), 54 => TE::Reduce(34), 64 => TE::Reduce(34), 65 => TE::Reduce(34), 66 => TE::Reduce(34), 67 => TE::Reduce(34), 68 => TE::Reduce(34), 69 => TE::Reduce(34), 70 => TE::Reduce(34), 71 => TE::Reduce(34), 72 => TE::Reduce(34), 73 => TE::Reduce(34), 74 => TE::Reduce(34), 75 => TE::Reduce(34), 76 => TE::Reduce(34), 77 => TE::Reduce(34), 78 => TE::Reduce(34), 79 => TE::Reduce(34), 80 => TE::Reduce(34), 81 => TE::Reduce(34), 87 => TE::Reduce(34), 93 => TE::Reduce(34), 95 => TE::Reduce(34) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 80 => TE::Reduce(50), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 26 => TE::Transit(203), 28 => TE::Transit(167), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 82 => TE::Shift(204), 83 => TE::Shift(169) },
    hashmap! { 80 => TE::Reduce(52) },
    hashmap! { 39 => TE::Shift(206) },
    hashmap! { 83 => TE::Shift(207) },
    hashmap! { 28 => TE::Transit(208), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 82 => TE::Shift(209), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 80 => TE::Reduce(39) },
    hashmap! { 19 => TE::Transit(211), 37 => TE::Transit(213), 38 => TE::Shift(21), 51 => TE::Shift(212), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20) },
    hashmap! { 39 => TE::Shift(214) },
    hashmap! { 39 => TE::Reduce(41) },
    hashmap! { 39 => TE::Reduce(42), 93 => TE::Shift(25) },
    hashmap! { 50 => TE::Shift(215) },
    hashmap! { 28 => TE::Transit(216), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 20 => TE::Transit(217), 43 => TE::Shift(218), 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 82 => TE::Reduce(44), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 82 => TE::Shift(219) },
    hashmap! { 28 => TE::Transit(221), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 9 => TE::Transit(48), 11 => TE::Transit(220), 12 => TE::Transit(40), 13 => TE::Transit(41), 14 => TE::Transit(44), 15 => TE::Transit(39), 17 => TE::Transit(45), 18 => TE::Transit(46), 21 => TE::Transit(47), 24 => TE::Transit(42), 25 => TE::Transit(43), 27 => TE::Transit(38), 28 => TE::Transit(51), 29 => TE::Transit(49), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 36 => TE::Transit(37), 37 => TE::Transit(30), 38 => TE::Shift(21), 39 => TE::Reduce(94), 43 => TE::Shift(71), 44 => TE::Shift(72), 45 => TE::Shift(75), 46 => TE::Shift(70), 48 => TE::Shift(76), 49 => TE::Shift(77), 51 => TE::Shift(50), 53 => TE::Shift(73), 54 => TE::Shift(74), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 78 => TE::Shift(33), 80 => TE::Reduce(58), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 38 => TE::Reduce(40), 39 => TE::Reduce(40), 43 => TE::Reduce(40), 44 => TE::Reduce(40), 45 => TE::Reduce(40), 46 => TE::Reduce(40), 47 => TE::Reduce(40), 48 => TE::Reduce(40), 49 => TE::Reduce(40), 51 => TE::Reduce(40), 52 => TE::Reduce(40), 53 => TE::Reduce(40), 54 => TE::Reduce(40), 64 => TE::Reduce(40), 65 => TE::Reduce(40), 66 => TE::Reduce(40), 67 => TE::Reduce(40), 68 => TE::Reduce(40), 69 => TE::Reduce(40), 70 => TE::Reduce(40), 71 => TE::Reduce(40), 72 => TE::Reduce(40), 73 => TE::Reduce(40), 74 => TE::Reduce(40), 75 => TE::Reduce(40), 76 => TE::Reduce(40), 77 => TE::Reduce(40), 78 => TE::Reduce(40), 79 => TE::Reduce(40), 80 => TE::Reduce(40), 81 => TE::Reduce(40), 87 => TE::Reduce(40), 93 => TE::Reduce(40), 95 => TE::Reduce(40) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 82 => TE::Reduce(43), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 79 => TE::Shift(225) },
    hashmap! { 52 => TE::Shift(226), 79 => TE::Reduce(46) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 84 => TE::Shift(230), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 38 => TE::Reduce(45), 39 => TE::Reduce(45), 43 => TE::Reduce(45), 44 => TE::Reduce(45), 45 => TE::Reduce(45), 46 => TE::Reduce(45), 47 => TE::Reduce(45), 48 => TE::Reduce(45), 49 => TE::Reduce(45), 51 => TE::Reduce(45), 52 => TE::Reduce(45), 53 => TE::Reduce(45), 54 => TE::Reduce(45), 64 => TE::Reduce(45), 65 => TE::Reduce(45), 66 => TE::Reduce(45), 67 => TE::Reduce(45), 68 => TE::Reduce(45), 69 => TE::Reduce(45), 70 => TE::Reduce(45), 71 => TE::Reduce(45), 72 => TE::Reduce(45), 73 => TE::Reduce(45), 74 => TE::Reduce(45), 75 => TE::Reduce(45), 76 => TE::Reduce(45), 77 => TE::Reduce(45), 78 => TE::Reduce(45), 79 => TE::Reduce(45), 80 => TE::Reduce(45), 81 => TE::Reduce(45), 87 => TE::Reduce(45), 93 => TE::Reduce(45), 95 => TE::Reduce(45) },
    hashmap! { 28 => TE::Transit(227), 29 => TE::Transit(89), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 39 => TE::Reduce(94), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 55 => TE::Shift(95), 56 => TE::Shift(96), 57 => TE::Shift(99), 58 => TE::Shift(100), 59 => TE::Shift(101), 60 => TE::Shift(102), 61 => TE::Shift(103), 62 => TE::Shift(104), 84 => TE::Shift(228), 86 => TE::Shift(90), 87 => TE::Shift(91), 88 => TE::Shift(92), 89 => TE::Shift(93), 90 => TE::Shift(94), 91 => TE::Shift(97), 92 => TE::Shift(98), 93 => TE::Shift(105), 96 => TE::Shift(106) },
    hashmap! { 9 => TE::Transit(48), 11 => TE::Transit(229), 12 => TE::Transit(40), 13 => TE::Transit(41), 14 => TE::Transit(44), 15 => TE::Transit(39), 17 => TE::Transit(45), 18 => TE::Transit(46), 21 => TE::Transit(47), 24 => TE::Transit(42), 25 => TE::Transit(43), 27 => TE::Transit(38), 28 => TE::Transit(51), 29 => TE::Transit(49), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 36 => TE::Transit(37), 37 => TE::Transit(30), 38 => TE::Shift(21), 39 => TE::Reduce(94), 43 => TE::Shift(71), 44 => TE::Shift(72), 45 => TE::Shift(75), 46 => TE::Shift(70), 48 => TE::Shift(76), 49 => TE::Shift(77), 51 => TE::Shift(50), 53 => TE::Shift(73), 54 => TE::Shift(74), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 78 => TE::Shift(33), 80 => TE::Reduce(58), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 52 => TE::Reduce(48), 79 => TE::Reduce(48) },
    hashmap! { 9 => TE::Transit(48), 11 => TE::Transit(231), 12 => TE::Transit(40), 13 => TE::Transit(41), 14 => TE::Transit(44), 15 => TE::Transit(39), 17 => TE::Transit(45), 18 => TE::Transit(46), 21 => TE::Transit(47), 24 => TE::Transit(42), 25 => TE::Transit(43), 27 => TE::Transit(38), 28 => TE::Transit(51), 29 => TE::Transit(49), 30 => TE::Transit(52), 31 => TE::Transit(53), 32 => TE::Transit(54), 33 => TE::Transit(68), 36 => TE::Transit(37), 37 => TE::Transit(30), 38 => TE::Shift(21), 39 => TE::Reduce(94), 43 => TE::Shift(71), 44 => TE::Shift(72), 45 => TE::Shift(75), 46 => TE::Shift(70), 48 => TE::Shift(76), 49 => TE::Shift(77), 51 => TE::Shift(50), 53 => TE::Shift(73), 54 => TE::Shift(74), 64 => TE::Shift(59), 65 => TE::Shift(60), 66 => TE::Shift(61), 67 => TE::Shift(62), 68 => TE::Shift(63), 69 => TE::Shift(64), 70 => TE::Shift(65), 71 => TE::Shift(66), 72 => TE::Shift(67), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 78 => TE::Shift(33), 80 => TE::Reduce(58), 81 => TE::Shift(56), 87 => TE::Shift(57), 93 => TE::Shift(55), 95 => TE::Shift(58) },
    hashmap! { 52 => TE::Reduce(49), 79 => TE::Reduce(49) },
    hashmap! { 36 => TE::Transit(233), 37 => TE::Transit(30), 38 => TE::Shift(21), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20) },
    hashmap! { 82 => TE::Reduce(16), 83 => TE::Reduce(16) },
    hashmap! { 39 => TE::Shift(235), 93 => TE::Shift(25) },
    hashmap! { 81 => TE::Shift(236) },
    hashmap! { 7 => TE::Transit(237), 8 => TE::Transit(28), 36 => TE::Transit(29), 37 => TE::Transit(30), 38 => TE::Shift(21), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 82 => TE::Reduce(15) },
    hashmap! { 82 => TE::Shift(238) },
    hashmap! { 9 => TE::Transit(239), 78 => TE::Shift(33) },
    hashmap! { 38 => TE::Reduce(12), 42 => TE::Reduce(12), 74 => TE::Reduce(12), 75 => TE::Reduce(12), 76 => TE::Reduce(12), 77 => TE::Reduce(12), 79 => TE::Reduce(12) },
    hashmap! { 78 => TE::Reduce(7) }
];
}

// ------------------------------------
// Module include prologue.
//
// Should include at least result type:
//
// type TResult = <...>;
// 
// You can specify TError = <...>, if not specified, it will be ()
//
// Can also include parsing hooks:
//
//   fn on_parse_begin(parser: &mut Parser, string: &'static str) {
//     ...
//   }
//
//   fn on_parse_end(parser: &mut Parser, string: &'static str) {
//     ...
//   }
//   
//   fn on_parse_error(parser: &Parser, token: &Token) {
//     ...
//   } 



pub mod ast;
extern crate common;
extern crate errors;
extern crate util;

use ast::*;
use common::*;
use errors::*;

impl Parser {
    fn get_loc(&self) -> Location {
        Location(self.tokenizer.token_start_line, self.tokenizer.token_start_column + 1)
    }
}

impl Token {
    fn get_loc(&self) -> Location {
        Location(self.start_line, self.start_column + 1)
    }

    fn get_id(&self) -> String {
        self.value.to_string()
    }
}

fn gen_binary(left: Expr, opt: Token, right: Expr, kind: Operator) -> Expr {
    Expr::Binary(Binary {
        loc: opt.get_loc(),
        opt: kind,
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn gen_unary(opt: Token, opr: Expr, kind: Operator) -> Expr {
    Expr::Unary(Unary {
        loc: opt.get_loc(),
        opt: kind,
        opr: Box::new(opr),
    })
}

fn on_parse_error(parser: &Parser, token: &Token) {
    for error in &parser.errors { eprintln!("{}", error); }
    let loc = token.get_loc();
    eprintln!("*** Error at ({},{}): syntax error", loc.0, loc.1);
    std::process::exit(1);
}

fn on_lex_error(lex: &Tokenizer, slice: &str) {
    for error in lex.get_errors() { eprintln!("{}", error); }
    eprintln!("*** Error at ({},{}): unrecognized character '{}'", lex.current_line, lex.current_column + 1, slice);
    std::process::exit(1);
}

// Final result type returned from `parse` method call.
pub type TResult = Result<Program, Vec<Error>>;
// Error type
pub type TError = Error;

// some util types, only for convenience
type ClassList = Vec<ClassDef>;
type FieldList = Vec<FieldDef>;
type VarDefList = Vec<VarDef>;
type StatementList = Vec<Statement>;
type ExprList = Vec<Expr>;
type ConstList = Vec<Const>;
type GuardedList = Vec<(Expr, Statement)>;
type Flag = bool;

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

lazy_static! {
    // Pre-parse the regex instead of parsing it every time when calling `get_next_token`.
    static ref REGEX_RULES: Vec<Regex> = LEX_RULES.iter().map(|rule| Regex::new(rule).unwrap()).collect();
}

struct Tokenizer {
    // Tokenizing string.
    string: &'static str,

    // Cursor for current symbol.
    cursor: i32,

    // States.
    states: Vec<&'static str>,

    // Line-based location tracking.
    current_line: i32,
    current_column: i32,
    current_line_begin_offset: i32,

    // Location data of a matched token.
    token_start_offset: i32,
    token_end_offset: i32,
    token_start_line: i32,
    token_end_line: i32,
    token_start_column: i32,
    token_end_column: i32,

    // Matched text, and its length.
    yytext: &'static str,
    yyleng: usize,

    string_builder: (String, i32, i32),

    handlers: [fn(&mut Tokenizer) -> &'static str; 89],
}

impl Tokenizer {
    // Creates a new Tokenizer instance.
    // The same instance can be then reused in parser
    // by calling `init_string`.
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

            string_builder: (String::new(), 0, 0),
            
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
    Tokenizer::_lex_rule78,
    Tokenizer::_lex_rule79,
    Tokenizer::_lex_rule80,
    Tokenizer::_lex_rule81,
    Tokenizer::_lex_rule82,
    Tokenizer::_lex_rule83,
    Tokenizer::_lex_rule84,
    Tokenizer::_lex_rule85,
    Tokenizer::_lex_rule86,
    Tokenizer::_lex_rule87,
    Tokenizer::_lex_rule88
],
        };

        tokenizer
    }

    // Initializes a parsing string. 
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

    // Returns next token.
    pub fn get_next_token(&mut self) -> Token {
        if !self.has_more_tokens() {
            self.yytext = EOF;
            return self.to_token(EOF)
        }

        let str_slice = &self.string[self.cursor as usize..];

        let lex_rules_for_state = LEX_RULES_BY_START_CONDITIONS
            .get(self.get_current_state())
            .unwrap();

        let mut max_match_len = -1;
        let mut max_match_token: Option<&'static str> = None;

        for i in lex_rules_for_state {
            let i = *i as usize;
            
            if let Some(matched) = self._match(str_slice, &REGEX_RULES[i]) {

                // Manual handling of EOF token (the end of string). Return it
                // as `EOF` symbol.
                if matched.len() == 0 {
                    self.cursor = self.cursor + 1;
                }

                // find longest match
                if matched.len() as i32 > max_match_len {
                    self.yytext = matched;
                    self.yyleng = matched.len();
                    max_match_len = matched.len() as i32;
                    max_match_token = Some(self.handlers[i](self));
                }
            }
        }

        if let Some(token) = max_match_token {
            self.cursor = self.cursor + (self.yyleng as i32);
            // "" - no token (skip)
            if token.len() == 0 {
                return self.get_next_token();
            }
            return self.to_token(token);
        }

        if self.is_eof() {
            self.cursor = self.cursor + 1;
            self.yytext = EOF;
            return self.to_token(EOF);
        }

        on_lex_error(self, &str_slice[0..1]);

        unreachable!()
    }

    // Throws default "Unexpected token" exception, showing the actual
    // line from the source, pointing with the ^ marker to the bad token.
    // In addition, shows `line:column` location.
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
                Some(matched)
            },
            None => None
        }
    }

    fn to_token(&self, token: &'static str) -> Token {
        Token {
            kind: *TOKENS_MAP.get(token).unwrap(),
            value: self.yytext,
            start_offset: self.token_start_offset,
            end_offset: self.token_end_offset,
            start_line: self.token_start_line,
            end_line: self.token_end_line,
            start_column: self.token_start_column,
            end_column: self.token_end_column,
        }
    }

    // Whether there are still tokens in the stream.
    pub fn has_more_tokens(&self) -> bool {
        self.cursor <= self.string.len() as i32
    }

    // Whether the cursor is at the EOF.
    pub fn is_eof(&self) -> bool {
        self.cursor == self.string.len() as i32
    }

    // Returns current tokenizing state.
    pub fn get_current_state(&self) -> &'static str {
        match self.states.last() {
            Some(last) => last,
            None => "INITIAL"
        }
    }

    // Enters a new state pushing it on the states stack.
    pub fn push_state(&mut self, state: &'static str) -> &mut Tokenizer {
        self.states.push(state);
        self
    }

    // Alias for `push_state`.
    pub fn begin(&mut self, state: &'static str) -> &mut Tokenizer {
        self.push_state(state);
        self
    }

    // Exits a current state popping it from the states stack.
    pub fn pop_state(&mut self) -> &'static str {
        match self.states.pop() {
            Some(top) => top,
            None => "INITIAL"
        }
    }

    fn report_error(&mut self, error: TError) {
        unsafe {
            let ptr = self as *mut Tokenizer;
            (*(ptr.add(1) as *mut Vec<TError>)).push(error);
        }
    }
    
    fn get_errors(&self) -> &Vec<TError> {
        unsafe {
            let ptr = self as *const Tokenizer;
            &(*(ptr.add(1) as *const Vec<TError>))
        }
    }

    // Lex rule handlers.
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
self.begin("S");
                        self.string_builder.0.clear();
                        self.string_builder.1 = self.token_start_line;
                        self.string_builder.2 = self.token_start_column + 1;
                        return "";
}

fn _lex_rule57(&mut self) -> &'static str {
let loc = Location(self.string_builder.1, self.string_builder.2);
                        let string = util::quote(&self.string_builder.0.clone());
                        self.report_error(Error::new(loc, NewlineInStr{ string }));
                        return "";
}

fn _lex_rule58(&mut self) -> &'static str {
return "";
}

fn _lex_rule59(&mut self) -> &'static str {
let loc = Location(self.string_builder.1, self.string_builder.2);
                        let string = util::quote(&self.string_builder.0.clone());
                        self.report_error(Error::new(loc, UnterminatedStr{ string }));
                        self.begin("INITIAL");
                        return "";
}

fn _lex_rule60(&mut self) -> &'static str {
self.begin("INITIAL"); return "STRING_CONST";
}

fn _lex_rule61(&mut self) -> &'static str {
self.string_builder.0.push('\n'); return "";
}

fn _lex_rule62(&mut self) -> &'static str {
self.string_builder.0.push('\t'); return "";
}

fn _lex_rule63(&mut self) -> &'static str {
self.string_builder.0.push('"');  return "";
}

fn _lex_rule64(&mut self) -> &'static str {
self.string_builder.0.push('\\'); return "";
}

fn _lex_rule65(&mut self) -> &'static str {
self.string_builder.0.push_str(self.yytext); return "";
}

fn _lex_rule66(&mut self) -> &'static str {
return "";
}

fn _lex_rule67(&mut self) -> &'static str {
return "";
}

fn _lex_rule68(&mut self) -> &'static str {
return "INT_CONST";
}

fn _lex_rule69(&mut self) -> &'static str {
return "IDENTIFIER";
}

fn _lex_rule70(&mut self) -> &'static str {
return "'{'";
}

fn _lex_rule71(&mut self) -> &'static str {
return "'}'";
}

fn _lex_rule72(&mut self) -> &'static str {
return "';'";
}

fn _lex_rule73(&mut self) -> &'static str {
return "'('";
}

fn _lex_rule74(&mut self) -> &'static str {
return "')'";
}

fn _lex_rule75(&mut self) -> &'static str {
return "','";
}

fn _lex_rule76(&mut self) -> &'static str {
return "':'";
}

fn _lex_rule77(&mut self) -> &'static str {
return "'='";
}

fn _lex_rule78(&mut self) -> &'static str {
return "'+'";
}

fn _lex_rule79(&mut self) -> &'static str {
return "'-'";
}

fn _lex_rule80(&mut self) -> &'static str {
return "'*'";
}

fn _lex_rule81(&mut self) -> &'static str {
return "'/'";
}

fn _lex_rule82(&mut self) -> &'static str {
return "'%'";
}

fn _lex_rule83(&mut self) -> &'static str {
return "'<'";
}

fn _lex_rule84(&mut self) -> &'static str {
return "'>'";
}

fn _lex_rule85(&mut self) -> &'static str {
return "'['";
}

fn _lex_rule86(&mut self) -> &'static str {
return "']'";
}

fn _lex_rule87(&mut self) -> &'static str {
return "'!'";
}

fn _lex_rule88(&mut self) -> &'static str {
return "'.'";
}
}

// ------------------------------------------------------------------
// Parser.

pub struct Parser {
    // Parsing stack: semantic values.
    values_stack: Vec<SV>,

    // Parsing stack: state numbers.
    states_stack: Vec<usize>,

    // Tokenizer instance.
    tokenizer: Tokenizer,
    
    // errors
    errors: Vec<TError>,

    // Semantic action handlers.
    handlers: [fn(&mut Parser) -> SV; 115],
}

impl Parser {
    // Creates a new Parser instance.
    pub fn new() -> Parser {
        Parser {
            // Stacks.
            values_stack: Vec::new(),
            states_stack: Vec::new(),

            tokenizer: Tokenizer::new(),
            errors: Vec::new(),

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
    Parser::_handler114
],
        }
    }

    // Parses a string.
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
        on_parse_error(self, &token);
    }

    fn _handler0(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = self.values_stack.pop().unwrap();

let _0 = _1;
_0
}

fn _handler1(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _1);

let _0 = if self.errors.is_empty() {
            Ok(Program { classes: _1, })
        } else {
            Err(std::mem::replace(&mut self.errors, Vec::new()))
        };
SV::_2(_0)
}

fn _handler2(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _3);
let mut _1 = pop!(self.values_stack, _1);

_1.push(_2);
        let _0 = _1;
SV::_1(_0)
}

fn _handler3(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _3);

let _0 = vec![_1];
SV::_1(_0)
}

fn _handler4(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _6 = pop!(self.values_stack, _6);
self.values_stack.pop();
let mut _4 = pop!(self.values_stack, _5);
let mut _3 = pop!(self.values_stack, _0);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _4);

let _0 = ClassDef {
            loc: _2.get_loc(),
            name: _3.get_id(),
            parent: _4,
            fields: _6,
            sealed: _1,
        };
SV::_3(_0)
}

fn _handler5(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();

let _0 = true;
SV::_4(_0)
}

fn _handler6(&mut self) -> SV {
// Semantic values prologue.


let _0 = false;
SV::_4(_0)
}

fn _handler7(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _0);
self.values_stack.pop();

let _0 = Some(_2.get_id());
SV::_5(_0)
}

fn _handler8(&mut self) -> SV {
// Semantic values prologue.


let _0 = None;
SV::_5(_0)
}

fn _handler9(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _7);
let mut _1 = pop!(self.values_stack, _6);

_1.push(FieldDef::VarDef(_2));
        let _0 = _1;
SV::_6(_0)
}

fn _handler10(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _8);
let mut _1 = pop!(self.values_stack, _6);

_1.push(FieldDef::MethodDef(_2));
        let _0 = _1;
SV::_6(_0)
}

fn _handler11(&mut self) -> SV {
// Semantic values prologue.


let _0 = Vec::new();
SV::_6(_0)
}

fn _handler12(&mut self) -> SV {
// Semantic values prologue.
let mut _7 = pop!(self.values_stack, _11);
self.values_stack.pop();
let mut _5 = pop!(self.values_stack, _10);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _0);
let mut _2 = pop!(self.values_stack, _9);
self.values_stack.pop();

let _0 = MethodDef {
            loc: _3.get_loc(),
            name: _3.get_id(),
            return_type: _2,
            parameters: _5,
            static_: true,
            body: _7,
        };
SV::_8(_0)
}

fn _handler13(&mut self) -> SV {
// Semantic values prologue.
let mut _6 = pop!(self.values_stack, _11);
self.values_stack.pop();
let mut _4 = pop!(self.values_stack, _10);
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _9);

let _0 = MethodDef {
            loc: _2.get_loc(),
            name: _2.get_id(),
            return_type: _1,
            parameters: _4,
            static_: false,
            body: _6,
        };
SV::_8(_0)
}

fn _handler14(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _10);

let _0 = _1;
SV::_10(_0)
}

fn _handler15(&mut self) -> SV {
// Semantic values prologue.


let _0 = Vec::new();
SV::_10(_0)
}

fn _handler16(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _7);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _10);

_1.push(_3);
        let _0 = _1;
SV::_10(_0)
}

fn _handler17(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _7);

let _0 = vec![_1];
SV::_10(_0)
}

fn _handler18(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _12);
let mut _1 = pop!(self.values_stack, _0);

let _0 = Block {
            loc: _1.get_loc(),
            statements: _2,
        };
SV::_11(_0)
}

fn _handler19(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _13);
let mut _1 = pop!(self.values_stack, _12);

_1.push(_2);
        let _0 = _1;
SV::_12(_0)
}

fn _handler20(&mut self) -> SV {
// Semantic values prologue.


let _0 = Vec::new();
SV::_12(_0)
}

fn _handler21(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _7);

let _0 = Statement::VarDef(_1);
SV::_13(_0)
}

fn _handler22(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _14);

let _0 = Statement::Simple(_1);
SV::_13(_0)
}

fn _handler23(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _13);

let _0 = _1;
SV::_13(_0)
}

fn _handler24(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _13);

let _0 = _1;
SV::_13(_0)
}

fn _handler25(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _13);

let _0 = _1;
SV::_13(_0)
}

fn _handler26(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _13);

let _0 = _1;
SV::_13(_0)
}

fn _handler27(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _13);

let _0 = _1;
SV::_13(_0)
}

fn _handler28(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _13);

let _0 = _1;
SV::_13(_0)
}

fn _handler29(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _13);

let _0 = _1;
SV::_13(_0)
}

fn _handler30(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _13);

let _0 = _1;
SV::_13(_0)
}

fn _handler31(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _13);

let _0 = _1;
SV::_13(_0)
}

fn _handler32(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _11);

let _0 = Statement::Block(_1);
SV::_13(_0)
}

fn _handler33(&mut self) -> SV {
// Semantic values prologue.
let mut _5 = pop!(self.values_stack, _13);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _15);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let _0 = Statement::While(While {
            loc: _1.get_loc(),
            cond: _3,
            body: Box::new(_5),
        });
SV::_13(_0)
}

fn _handler34(&mut self) -> SV {
// Semantic values prologue.
let mut _9 = pop!(self.values_stack, _13);
self.values_stack.pop();
let mut _7 = pop!(self.values_stack, _14);
self.values_stack.pop();
let mut _5 = pop!(self.values_stack, _15);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _14);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let _0 = Statement::For(For {
            loc: _1.get_loc(),
            init: _3,
            cond: _5,
            update: _7,
            body: Box::new(_9),
        });
SV::_13(_0)
}

fn _handler35(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let _0 = Statement::Break(Break { loc: _1.get_loc(), });
SV::_13(_0)
}

fn _handler36(&mut self) -> SV {
// Semantic values prologue.
let mut _6 = pop!(self.values_stack, _16);
let mut _5 = pop!(self.values_stack, _13);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _15);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let _0 = Statement::If(If {
            loc: _1.get_loc(),
            cond: _3,
            on_true: Box::new(_5),
            on_false: match _6 {
                Some(statement) => Some(Box::new(statement)),
                None => None,
            },
        });
SV::_13(_0)
}

fn _handler37(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _13);
let mut _1 = pop!(self.values_stack, _0);

let _0 = Some(_2);
SV::_16(_0)
}

fn _handler38(&mut self) -> SV {
// Semantic values prologue.


let _0 = None;
SV::_16(_0)
}

fn _handler39(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _5 = pop!(self.values_stack, _15);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _0);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let _0 = Statement::ObjectCopy(ObjectCopy {
            loc: _1.get_loc(),
            dst: _3.get_id(),
            src: _5,
        });
SV::_13(_0)
}

fn _handler40(&mut self) -> SV {
// Semantic values prologue.
let mut _9 = pop!(self.values_stack, _13);
self.values_stack.pop();
let mut _7 = pop!(self.values_stack, _17);
let mut _6 = pop!(self.values_stack, _15);
self.values_stack.pop();
let mut _4 = pop!(self.values_stack, _0);
let mut _3 = pop!(self.values_stack, _9);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let _0 = Statement::Foreach(Foreach {
            loc: _1.get_loc(),
            type_: _3,
            name: _4.get_id(),
            array: _6,
            cond: _7,
            body: Box::new(_9),
        });
SV::_13(_0)
}

fn _handler41(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();

let _0 = Type::Var;
SV::_9(_0)
}

fn _handler42(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _9);

let _0 = _1;
SV::_9(_0)
}

fn _handler43(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _15);
let mut _1 = pop!(self.values_stack, _0);

let _0 = Some(_2);
SV::_17(_0)
}

fn _handler44(&mut self) -> SV {
// Semantic values prologue.


let _0 = None;
SV::_17(_0)
}

fn _handler45(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _18);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let _0 = Statement::Guarded(Guarded {
            loc: _1.get_loc(),
            guarded: _3,
        });
SV::_13(_0)
}

fn _handler46(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _18);

let _0 = _1;
SV::_18(_0)
}

fn _handler47(&mut self) -> SV {
// Semantic values prologue.


let _0 = Vec::new();
SV::_18(_0)
}

fn _handler48(&mut self) -> SV {
// Semantic values prologue.
let mut _5 = pop!(self.values_stack, _13);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _15);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _18);

_1.push((_3, _5));
        let _0 = _1;
SV::_18(_0)
}

fn _handler49(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _13);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _15);

let _0 = vec![(_1, _3)];
SV::_18(_0)
}

fn _handler50(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _15);
let mut _1 = pop!(self.values_stack, _0);

let _0 = Statement::Return(Return {
            loc: _1.get_loc(),
            expr: Some(_2),
        });
SV::_13(_0)
}

fn _handler51(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let _0 = Statement::Return(Return {
            loc: _1.get_loc(),
            expr: None,
        });
SV::_13(_0)
}

fn _handler52(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _19);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let _0 = Statement::Print(Print {
            loc: _1.get_loc(),
            print: _3,
        });
SV::_13(_0)
}

fn _handler53(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _19);

_1.push(_3);
        let _0 = _1;
SV::_19(_0)
}

fn _handler54(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _15);

let _0 = vec![_1];
SV::_19(_0)
}

fn _handler55(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _20);

let _0 = Simple::Assign(Assign {
            loc: _2.get_loc(),
            dst: _1,
            src: _3,
        });
SV::_14(_0)
}

fn _handler56(&mut self) -> SV {
// Semantic values prologue.
let mut _4 = pop!(self.values_stack, _15);
let mut _3 = pop!(self.values_stack, _0);
let mut _2 = pop!(self.values_stack, _0);
self.values_stack.pop();

let _0 = Simple::VarAssign(VarAssign {
            loc: _3.get_loc(),
            name: _2.get_id(),
            src: _4,
        });
SV::_14(_0)
}

fn _handler57(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _15);

let _0 = Simple::Expr(_1);
SV::_14(_0)
}

fn _handler58(&mut self) -> SV {
// Semantic values prologue.


let _0 = Simple::Skip(Skip { loc: self.get_loc(), });
SV::_14(_0)
}

fn _handler59(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _20);

let _0 = Expr::LValue(_1);
SV::_15(_0)
}

fn _handler60(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _15);

let _0 = _1;
SV::_15(_0)
}

fn _handler61(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _21);

let _0 = Expr::Const(_1);
SV::_15(_0)
}

fn _handler62(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = gen_binary(_1, _2, _3, Operator::Add);
SV::_15(_0)
}

fn _handler63(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = gen_binary(_1, _2, _3, Operator::Sub);
SV::_15(_0)
}

fn _handler64(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = gen_binary(_1, _2, _3, Operator::Mul);
SV::_15(_0)
}

fn _handler65(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = gen_binary(_1, _2, _3, Operator::Div);
SV::_15(_0)
}

fn _handler66(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = gen_binary(_1, _2, _3, Operator::Mod);
SV::_15(_0)
}

fn _handler67(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = gen_binary(_1, _2, _3, Operator::Eq);
SV::_15(_0)
}

fn _handler68(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = gen_binary(_1, _2, _3, Operator::Ne);
SV::_15(_0)
}

fn _handler69(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = gen_binary(_1, _2, _3, Operator::Lt);
SV::_15(_0)
}

fn _handler70(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = gen_binary(_1, _2, _3, Operator::Gt);
SV::_15(_0)
}

fn _handler71(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = gen_binary(_1, _2, _3, Operator::Le);
SV::_15(_0)
}

fn _handler72(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = gen_binary(_1, _2, _3, Operator::Ge);
SV::_15(_0)
}

fn _handler73(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = gen_binary(_1, _2, _3, Operator::And);
SV::_15(_0)
}

fn _handler74(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = gen_binary(_1, _2, _3, Operator::Or);
SV::_15(_0)
}

fn _handler75(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = gen_binary(_1, _2, _3, Operator::Repeat);
SV::_15(_0)
}

fn _handler76(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = gen_binary(_1, _2, _3, Operator::Concat);
SV::_15(_0)
}

fn _handler77(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _5 = pop!(self.values_stack, _15);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = Expr::Range(Range {
            loc: _2.get_loc(),
            array: Box::new(_1),
            lower: Box::new(_3),
            upper: Box::new(_5),
        });
SV::_15(_0)
}

fn _handler78(&mut self) -> SV {
// Semantic values prologue.
let mut _6 = pop!(self.values_stack, _15);
self.values_stack.pop();
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _15);
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _15);

let _0 = Expr::Default(Default {
            loc: _2.get_loc(),
            array: Box::new(_1),
            index: Box::new(_3),
            default: Box::new(_6),
        });
SV::_15(_0)
}

fn _handler79(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _6 = pop!(self.values_stack, _15);
self.values_stack.pop();
let mut _4 = pop!(self.values_stack, _0);
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _15);
let mut _1 = pop!(self.values_stack, _0);

let _0 = Expr::Comprehension(Comprehension {
            loc: _1.get_loc(),
            expr: Box::new(_2),
            name: _4.get_id(),
            array: Box::new(_6),
            cond: None,
        });
SV::_15(_0)
}

fn _handler80(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _8 = pop!(self.values_stack, _15);
self.values_stack.pop();
let mut _6 = pop!(self.values_stack, _15);
self.values_stack.pop();
let mut _4 = pop!(self.values_stack, _0);
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _15);
let mut _1 = pop!(self.values_stack, _0);

let _0 = Expr::Comprehension(Comprehension {
            loc: _1.get_loc(),
            expr: Box::new(_2),
            name: _4.get_id(),
            array: Box::new(_6),
            cond: Some(Box::new(_8)),
        });
SV::_15(_0)
}

fn _handler81(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _15);
self.values_stack.pop();

let _0 = _2;
SV::_15(_0)
}

fn _handler82(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _15);
let mut _1 = pop!(self.values_stack, _0);

let _0 = gen_unary(_1, _2, Operator::Neg);
SV::_15(_0)
}

fn _handler83(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _15);
let mut _1 = pop!(self.values_stack, _0);

let _0 = gen_unary(_1, _2, Operator::Not);
SV::_15(_0)
}

fn _handler84(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let _0 = Expr::ReadInt(ReadInt { loc: _1.get_loc(), });
SV::_15(_0)
}

fn _handler85(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let _0 = Expr::ReadLine(ReadLine { loc: _1.get_loc(), });
SV::_15(_0)
}

fn _handler86(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let _0 = Expr::This(This { loc: _1.get_loc(), });
SV::_15(_0)
}

fn _handler87(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _0);

let _0 = Expr::NewClass(NewClass {
            loc: _1.get_loc(),
            name: _2.get_id(),
        });
SV::_15(_0)
}

fn _handler88(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _4 = pop!(self.values_stack, _15);
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _9);
let mut _1 = pop!(self.values_stack, _0);

let _0 = Expr::NewArray(NewArray {
            loc: _1.get_loc(),
            type_: _2,
            len: Box::new(_4),
        });
SV::_15(_0)
}

fn _handler89(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _5 = pop!(self.values_stack, _0);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _15);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _0);

let _0 = Expr::TypeTest(TypeTest {
            loc: _1.get_loc(),
            expr: Box::new(_3),
            name: _5.get_id(),
        });
SV::_15(_0)
}

fn _handler90(&mut self) -> SV {
// Semantic values prologue.
let mut _5 = pop!(self.values_stack, _15);
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _0);
self.values_stack.pop();
self.values_stack.pop();

let _0 = Expr::TypeCast(TypeCast {
            loc: _3.get_loc(),
            name: _3.get_id(),
            expr: Box::new(_5),
        });
SV::_15(_0)
}

fn _handler91(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _17);

let _0 = LValue::Identifier(Identifier {
            loc: _2.get_loc(),
            owner: match _1 {
                Some(expr) => Some(Box::new(expr)),
                None => None,
            },
            name: _2.get_id(),
        });
SV::_20(_0)
}

fn _handler92(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _3 = pop!(self.values_stack, _15);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _15);

let _0 = LValue::Indexed(Indexed {
            loc: _1.get_loc(),
            array: Box::new(_1),
            index: Box::new(_3),
        });
SV::_20(_0)
}

fn _handler93(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _15);

let _0 = Some(_1);
SV::_17(_0)
}

fn _handler94(&mut self) -> SV {
// Semantic values prologue.


let _0 = None;
SV::_17(_0)
}

fn _handler95(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _4 = pop!(self.values_stack, _19);
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _17);

let _0 = Expr::Call(Call {
            loc: _2.get_loc(),
            receiver: match _1 {
                Some(expr) => Some(Box::new(expr)),
                None => None,
            },
            name: _2.get_id(),
            arguments: _4,
        });
SV::_15(_0)
}

fn _handler96(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let _0 = Const::IntConst(IntConst {
            loc: _1.get_loc(),
            value: _1.value.parse::<i32>().unwrap_or_else(|_| {
                self.errors.push(Error::new(_1.get_loc(), IntTooLarge{ string: _1.get_id(), }));
                0
            }),
        });
SV::_21(_0)
}

fn _handler97(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let _0 = Const::BoolConst(BoolConst {
            loc: _1.get_loc(),
            value: true,
        });
SV::_21(_0)
}

fn _handler98(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let _0 = Const::BoolConst(BoolConst {
            loc: _1.get_loc(),
            value: false,
        });
SV::_21(_0)
}

fn _handler99(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();

let _0 = Const::StringConst(StringConst {
            loc: Location(self.tokenizer.string_builder.1, self.tokenizer.string_builder.2),
            value: self.tokenizer.string_builder.0.clone(),
        });
SV::_21(_0)
}

fn _handler100(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _22);

let _0 = Const::ArrayConst(ArrayConst {
            loc: self.get_loc(),
            value: _1,
        });
SV::_21(_0)
}

fn _handler101(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let _0 = Const::Null(Null { loc: _1.get_loc(), });
SV::_21(_0)
}

fn _handler102(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
let mut _2 = pop!(self.values_stack, _22);
self.values_stack.pop();

let _0 = _2;
SV::_22(_0)
}

fn _handler103(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
self.values_stack.pop();

let _0 = Vec::new();
SV::_22(_0)
}

fn _handler104(&mut self) -> SV {
// Semantic values prologue.
let mut _3 = pop!(self.values_stack, _21);
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _22);

_1.push(_3);
        let _0 = _1;
SV::_22(_0)
}

fn _handler105(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _21);

let _0 = vec![_1];
SV::_22(_0)
}

fn _handler106(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _19);

let _0 = _1;
SV::_19(_0)
}

fn _handler107(&mut self) -> SV {
// Semantic values prologue.


let _0 = Vec::new();
SV::_19(_0)
}

fn _handler108(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _9);

let _0 = VarDef {
            loc: _2.get_loc(),
            name: _2.get_id(),
            type_: _1,
        };
SV::_7(_0)
}

fn _handler109(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let _0 = Type::Basic("int");
SV::_9(_0)
}

fn _handler110(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let _0 = Type::Basic("void");
SV::_9(_0)
}

fn _handler111(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let _0 = Type::Basic("bool");
SV::_9(_0)
}

fn _handler112(&mut self) -> SV {
// Semantic values prologue.
let mut _1 = pop!(self.values_stack, _0);

let _0 = Type::Basic("string");
SV::_9(_0)
}

fn _handler113(&mut self) -> SV {
// Semantic values prologue.
let mut _2 = pop!(self.values_stack, _0);
let mut _1 = pop!(self.values_stack, _0);

let _0 = Type::Class(_2.get_id());
SV::_9(_0)
}

fn _handler114(&mut self) -> SV {
// Semantic values prologue.
self.values_stack.pop();
self.values_stack.pop();
let mut _1 = pop!(self.values_stack, _9);

let _0 = Type::Array(Box::new(_1));
SV::_9(_0)
}
}
