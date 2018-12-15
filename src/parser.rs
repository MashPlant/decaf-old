#![allow(dead_code)]
#![allow(unused_mut)]

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
  _5(Option<Str>),
  _6(FieldList),
  _7(VarDef),
  _8(MethodDef),
  _9(Type),
  _10(VarDefList),
  _11(Block),
  _12(StmtList),
  _13(Stmt),
  _14(Simple),
  _15(Expr),
  _16(Option<Expr>),
  _17(Option<Block>),
  _18(GuardedList),
  _19(ExprList),
}

// Lex rules.
static LEX_RULES: [&'static str; 98] = [
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
  r##########"^--"##########,
  r##########"^<<"##########,
  r##########"^>>"##########,
  r##########"^\+"##########,
  r##########"^-"##########,
  r##########"^\*"##########,
  r##########"^/"##########,
  r##########"^%"##########,
  r##########"^&"##########,
  r##########"^\|"##########,
  r##########"^\^"##########,
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
  r##########"^\["##########,
  r##########"^\]"##########,
  r##########"^\+"##########,
  r##########"^\-"##########,
  r##########"^\*"##########,
  r##########"^/"##########,
  r##########"^%"##########,
  r##########"^<"##########,
  r##########"^>"##########,
  r##########"^&"##########,
  r##########"^\|"##########,
  r##########"^\^"##########,
  r##########"^!"##########,
  r##########"^\."##########
];

// EOF value.
static EOF: &'static str = "$";

// A macro for map literals.
// usage: hashmap!{ 1 => "one", 2 => "two" };
macro_rules! hashmap (
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
static PRODUCTIONS: [[i32; 2]; 119] = [[-1, 1], [0, 1], [1, 2], [1, 1], [2, 7], [3, 1], [3, 0], [4, 2], [4, 0], [5, 3], [5, 2], [5, 0], [6, 7], [6, 6], [7, 1], [7, 0], [8, 3], [8, 1], [9, 3], [10, 2], [10, 0], [11, 2], [11, 1], [11, 1], [11, 1], [11, 2], [11, 2], [11, 2], [11, 2], [11, 1], [11, 1], [11, 1], [12, 1], [13, 5], [14, 9], [15, 9], [16, 1], [17, 6], [18, 2], [18, 0], [19, 6], [20, 1], [20, 1], [21, 2], [21, 0], [22, 4], [23, 1], [23, 0], [24, 5], [24, 3], [25, 2], [25, 1], [26, 4], [27, 3], [27, 1], [28, 3], [28, 4], [28, 4], [28, 1], [28, 1], [28, 0], [29, 1], [29, 5], [29, 1], [29, 1], [29, 1], [29, 1], [29, 3], [29, 1], [29, 3], [29, 3], [29, 3], [29, 3], [29, 3], [29, 3], [29, 3], [29, 3], [29, 3], [29, 3], [29, 3], [29, 3], [29, 3], [29, 3], [29, 3], [29, 3], [29, 3], [29, 3], [29, 3], [29, 6], [29, 6], [29, 7], [29, 9], [29, 3], [29, 2], [29, 2], [29, 2], [29, 2], [29, 2], [29, 2], [29, 3], [29, 3], [29, 1], [29, 4], [29, 5], [29, 6], [29, 5], [30, 2], [30, 4], [31, 2], [31, 0], [32, 1], [32, 0], [33, 2], [34, 1], [34, 1], [34, 1], [34, 1], [34, 2], [34, 3]];

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
    static ref LEX_RULES_BY_START_CONDITIONS: HashMap<&'static str, Vec<i32>> = hashmap! { "INITIAL" => vec! [ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97 ], "S" => vec! [ 63, 64, 65, 66, 67, 68, 69, 70, 71 ] };

    // Maps a string name of a token type to its encoded number (the first
    // token number starts after all numbers for non-terminal).
    static ref TOKENS_MAP: HashMap<&'static str, i32> = hashmap! { "CLASS" => 35, "IDENTIFIER" => 36, "SEALED" => 37, "EXTENDS" => 38, "STATIC" => 39, "WHILE" => 40, "FOR" => 41, "FOREACH" => 42, "IN" => 43, "BREAK" => 44, "IF" => 45, "ELSE" => 46, "SCOPY" => 47, "VAR" => 48, "GUARD_SPLIT" => 49, "RETURN" => 50, "PRINT" => 51, "INT_CONST" => 52, "TRUE" => 53, "FALSE" => 54, "STRING_CONST" => 55, "NULL" => 56, "EQUAL" => 57, "NOT_EQUAL" => 58, "LESS_EQUAL" => 59, "GREATER_EQUAL" => 60, "AND" => 61, "OR" => 62, "REPEAT" => 63, "SHL" => 64, "SHR" => 65, "DEFAULT" => 66, "INC" => 67, "DEC" => 68, "READ_INTEGER" => 69, "READ_LINE" => 70, "THIS" => 71, "NEW" => 72, "INSTANCEOF" => 73, "INT" => 74, "VOID" => 75, "BOOL" => 76, "STRING" => 77, "'{'" => 78, "'}'" => 79, "';'" => 80, "'('" => 81, "')'" => 82, "','" => 83, "':'" => 84, "'='" => 85, "'['" => 86, "']'" => 87, "'+'" => 88, "'-'" => 89, "'*'" => 90, "'/'" => 91, "'%'" => 92, "'<'" => 93, "'>'" => 94, "'&'" => 95, "'|'" => 96, "'^'" => 97, "'!'" => 98, "'.'" => 99, "$" => 100 };

    // Parsing table.
    // Vector index is the state number, value is a map
    // from an encoded symbol to table entry (TE).
    static ref TABLE: Vec<HashMap<i32, TE>>= vec![
    hashmap! { 0 => TE::Transit(1), 1 => TE::Transit(2), 2 => TE::Transit(3), 3 => TE::Transit(4), 35 => TE::Reduce(6), 37 => TE::Shift(5) },
    hashmap! { 100 => TE::Accept },
    hashmap! { 2 => TE::Transit(6), 3 => TE::Transit(4), 35 => TE::Reduce(6), 37 => TE::Shift(5), 100 => TE::Reduce(1) },
    hashmap! { 35 => TE::Reduce(3), 37 => TE::Reduce(3), 100 => TE::Reduce(3) },
    hashmap! { 35 => TE::Shift(7) },
    hashmap! { 35 => TE::Reduce(5) },
    hashmap! { 35 => TE::Reduce(2), 37 => TE::Reduce(2), 100 => TE::Reduce(2) },
    hashmap! { 36 => TE::Shift(8) },
    hashmap! { 4 => TE::Transit(9), 38 => TE::Shift(10), 78 => TE::Reduce(8) },
    hashmap! { 78 => TE::Shift(11) },
    hashmap! { 36 => TE::Shift(249) },
    hashmap! { 5 => TE::Transit(12), 35 => TE::Reduce(11), 39 => TE::Reduce(11), 74 => TE::Reduce(11), 75 => TE::Reduce(11), 76 => TE::Reduce(11), 77 => TE::Reduce(11), 79 => TE::Reduce(11) },
    hashmap! { 6 => TE::Transit(15), 33 => TE::Transit(14), 34 => TE::Transit(16), 35 => TE::Shift(21), 39 => TE::Shift(22), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 79 => TE::Shift(13) },
    hashmap! { 35 => TE::Reduce(4), 37 => TE::Reduce(4), 100 => TE::Reduce(4) },
    hashmap! { 80 => TE::Shift(23) },
    hashmap! { 35 => TE::Reduce(10), 39 => TE::Reduce(10), 74 => TE::Reduce(10), 75 => TE::Reduce(10), 76 => TE::Reduce(10), 77 => TE::Reduce(10), 79 => TE::Reduce(10) },
    hashmap! { 36 => TE::Shift(24), 86 => TE::Shift(25) },
    hashmap! { 36 => TE::Reduce(113), 86 => TE::Reduce(113) },
    hashmap! { 36 => TE::Reduce(114), 86 => TE::Reduce(114) },
    hashmap! { 36 => TE::Reduce(115), 86 => TE::Reduce(115) },
    hashmap! { 36 => TE::Reduce(116), 86 => TE::Reduce(116) },
    hashmap! { 36 => TE::Shift(179) },
    hashmap! { 34 => TE::Transit(243), 35 => TE::Shift(21), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20) },
    hashmap! { 35 => TE::Reduce(9), 39 => TE::Reduce(9), 74 => TE::Reduce(9), 75 => TE::Reduce(9), 76 => TE::Reduce(9), 77 => TE::Reduce(9), 79 => TE::Reduce(9) },
    hashmap! { 80 => TE::Reduce(112), 81 => TE::Shift(26) },
    hashmap! { 87 => TE::Shift(172) },
    hashmap! { 7 => TE::Transit(27), 8 => TE::Transit(28), 33 => TE::Transit(29), 34 => TE::Transit(30), 35 => TE::Shift(21), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 82 => TE::Reduce(15) },
    hashmap! { 82 => TE::Shift(31) },
    hashmap! { 82 => TE::Reduce(14), 83 => TE::Shift(240) },
    hashmap! { 82 => TE::Reduce(17), 83 => TE::Reduce(17) },
    hashmap! { 36 => TE::Shift(242), 86 => TE::Shift(25) },
    hashmap! { 9 => TE::Transit(32), 78 => TE::Shift(33) },
    hashmap! { 35 => TE::Reduce(13), 39 => TE::Reduce(13), 74 => TE::Reduce(13), 75 => TE::Reduce(13), 76 => TE::Reduce(13), 77 => TE::Reduce(13), 79 => TE::Reduce(13) },
    hashmap! { 10 => TE::Transit(34), 35 => TE::Reduce(20), 36 => TE::Reduce(20), 40 => TE::Reduce(20), 41 => TE::Reduce(20), 42 => TE::Reduce(20), 44 => TE::Reduce(20), 45 => TE::Reduce(20), 47 => TE::Reduce(20), 48 => TE::Reduce(20), 50 => TE::Reduce(20), 51 => TE::Reduce(20), 52 => TE::Reduce(20), 53 => TE::Reduce(20), 54 => TE::Reduce(20), 55 => TE::Reduce(20), 56 => TE::Reduce(20), 67 => TE::Reduce(20), 68 => TE::Reduce(20), 69 => TE::Reduce(20), 70 => TE::Reduce(20), 71 => TE::Reduce(20), 72 => TE::Reduce(20), 73 => TE::Reduce(20), 74 => TE::Reduce(20), 75 => TE::Reduce(20), 76 => TE::Reduce(20), 77 => TE::Reduce(20), 78 => TE::Reduce(20), 79 => TE::Reduce(20), 80 => TE::Reduce(20), 81 => TE::Reduce(20), 86 => TE::Reduce(20), 89 => TE::Reduce(20), 98 => TE::Reduce(20) },
    hashmap! { 9 => TE::Transit(47), 11 => TE::Transit(36), 13 => TE::Transit(39), 14 => TE::Transit(40), 15 => TE::Transit(45), 16 => TE::Transit(43), 17 => TE::Transit(38), 19 => TE::Transit(44), 22 => TE::Transit(46), 25 => TE::Transit(41), 26 => TE::Transit(42), 28 => TE::Transit(37), 29 => TE::Transit(52), 30 => TE::Transit(48), 31 => TE::Transit(53), 33 => TE::Transit(51), 34 => TE::Transit(49), 35 => TE::Shift(21), 36 => TE::Reduce(109), 40 => TE::Shift(71), 41 => TE::Shift(72), 42 => TE::Shift(77), 44 => TE::Shift(75), 45 => TE::Shift(70), 47 => TE::Shift(76), 48 => TE::Shift(50), 50 => TE::Shift(73), 51 => TE::Shift(74), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 78 => TE::Shift(33), 79 => TE::Shift(35), 80 => TE::Reduce(60), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 35 => TE::Reduce(18), 36 => TE::Reduce(18), 39 => TE::Reduce(18), 40 => TE::Reduce(18), 41 => TE::Reduce(18), 42 => TE::Reduce(18), 44 => TE::Reduce(18), 45 => TE::Reduce(18), 46 => TE::Reduce(18), 47 => TE::Reduce(18), 48 => TE::Reduce(18), 49 => TE::Reduce(18), 50 => TE::Reduce(18), 51 => TE::Reduce(18), 52 => TE::Reduce(18), 53 => TE::Reduce(18), 54 => TE::Reduce(18), 55 => TE::Reduce(18), 56 => TE::Reduce(18), 67 => TE::Reduce(18), 68 => TE::Reduce(18), 69 => TE::Reduce(18), 70 => TE::Reduce(18), 71 => TE::Reduce(18), 72 => TE::Reduce(18), 73 => TE::Reduce(18), 74 => TE::Reduce(18), 75 => TE::Reduce(18), 76 => TE::Reduce(18), 77 => TE::Reduce(18), 78 => TE::Reduce(18), 79 => TE::Reduce(18), 80 => TE::Reduce(18), 81 => TE::Reduce(18), 86 => TE::Reduce(18), 89 => TE::Reduce(18), 98 => TE::Reduce(18) },
    hashmap! { 35 => TE::Reduce(19), 36 => TE::Reduce(19), 40 => TE::Reduce(19), 41 => TE::Reduce(19), 42 => TE::Reduce(19), 44 => TE::Reduce(19), 45 => TE::Reduce(19), 47 => TE::Reduce(19), 48 => TE::Reduce(19), 50 => TE::Reduce(19), 51 => TE::Reduce(19), 52 => TE::Reduce(19), 53 => TE::Reduce(19), 54 => TE::Reduce(19), 55 => TE::Reduce(19), 56 => TE::Reduce(19), 67 => TE::Reduce(19), 68 => TE::Reduce(19), 69 => TE::Reduce(19), 70 => TE::Reduce(19), 71 => TE::Reduce(19), 72 => TE::Reduce(19), 73 => TE::Reduce(19), 74 => TE::Reduce(19), 75 => TE::Reduce(19), 76 => TE::Reduce(19), 77 => TE::Reduce(19), 78 => TE::Reduce(19), 79 => TE::Reduce(19), 80 => TE::Reduce(19), 81 => TE::Reduce(19), 86 => TE::Reduce(19), 89 => TE::Reduce(19), 98 => TE::Reduce(19) },
    hashmap! { 80 => TE::Shift(78) },
    hashmap! { 35 => TE::Reduce(22), 36 => TE::Reduce(22), 40 => TE::Reduce(22), 41 => TE::Reduce(22), 42 => TE::Reduce(22), 44 => TE::Reduce(22), 45 => TE::Reduce(22), 46 => TE::Reduce(22), 47 => TE::Reduce(22), 48 => TE::Reduce(22), 49 => TE::Reduce(22), 50 => TE::Reduce(22), 51 => TE::Reduce(22), 52 => TE::Reduce(22), 53 => TE::Reduce(22), 54 => TE::Reduce(22), 55 => TE::Reduce(22), 56 => TE::Reduce(22), 67 => TE::Reduce(22), 68 => TE::Reduce(22), 69 => TE::Reduce(22), 70 => TE::Reduce(22), 71 => TE::Reduce(22), 72 => TE::Reduce(22), 73 => TE::Reduce(22), 74 => TE::Reduce(22), 75 => TE::Reduce(22), 76 => TE::Reduce(22), 77 => TE::Reduce(22), 78 => TE::Reduce(22), 79 => TE::Reduce(22), 80 => TE::Reduce(22), 81 => TE::Reduce(22), 86 => TE::Reduce(22), 89 => TE::Reduce(22), 98 => TE::Reduce(22) },
    hashmap! { 35 => TE::Reduce(23), 36 => TE::Reduce(23), 40 => TE::Reduce(23), 41 => TE::Reduce(23), 42 => TE::Reduce(23), 44 => TE::Reduce(23), 45 => TE::Reduce(23), 46 => TE::Reduce(23), 47 => TE::Reduce(23), 48 => TE::Reduce(23), 49 => TE::Reduce(23), 50 => TE::Reduce(23), 51 => TE::Reduce(23), 52 => TE::Reduce(23), 53 => TE::Reduce(23), 54 => TE::Reduce(23), 55 => TE::Reduce(23), 56 => TE::Reduce(23), 67 => TE::Reduce(23), 68 => TE::Reduce(23), 69 => TE::Reduce(23), 70 => TE::Reduce(23), 71 => TE::Reduce(23), 72 => TE::Reduce(23), 73 => TE::Reduce(23), 74 => TE::Reduce(23), 75 => TE::Reduce(23), 76 => TE::Reduce(23), 77 => TE::Reduce(23), 78 => TE::Reduce(23), 79 => TE::Reduce(23), 80 => TE::Reduce(23), 81 => TE::Reduce(23), 86 => TE::Reduce(23), 89 => TE::Reduce(23), 98 => TE::Reduce(23) },
    hashmap! { 35 => TE::Reduce(24), 36 => TE::Reduce(24), 40 => TE::Reduce(24), 41 => TE::Reduce(24), 42 => TE::Reduce(24), 44 => TE::Reduce(24), 45 => TE::Reduce(24), 46 => TE::Reduce(24), 47 => TE::Reduce(24), 48 => TE::Reduce(24), 49 => TE::Reduce(24), 50 => TE::Reduce(24), 51 => TE::Reduce(24), 52 => TE::Reduce(24), 53 => TE::Reduce(24), 54 => TE::Reduce(24), 55 => TE::Reduce(24), 56 => TE::Reduce(24), 67 => TE::Reduce(24), 68 => TE::Reduce(24), 69 => TE::Reduce(24), 70 => TE::Reduce(24), 71 => TE::Reduce(24), 72 => TE::Reduce(24), 73 => TE::Reduce(24), 74 => TE::Reduce(24), 75 => TE::Reduce(24), 76 => TE::Reduce(24), 77 => TE::Reduce(24), 78 => TE::Reduce(24), 79 => TE::Reduce(24), 80 => TE::Reduce(24), 81 => TE::Reduce(24), 86 => TE::Reduce(24), 89 => TE::Reduce(24), 98 => TE::Reduce(24) },
    hashmap! { 80 => TE::Shift(79) },
    hashmap! { 80 => TE::Shift(80) },
    hashmap! { 80 => TE::Shift(81) },
    hashmap! { 80 => TE::Shift(82) },
    hashmap! { 35 => TE::Reduce(29), 36 => TE::Reduce(29), 40 => TE::Reduce(29), 41 => TE::Reduce(29), 42 => TE::Reduce(29), 44 => TE::Reduce(29), 45 => TE::Reduce(29), 46 => TE::Reduce(29), 47 => TE::Reduce(29), 48 => TE::Reduce(29), 49 => TE::Reduce(29), 50 => TE::Reduce(29), 51 => TE::Reduce(29), 52 => TE::Reduce(29), 53 => TE::Reduce(29), 54 => TE::Reduce(29), 55 => TE::Reduce(29), 56 => TE::Reduce(29), 67 => TE::Reduce(29), 68 => TE::Reduce(29), 69 => TE::Reduce(29), 70 => TE::Reduce(29), 71 => TE::Reduce(29), 72 => TE::Reduce(29), 73 => TE::Reduce(29), 74 => TE::Reduce(29), 75 => TE::Reduce(29), 76 => TE::Reduce(29), 77 => TE::Reduce(29), 78 => TE::Reduce(29), 79 => TE::Reduce(29), 80 => TE::Reduce(29), 81 => TE::Reduce(29), 86 => TE::Reduce(29), 89 => TE::Reduce(29), 98 => TE::Reduce(29) },
    hashmap! { 35 => TE::Reduce(30), 36 => TE::Reduce(30), 40 => TE::Reduce(30), 41 => TE::Reduce(30), 42 => TE::Reduce(30), 44 => TE::Reduce(30), 45 => TE::Reduce(30), 46 => TE::Reduce(30), 47 => TE::Reduce(30), 48 => TE::Reduce(30), 49 => TE::Reduce(30), 50 => TE::Reduce(30), 51 => TE::Reduce(30), 52 => TE::Reduce(30), 53 => TE::Reduce(30), 54 => TE::Reduce(30), 55 => TE::Reduce(30), 56 => TE::Reduce(30), 67 => TE::Reduce(30), 68 => TE::Reduce(30), 69 => TE::Reduce(30), 70 => TE::Reduce(30), 71 => TE::Reduce(30), 72 => TE::Reduce(30), 73 => TE::Reduce(30), 74 => TE::Reduce(30), 75 => TE::Reduce(30), 76 => TE::Reduce(30), 77 => TE::Reduce(30), 78 => TE::Reduce(30), 79 => TE::Reduce(30), 80 => TE::Reduce(30), 81 => TE::Reduce(30), 86 => TE::Reduce(30), 89 => TE::Reduce(30), 98 => TE::Reduce(30) },
    hashmap! { 35 => TE::Reduce(31), 36 => TE::Reduce(31), 40 => TE::Reduce(31), 41 => TE::Reduce(31), 42 => TE::Reduce(31), 44 => TE::Reduce(31), 45 => TE::Reduce(31), 46 => TE::Reduce(31), 47 => TE::Reduce(31), 48 => TE::Reduce(31), 49 => TE::Reduce(31), 50 => TE::Reduce(31), 51 => TE::Reduce(31), 52 => TE::Reduce(31), 53 => TE::Reduce(31), 54 => TE::Reduce(31), 55 => TE::Reduce(31), 56 => TE::Reduce(31), 67 => TE::Reduce(31), 68 => TE::Reduce(31), 69 => TE::Reduce(31), 70 => TE::Reduce(31), 71 => TE::Reduce(31), 72 => TE::Reduce(31), 73 => TE::Reduce(31), 74 => TE::Reduce(31), 75 => TE::Reduce(31), 76 => TE::Reduce(31), 77 => TE::Reduce(31), 78 => TE::Reduce(31), 79 => TE::Reduce(31), 80 => TE::Reduce(31), 81 => TE::Reduce(31), 86 => TE::Reduce(31), 89 => TE::Reduce(31), 98 => TE::Reduce(31) },
    hashmap! { 57 => TE::Reduce(61), 58 => TE::Reduce(61), 59 => TE::Reduce(61), 60 => TE::Reduce(61), 61 => TE::Reduce(61), 62 => TE::Reduce(61), 63 => TE::Reduce(61), 64 => TE::Reduce(61), 65 => TE::Reduce(61), 67 => TE::Reduce(61), 68 => TE::Reduce(61), 80 => TE::Reduce(61), 82 => TE::Reduce(61), 85 => TE::Shift(83), 86 => TE::Reduce(61), 88 => TE::Reduce(61), 89 => TE::Reduce(61), 90 => TE::Reduce(61), 91 => TE::Reduce(61), 92 => TE::Reduce(61), 93 => TE::Reduce(61), 94 => TE::Reduce(61), 95 => TE::Reduce(61), 96 => TE::Reduce(61), 97 => TE::Reduce(61), 99 => TE::Reduce(61) },
    hashmap! { 36 => TE::Shift(182), 86 => TE::Shift(25) },
    hashmap! { 36 => TE::Shift(185) },
    hashmap! { 80 => TE::Reduce(58), 82 => TE::Reduce(58) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(59), 82 => TE::Reduce(59), 86 => TE::Shift(105), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 36 => TE::Shift(133) },
    hashmap! { 40 => TE::Reduce(63), 41 => TE::Reduce(63), 45 => TE::Reduce(63), 57 => TE::Reduce(63), 58 => TE::Reduce(63), 59 => TE::Reduce(63), 60 => TE::Reduce(63), 61 => TE::Reduce(63), 62 => TE::Reduce(63), 63 => TE::Reduce(63), 64 => TE::Reduce(63), 65 => TE::Reduce(63), 67 => TE::Reduce(63), 68 => TE::Reduce(63), 80 => TE::Reduce(63), 82 => TE::Reduce(63), 83 => TE::Reduce(63), 84 => TE::Reduce(63), 86 => TE::Reduce(63), 87 => TE::Reduce(63), 88 => TE::Reduce(63), 89 => TE::Reduce(63), 90 => TE::Reduce(63), 91 => TE::Reduce(63), 92 => TE::Reduce(63), 93 => TE::Reduce(63), 94 => TE::Reduce(63), 95 => TE::Reduce(63), 96 => TE::Reduce(63), 97 => TE::Reduce(63), 99 => TE::Reduce(63) },
    hashmap! { 40 => TE::Reduce(64), 41 => TE::Reduce(64), 45 => TE::Reduce(64), 57 => TE::Reduce(64), 58 => TE::Reduce(64), 59 => TE::Reduce(64), 60 => TE::Reduce(64), 61 => TE::Reduce(64), 62 => TE::Reduce(64), 63 => TE::Reduce(64), 64 => TE::Reduce(64), 65 => TE::Reduce(64), 67 => TE::Reduce(64), 68 => TE::Reduce(64), 80 => TE::Reduce(64), 82 => TE::Reduce(64), 83 => TE::Reduce(64), 84 => TE::Reduce(64), 86 => TE::Reduce(64), 87 => TE::Reduce(64), 88 => TE::Reduce(64), 89 => TE::Reduce(64), 90 => TE::Reduce(64), 91 => TE::Reduce(64), 92 => TE::Reduce(64), 93 => TE::Reduce(64), 94 => TE::Reduce(64), 95 => TE::Reduce(64), 96 => TE::Reduce(64), 97 => TE::Reduce(64), 99 => TE::Reduce(64) },
    hashmap! { 40 => TE::Reduce(65), 41 => TE::Reduce(65), 45 => TE::Reduce(65), 57 => TE::Reduce(65), 58 => TE::Reduce(65), 59 => TE::Reduce(65), 60 => TE::Reduce(65), 61 => TE::Reduce(65), 62 => TE::Reduce(65), 63 => TE::Reduce(65), 64 => TE::Reduce(65), 65 => TE::Reduce(65), 67 => TE::Reduce(65), 68 => TE::Reduce(65), 80 => TE::Reduce(65), 82 => TE::Reduce(65), 83 => TE::Reduce(65), 84 => TE::Reduce(65), 86 => TE::Reduce(65), 87 => TE::Reduce(65), 88 => TE::Reduce(65), 89 => TE::Reduce(65), 90 => TE::Reduce(65), 91 => TE::Reduce(65), 92 => TE::Reduce(65), 93 => TE::Reduce(65), 94 => TE::Reduce(65), 95 => TE::Reduce(65), 96 => TE::Reduce(65), 97 => TE::Reduce(65), 99 => TE::Reduce(65) },
    hashmap! { 40 => TE::Reduce(66), 41 => TE::Reduce(66), 45 => TE::Reduce(66), 57 => TE::Reduce(66), 58 => TE::Reduce(66), 59 => TE::Reduce(66), 60 => TE::Reduce(66), 61 => TE::Reduce(66), 62 => TE::Reduce(66), 63 => TE::Reduce(66), 64 => TE::Reduce(66), 65 => TE::Reduce(66), 67 => TE::Reduce(66), 68 => TE::Reduce(66), 80 => TE::Reduce(66), 82 => TE::Reduce(66), 83 => TE::Reduce(66), 84 => TE::Reduce(66), 86 => TE::Reduce(66), 87 => TE::Reduce(66), 88 => TE::Reduce(66), 89 => TE::Reduce(66), 90 => TE::Reduce(66), 91 => TE::Reduce(66), 92 => TE::Reduce(66), 93 => TE::Reduce(66), 94 => TE::Reduce(66), 95 => TE::Reduce(66), 96 => TE::Reduce(66), 97 => TE::Reduce(66), 99 => TE::Reduce(66) },
    hashmap! { 27 => TE::Transit(141), 29 => TE::Transit(142), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 40 => TE::Reduce(68), 41 => TE::Reduce(68), 45 => TE::Reduce(68), 57 => TE::Reduce(68), 58 => TE::Reduce(68), 59 => TE::Reduce(68), 60 => TE::Reduce(68), 61 => TE::Reduce(68), 62 => TE::Reduce(68), 63 => TE::Reduce(68), 64 => TE::Reduce(68), 65 => TE::Reduce(68), 67 => TE::Reduce(68), 68 => TE::Reduce(68), 80 => TE::Reduce(68), 82 => TE::Reduce(68), 83 => TE::Reduce(68), 84 => TE::Reduce(68), 86 => TE::Reduce(68), 87 => TE::Reduce(68), 88 => TE::Reduce(68), 89 => TE::Reduce(68), 90 => TE::Reduce(68), 91 => TE::Reduce(68), 92 => TE::Reduce(68), 93 => TE::Reduce(68), 94 => TE::Reduce(68), 95 => TE::Reduce(68), 96 => TE::Reduce(68), 97 => TE::Reduce(68), 99 => TE::Reduce(68) },
    hashmap! { 29 => TE::Transit(152), 30 => TE::Transit(85), 31 => TE::Transit(53), 35 => TE::Shift(153), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(158), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(159), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(160), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(161), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 81 => TE::Shift(162) },
    hashmap! { 81 => TE::Shift(164) },
    hashmap! { 40 => TE::Reduce(101), 41 => TE::Reduce(101), 45 => TE::Reduce(101), 57 => TE::Reduce(101), 58 => TE::Reduce(101), 59 => TE::Reduce(101), 60 => TE::Reduce(101), 61 => TE::Reduce(101), 62 => TE::Reduce(101), 63 => TE::Reduce(101), 64 => TE::Reduce(101), 65 => TE::Reduce(101), 67 => TE::Reduce(101), 68 => TE::Reduce(101), 80 => TE::Reduce(101), 82 => TE::Reduce(101), 83 => TE::Reduce(101), 84 => TE::Reduce(101), 86 => TE::Reduce(101), 87 => TE::Reduce(101), 88 => TE::Reduce(101), 89 => TE::Reduce(101), 90 => TE::Reduce(101), 91 => TE::Reduce(101), 92 => TE::Reduce(101), 93 => TE::Reduce(101), 94 => TE::Reduce(101), 95 => TE::Reduce(101), 96 => TE::Reduce(101), 97 => TE::Reduce(101), 99 => TE::Reduce(101) },
    hashmap! { 34 => TE::Transit(167), 35 => TE::Shift(21), 36 => TE::Shift(166), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20) },
    hashmap! { 81 => TE::Shift(174) },
    hashmap! { 78 => TE::Shift(189), 81 => TE::Shift(188) },
    hashmap! { 81 => TE::Shift(197) },
    hashmap! { 81 => TE::Shift(201) },
    hashmap! { 29 => TE::Transit(209), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 80 => TE::Reduce(51), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 81 => TE::Shift(210) },
    hashmap! { 80 => TE::Reduce(36) },
    hashmap! { 81 => TE::Shift(213) },
    hashmap! { 81 => TE::Shift(218) },
    hashmap! { 35 => TE::Reduce(21), 36 => TE::Reduce(21), 40 => TE::Reduce(21), 41 => TE::Reduce(21), 42 => TE::Reduce(21), 44 => TE::Reduce(21), 45 => TE::Reduce(21), 46 => TE::Reduce(21), 47 => TE::Reduce(21), 48 => TE::Reduce(21), 49 => TE::Reduce(21), 50 => TE::Reduce(21), 51 => TE::Reduce(21), 52 => TE::Reduce(21), 53 => TE::Reduce(21), 54 => TE::Reduce(21), 55 => TE::Reduce(21), 56 => TE::Reduce(21), 67 => TE::Reduce(21), 68 => TE::Reduce(21), 69 => TE::Reduce(21), 70 => TE::Reduce(21), 71 => TE::Reduce(21), 72 => TE::Reduce(21), 73 => TE::Reduce(21), 74 => TE::Reduce(21), 75 => TE::Reduce(21), 76 => TE::Reduce(21), 77 => TE::Reduce(21), 78 => TE::Reduce(21), 79 => TE::Reduce(21), 80 => TE::Reduce(21), 81 => TE::Reduce(21), 86 => TE::Reduce(21), 89 => TE::Reduce(21), 98 => TE::Reduce(21) },
    hashmap! { 35 => TE::Reduce(25), 36 => TE::Reduce(25), 40 => TE::Reduce(25), 41 => TE::Reduce(25), 42 => TE::Reduce(25), 44 => TE::Reduce(25), 45 => TE::Reduce(25), 46 => TE::Reduce(25), 47 => TE::Reduce(25), 48 => TE::Reduce(25), 49 => TE::Reduce(25), 50 => TE::Reduce(25), 51 => TE::Reduce(25), 52 => TE::Reduce(25), 53 => TE::Reduce(25), 54 => TE::Reduce(25), 55 => TE::Reduce(25), 56 => TE::Reduce(25), 67 => TE::Reduce(25), 68 => TE::Reduce(25), 69 => TE::Reduce(25), 70 => TE::Reduce(25), 71 => TE::Reduce(25), 72 => TE::Reduce(25), 73 => TE::Reduce(25), 74 => TE::Reduce(25), 75 => TE::Reduce(25), 76 => TE::Reduce(25), 77 => TE::Reduce(25), 78 => TE::Reduce(25), 79 => TE::Reduce(25), 80 => TE::Reduce(25), 81 => TE::Reduce(25), 86 => TE::Reduce(25), 89 => TE::Reduce(25), 98 => TE::Reduce(25) },
    hashmap! { 35 => TE::Reduce(26), 36 => TE::Reduce(26), 40 => TE::Reduce(26), 41 => TE::Reduce(26), 42 => TE::Reduce(26), 44 => TE::Reduce(26), 45 => TE::Reduce(26), 46 => TE::Reduce(26), 47 => TE::Reduce(26), 48 => TE::Reduce(26), 49 => TE::Reduce(26), 50 => TE::Reduce(26), 51 => TE::Reduce(26), 52 => TE::Reduce(26), 53 => TE::Reduce(26), 54 => TE::Reduce(26), 55 => TE::Reduce(26), 56 => TE::Reduce(26), 67 => TE::Reduce(26), 68 => TE::Reduce(26), 69 => TE::Reduce(26), 70 => TE::Reduce(26), 71 => TE::Reduce(26), 72 => TE::Reduce(26), 73 => TE::Reduce(26), 74 => TE::Reduce(26), 75 => TE::Reduce(26), 76 => TE::Reduce(26), 77 => TE::Reduce(26), 78 => TE::Reduce(26), 79 => TE::Reduce(26), 80 => TE::Reduce(26), 81 => TE::Reduce(26), 86 => TE::Reduce(26), 89 => TE::Reduce(26), 98 => TE::Reduce(26) },
    hashmap! { 35 => TE::Reduce(27), 36 => TE::Reduce(27), 40 => TE::Reduce(27), 41 => TE::Reduce(27), 42 => TE::Reduce(27), 44 => TE::Reduce(27), 45 => TE::Reduce(27), 46 => TE::Reduce(27), 47 => TE::Reduce(27), 48 => TE::Reduce(27), 49 => TE::Reduce(27), 50 => TE::Reduce(27), 51 => TE::Reduce(27), 52 => TE::Reduce(27), 53 => TE::Reduce(27), 54 => TE::Reduce(27), 55 => TE::Reduce(27), 56 => TE::Reduce(27), 67 => TE::Reduce(27), 68 => TE::Reduce(27), 69 => TE::Reduce(27), 70 => TE::Reduce(27), 71 => TE::Reduce(27), 72 => TE::Reduce(27), 73 => TE::Reduce(27), 74 => TE::Reduce(27), 75 => TE::Reduce(27), 76 => TE::Reduce(27), 77 => TE::Reduce(27), 78 => TE::Reduce(27), 79 => TE::Reduce(27), 80 => TE::Reduce(27), 81 => TE::Reduce(27), 86 => TE::Reduce(27), 89 => TE::Reduce(27), 98 => TE::Reduce(27) },
    hashmap! { 35 => TE::Reduce(28), 36 => TE::Reduce(28), 40 => TE::Reduce(28), 41 => TE::Reduce(28), 42 => TE::Reduce(28), 44 => TE::Reduce(28), 45 => TE::Reduce(28), 46 => TE::Reduce(28), 47 => TE::Reduce(28), 48 => TE::Reduce(28), 49 => TE::Reduce(28), 50 => TE::Reduce(28), 51 => TE::Reduce(28), 52 => TE::Reduce(28), 53 => TE::Reduce(28), 54 => TE::Reduce(28), 55 => TE::Reduce(28), 56 => TE::Reduce(28), 67 => TE::Reduce(28), 68 => TE::Reduce(28), 69 => TE::Reduce(28), 70 => TE::Reduce(28), 71 => TE::Reduce(28), 72 => TE::Reduce(28), 73 => TE::Reduce(28), 74 => TE::Reduce(28), 75 => TE::Reduce(28), 76 => TE::Reduce(28), 77 => TE::Reduce(28), 78 => TE::Reduce(28), 79 => TE::Reduce(28), 80 => TE::Reduce(28), 81 => TE::Reduce(28), 86 => TE::Reduce(28), 89 => TE::Reduce(28), 98 => TE::Reduce(28) },
    hashmap! { 29 => TE::Transit(84), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(55), 82 => TE::Reduce(55), 86 => TE::Shift(105), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(61), 41 => TE::Reduce(61), 45 => TE::Reduce(61), 57 => TE::Reduce(61), 58 => TE::Reduce(61), 59 => TE::Reduce(61), 60 => TE::Reduce(61), 61 => TE::Reduce(61), 62 => TE::Reduce(61), 63 => TE::Reduce(61), 64 => TE::Reduce(61), 65 => TE::Reduce(61), 67 => TE::Reduce(61), 68 => TE::Reduce(61), 80 => TE::Reduce(61), 82 => TE::Reduce(61), 83 => TE::Reduce(61), 84 => TE::Reduce(61), 86 => TE::Reduce(61), 87 => TE::Reduce(61), 88 => TE::Reduce(61), 89 => TE::Reduce(61), 90 => TE::Reduce(61), 91 => TE::Reduce(61), 92 => TE::Reduce(61), 93 => TE::Reduce(61), 94 => TE::Reduce(61), 95 => TE::Reduce(61), 96 => TE::Reduce(61), 97 => TE::Reduce(61), 99 => TE::Reduce(61) },
    hashmap! { 29 => TE::Transit(109), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(110), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(111), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(112), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(113), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(114), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(115), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(116), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(117), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(118), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(119), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(120), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(121), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(122), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(123), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(124), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(125), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(126), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(127), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 29 => TE::Transit(128), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 40 => TE::Reduce(97), 41 => TE::Reduce(97), 45 => TE::Reduce(97), 57 => TE::Reduce(97), 58 => TE::Reduce(97), 59 => TE::Reduce(97), 60 => TE::Reduce(97), 61 => TE::Reduce(97), 62 => TE::Reduce(97), 63 => TE::Reduce(97), 64 => TE::Reduce(97), 65 => TE::Reduce(97), 67 => TE::Reduce(97), 68 => TE::Reduce(97), 80 => TE::Reduce(97), 82 => TE::Reduce(97), 83 => TE::Reduce(97), 84 => TE::Reduce(97), 86 => TE::Reduce(97), 87 => TE::Reduce(97), 88 => TE::Reduce(97), 89 => TE::Reduce(97), 90 => TE::Reduce(97), 91 => TE::Reduce(97), 92 => TE::Reduce(97), 93 => TE::Reduce(97), 94 => TE::Reduce(97), 95 => TE::Reduce(97), 96 => TE::Reduce(97), 97 => TE::Reduce(97), 99 => TE::Reduce(97) },
    hashmap! { 40 => TE::Reduce(98), 41 => TE::Reduce(98), 45 => TE::Reduce(98), 57 => TE::Reduce(98), 58 => TE::Reduce(98), 59 => TE::Reduce(98), 60 => TE::Reduce(98), 61 => TE::Reduce(98), 62 => TE::Reduce(98), 63 => TE::Reduce(98), 64 => TE::Reduce(98), 65 => TE::Reduce(98), 67 => TE::Reduce(98), 68 => TE::Reduce(98), 80 => TE::Reduce(98), 82 => TE::Reduce(98), 83 => TE::Reduce(98), 84 => TE::Reduce(98), 86 => TE::Reduce(98), 87 => TE::Reduce(98), 88 => TE::Reduce(98), 89 => TE::Reduce(98), 90 => TE::Reduce(98), 91 => TE::Reduce(98), 92 => TE::Reduce(98), 93 => TE::Reduce(98), 94 => TE::Reduce(98), 95 => TE::Reduce(98), 96 => TE::Reduce(98), 97 => TE::Reduce(98), 99 => TE::Reduce(98) },
    hashmap! { 36 => TE::Reduce(108) },
    hashmap! { 40 => TE::Reduce(69), 41 => TE::Reduce(69), 45 => TE::Reduce(69), 57 => TE::Reduce(69), 58 => TE::Reduce(69), 59 => TE::Reduce(69), 60 => TE::Reduce(69), 61 => TE::Reduce(69), 62 => TE::Reduce(69), 63 => TE::Reduce(69), 64 => TE::Reduce(69), 65 => TE::Reduce(69), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(69), 82 => TE::Reduce(69), 83 => TE::Reduce(69), 84 => TE::Reduce(69), 86 => TE::Shift(105), 87 => TE::Reduce(69), 88 => TE::Reduce(69), 89 => TE::Reduce(69), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Reduce(69), 94 => TE::Reduce(69), 95 => TE::Reduce(69), 96 => TE::Reduce(69), 97 => TE::Reduce(69), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(70), 41 => TE::Reduce(70), 45 => TE::Reduce(70), 57 => TE::Reduce(70), 58 => TE::Reduce(70), 59 => TE::Reduce(70), 60 => TE::Reduce(70), 61 => TE::Reduce(70), 62 => TE::Reduce(70), 63 => TE::Reduce(70), 64 => TE::Reduce(70), 65 => TE::Reduce(70), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(70), 82 => TE::Reduce(70), 83 => TE::Reduce(70), 84 => TE::Reduce(70), 86 => TE::Shift(105), 87 => TE::Reduce(70), 88 => TE::Reduce(70), 89 => TE::Reduce(70), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Reduce(70), 94 => TE::Reduce(70), 95 => TE::Reduce(70), 96 => TE::Reduce(70), 97 => TE::Reduce(70), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(71), 41 => TE::Reduce(71), 45 => TE::Reduce(71), 57 => TE::Reduce(71), 58 => TE::Reduce(71), 59 => TE::Reduce(71), 60 => TE::Reduce(71), 61 => TE::Reduce(71), 62 => TE::Reduce(71), 63 => TE::Reduce(71), 64 => TE::Reduce(71), 65 => TE::Reduce(71), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(71), 82 => TE::Reduce(71), 83 => TE::Reduce(71), 84 => TE::Reduce(71), 86 => TE::Shift(105), 87 => TE::Reduce(71), 88 => TE::Reduce(71), 89 => TE::Reduce(71), 90 => TE::Reduce(71), 91 => TE::Reduce(71), 92 => TE::Reduce(71), 93 => TE::Reduce(71), 94 => TE::Reduce(71), 95 => TE::Reduce(71), 96 => TE::Reduce(71), 97 => TE::Reduce(71), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(72), 41 => TE::Reduce(72), 45 => TE::Reduce(72), 57 => TE::Reduce(72), 58 => TE::Reduce(72), 59 => TE::Reduce(72), 60 => TE::Reduce(72), 61 => TE::Reduce(72), 62 => TE::Reduce(72), 63 => TE::Reduce(72), 64 => TE::Reduce(72), 65 => TE::Reduce(72), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(72), 82 => TE::Reduce(72), 83 => TE::Reduce(72), 84 => TE::Reduce(72), 86 => TE::Shift(105), 87 => TE::Reduce(72), 88 => TE::Reduce(72), 89 => TE::Reduce(72), 90 => TE::Reduce(72), 91 => TE::Reduce(72), 92 => TE::Reduce(72), 93 => TE::Reduce(72), 94 => TE::Reduce(72), 95 => TE::Reduce(72), 96 => TE::Reduce(72), 97 => TE::Reduce(72), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(73), 41 => TE::Reduce(73), 45 => TE::Reduce(73), 57 => TE::Reduce(73), 58 => TE::Reduce(73), 59 => TE::Reduce(73), 60 => TE::Reduce(73), 61 => TE::Reduce(73), 62 => TE::Reduce(73), 63 => TE::Reduce(73), 64 => TE::Reduce(73), 65 => TE::Reduce(73), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(73), 82 => TE::Reduce(73), 83 => TE::Reduce(73), 84 => TE::Reduce(73), 86 => TE::Shift(105), 87 => TE::Reduce(73), 88 => TE::Reduce(73), 89 => TE::Reduce(73), 90 => TE::Reduce(73), 91 => TE::Reduce(73), 92 => TE::Reduce(73), 93 => TE::Reduce(73), 94 => TE::Reduce(73), 95 => TE::Reduce(73), 96 => TE::Reduce(73), 97 => TE::Reduce(73), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(74), 41 => TE::Reduce(74), 45 => TE::Reduce(74), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Reduce(74), 62 => TE::Reduce(74), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(74), 82 => TE::Reduce(74), 83 => TE::Reduce(74), 84 => TE::Reduce(74), 86 => TE::Shift(105), 87 => TE::Reduce(74), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Reduce(74), 96 => TE::Reduce(74), 97 => TE::Reduce(74), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(75), 41 => TE::Reduce(75), 45 => TE::Reduce(75), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Reduce(75), 62 => TE::Reduce(75), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(75), 82 => TE::Reduce(75), 83 => TE::Reduce(75), 84 => TE::Reduce(75), 86 => TE::Shift(105), 87 => TE::Reduce(75), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Reduce(75), 96 => TE::Reduce(75), 97 => TE::Reduce(75), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(76), 41 => TE::Reduce(76), 45 => TE::Reduce(76), 57 => TE::Reduce(76), 58 => TE::Reduce(76), 61 => TE::Reduce(76), 62 => TE::Reduce(76), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(76), 82 => TE::Reduce(76), 83 => TE::Reduce(76), 84 => TE::Reduce(76), 86 => TE::Shift(105), 87 => TE::Reduce(76), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 95 => TE::Reduce(76), 96 => TE::Reduce(76), 97 => TE::Reduce(76), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(77), 41 => TE::Reduce(77), 45 => TE::Reduce(77), 57 => TE::Reduce(77), 58 => TE::Reduce(77), 61 => TE::Reduce(77), 62 => TE::Reduce(77), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(77), 82 => TE::Reduce(77), 83 => TE::Reduce(77), 84 => TE::Reduce(77), 86 => TE::Shift(105), 87 => TE::Reduce(77), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 95 => TE::Reduce(77), 96 => TE::Reduce(77), 97 => TE::Reduce(77), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(78), 41 => TE::Reduce(78), 45 => TE::Reduce(78), 57 => TE::Reduce(78), 58 => TE::Reduce(78), 61 => TE::Reduce(78), 62 => TE::Reduce(78), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(78), 82 => TE::Reduce(78), 83 => TE::Reduce(78), 84 => TE::Reduce(78), 86 => TE::Shift(105), 87 => TE::Reduce(78), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 95 => TE::Reduce(78), 96 => TE::Reduce(78), 97 => TE::Reduce(78), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(79), 41 => TE::Reduce(79), 45 => TE::Reduce(79), 57 => TE::Reduce(79), 58 => TE::Reduce(79), 61 => TE::Reduce(79), 62 => TE::Reduce(79), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(79), 82 => TE::Reduce(79), 83 => TE::Reduce(79), 84 => TE::Reduce(79), 86 => TE::Shift(105), 87 => TE::Reduce(79), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 95 => TE::Reduce(79), 96 => TE::Reduce(79), 97 => TE::Reduce(79), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(80), 41 => TE::Reduce(80), 45 => TE::Reduce(80), 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Reduce(80), 62 => TE::Reduce(80), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(80), 82 => TE::Reduce(80), 83 => TE::Reduce(80), 84 => TE::Reduce(80), 86 => TE::Shift(105), 87 => TE::Reduce(80), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(81), 41 => TE::Reduce(81), 45 => TE::Reduce(81), 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Reduce(81), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(81), 82 => TE::Reduce(81), 83 => TE::Reduce(81), 84 => TE::Reduce(81), 86 => TE::Shift(105), 87 => TE::Reduce(81), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(82), 41 => TE::Reduce(82), 45 => TE::Reduce(82), 57 => TE::Reduce(82), 58 => TE::Reduce(82), 59 => TE::Reduce(82), 60 => TE::Reduce(82), 61 => TE::Reduce(82), 62 => TE::Reduce(82), 63 => TE::Reduce(82), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(82), 82 => TE::Reduce(82), 83 => TE::Reduce(82), 84 => TE::Reduce(82), 86 => TE::Shift(105), 87 => TE::Reduce(82), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Reduce(82), 94 => TE::Reduce(82), 95 => TE::Reduce(82), 96 => TE::Reduce(82), 97 => TE::Reduce(82), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(83), 41 => TE::Reduce(83), 45 => TE::Reduce(83), 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Reduce(83), 62 => TE::Reduce(83), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(83), 82 => TE::Reduce(83), 83 => TE::Reduce(83), 84 => TE::Reduce(83), 86 => TE::Shift(105), 87 => TE::Reduce(83), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Reduce(83), 96 => TE::Reduce(83), 97 => TE::Reduce(83), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(84), 41 => TE::Reduce(84), 45 => TE::Reduce(84), 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Reduce(84), 62 => TE::Reduce(84), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(84), 82 => TE::Reduce(84), 83 => TE::Reduce(84), 84 => TE::Reduce(84), 86 => TE::Shift(105), 87 => TE::Reduce(84), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Reduce(84), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(85), 41 => TE::Reduce(85), 45 => TE::Reduce(85), 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Reduce(85), 62 => TE::Reduce(85), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(85), 82 => TE::Reduce(85), 83 => TE::Reduce(85), 84 => TE::Reduce(85), 86 => TE::Shift(105), 87 => TE::Reduce(85), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Reduce(85), 97 => TE::Reduce(85), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(86), 41 => TE::Reduce(86), 45 => TE::Reduce(86), 57 => TE::Reduce(86), 58 => TE::Reduce(86), 59 => TE::Reduce(86), 60 => TE::Reduce(86), 61 => TE::Reduce(86), 62 => TE::Reduce(86), 63 => TE::Reduce(86), 64 => TE::Reduce(86), 65 => TE::Reduce(86), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(86), 82 => TE::Reduce(86), 83 => TE::Reduce(86), 84 => TE::Reduce(86), 86 => TE::Shift(105), 87 => TE::Reduce(86), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Reduce(86), 94 => TE::Reduce(86), 95 => TE::Reduce(86), 96 => TE::Reduce(86), 97 => TE::Reduce(86), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(87), 41 => TE::Reduce(87), 45 => TE::Reduce(87), 57 => TE::Reduce(87), 58 => TE::Reduce(87), 59 => TE::Reduce(87), 60 => TE::Reduce(87), 61 => TE::Reduce(87), 62 => TE::Reduce(87), 63 => TE::Reduce(87), 64 => TE::Reduce(87), 65 => TE::Reduce(87), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(87), 82 => TE::Reduce(87), 83 => TE::Reduce(87), 84 => TE::Reduce(87), 86 => TE::Shift(105), 87 => TE::Reduce(87), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Reduce(87), 94 => TE::Reduce(87), 95 => TE::Reduce(87), 96 => TE::Reduce(87), 97 => TE::Reduce(87), 99 => TE::Shift(108) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 84 => TE::Shift(129), 86 => TE::Shift(105), 87 => TE::Shift(130), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 29 => TE::Transit(131), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 40 => TE::Reduce(107), 41 => TE::Reduce(107), 45 => TE::Reduce(107), 57 => TE::Reduce(107), 58 => TE::Reduce(107), 59 => TE::Reduce(107), 60 => TE::Reduce(107), 61 => TE::Reduce(107), 62 => TE::Reduce(107), 63 => TE::Reduce(107), 64 => TE::Reduce(107), 65 => TE::Reduce(107), 66 => TE::Shift(180), 67 => TE::Reduce(107), 68 => TE::Reduce(107), 80 => TE::Reduce(107), 82 => TE::Reduce(107), 83 => TE::Reduce(107), 84 => TE::Reduce(107), 85 => TE::Reduce(107), 86 => TE::Reduce(107), 87 => TE::Reduce(107), 88 => TE::Reduce(107), 89 => TE::Reduce(107), 90 => TE::Reduce(107), 91 => TE::Reduce(107), 92 => TE::Reduce(107), 93 => TE::Reduce(107), 94 => TE::Reduce(107), 95 => TE::Reduce(107), 96 => TE::Reduce(107), 97 => TE::Reduce(107), 99 => TE::Reduce(107) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 86 => TE::Shift(105), 87 => TE::Shift(132), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(88), 41 => TE::Reduce(88), 45 => TE::Reduce(88), 57 => TE::Reduce(88), 58 => TE::Reduce(88), 59 => TE::Reduce(88), 60 => TE::Reduce(88), 61 => TE::Reduce(88), 62 => TE::Reduce(88), 63 => TE::Reduce(88), 64 => TE::Reduce(88), 65 => TE::Reduce(88), 67 => TE::Reduce(88), 68 => TE::Reduce(88), 80 => TE::Reduce(88), 82 => TE::Reduce(88), 83 => TE::Reduce(88), 84 => TE::Reduce(88), 86 => TE::Reduce(88), 87 => TE::Reduce(88), 88 => TE::Reduce(88), 89 => TE::Reduce(88), 90 => TE::Reduce(88), 91 => TE::Reduce(88), 92 => TE::Reduce(88), 93 => TE::Reduce(88), 94 => TE::Reduce(88), 95 => TE::Reduce(88), 96 => TE::Reduce(88), 97 => TE::Reduce(88), 99 => TE::Reduce(88) },
    hashmap! { 40 => TE::Reduce(106), 41 => TE::Reduce(106), 45 => TE::Reduce(106), 57 => TE::Reduce(106), 58 => TE::Reduce(106), 59 => TE::Reduce(106), 60 => TE::Reduce(106), 61 => TE::Reduce(106), 62 => TE::Reduce(106), 63 => TE::Reduce(106), 64 => TE::Reduce(106), 65 => TE::Reduce(106), 67 => TE::Reduce(106), 68 => TE::Reduce(106), 80 => TE::Reduce(106), 81 => TE::Shift(134), 82 => TE::Reduce(106), 83 => TE::Reduce(106), 84 => TE::Reduce(106), 85 => TE::Reduce(106), 86 => TE::Reduce(106), 87 => TE::Reduce(106), 88 => TE::Reduce(106), 89 => TE::Reduce(106), 90 => TE::Reduce(106), 91 => TE::Reduce(106), 92 => TE::Reduce(106), 93 => TE::Reduce(106), 94 => TE::Reduce(106), 95 => TE::Reduce(106), 96 => TE::Reduce(106), 97 => TE::Reduce(106), 99 => TE::Reduce(106) },
    hashmap! { 27 => TE::Transit(136), 29 => TE::Transit(137), 30 => TE::Transit(85), 31 => TE::Transit(53), 32 => TE::Transit(135), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 82 => TE::Reduce(111), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 82 => TE::Shift(138) },
    hashmap! { 82 => TE::Reduce(110), 83 => TE::Shift(139) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 82 => TE::Reduce(54), 83 => TE::Reduce(54), 86 => TE::Shift(105), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(62), 41 => TE::Reduce(62), 45 => TE::Reduce(62), 57 => TE::Reduce(62), 58 => TE::Reduce(62), 59 => TE::Reduce(62), 60 => TE::Reduce(62), 61 => TE::Reduce(62), 62 => TE::Reduce(62), 63 => TE::Reduce(62), 64 => TE::Reduce(62), 65 => TE::Reduce(62), 67 => TE::Reduce(62), 68 => TE::Reduce(62), 80 => TE::Reduce(62), 82 => TE::Reduce(62), 83 => TE::Reduce(62), 84 => TE::Reduce(62), 86 => TE::Reduce(62), 87 => TE::Reduce(62), 88 => TE::Reduce(62), 89 => TE::Reduce(62), 90 => TE::Reduce(62), 91 => TE::Reduce(62), 92 => TE::Reduce(62), 93 => TE::Reduce(62), 94 => TE::Reduce(62), 95 => TE::Reduce(62), 96 => TE::Reduce(62), 97 => TE::Reduce(62), 99 => TE::Reduce(62) },
    hashmap! { 29 => TE::Transit(140), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 82 => TE::Reduce(53), 83 => TE::Reduce(53), 86 => TE::Shift(105), 87 => TE::Reduce(53), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 83 => TE::Shift(139), 87 => TE::Shift(143) },
    hashmap! { 41 => TE::Shift(144), 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 83 => TE::Reduce(54), 86 => TE::Shift(105), 87 => TE::Reduce(54), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(67), 41 => TE::Reduce(67), 45 => TE::Reduce(67), 57 => TE::Reduce(67), 58 => TE::Reduce(67), 59 => TE::Reduce(67), 60 => TE::Reduce(67), 61 => TE::Reduce(67), 62 => TE::Reduce(67), 63 => TE::Reduce(67), 64 => TE::Reduce(67), 65 => TE::Reduce(67), 67 => TE::Reduce(67), 68 => TE::Reduce(67), 80 => TE::Reduce(67), 82 => TE::Reduce(67), 83 => TE::Reduce(67), 84 => TE::Reduce(67), 86 => TE::Reduce(67), 87 => TE::Reduce(67), 88 => TE::Reduce(67), 89 => TE::Reduce(67), 90 => TE::Reduce(67), 91 => TE::Reduce(67), 92 => TE::Reduce(67), 93 => TE::Reduce(67), 94 => TE::Reduce(67), 95 => TE::Reduce(67), 96 => TE::Reduce(67), 97 => TE::Reduce(67), 99 => TE::Reduce(67) },
    hashmap! { 36 => TE::Shift(145) },
    hashmap! { 43 => TE::Shift(146) },
    hashmap! { 29 => TE::Transit(147), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 45 => TE::Shift(149), 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 86 => TE::Shift(105), 87 => TE::Shift(148), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(90), 41 => TE::Reduce(90), 45 => TE::Reduce(90), 57 => TE::Reduce(90), 58 => TE::Reduce(90), 59 => TE::Reduce(90), 60 => TE::Reduce(90), 61 => TE::Reduce(90), 62 => TE::Reduce(90), 63 => TE::Reduce(90), 64 => TE::Reduce(90), 65 => TE::Reduce(90), 67 => TE::Reduce(90), 68 => TE::Reduce(90), 80 => TE::Reduce(90), 82 => TE::Reduce(90), 83 => TE::Reduce(90), 84 => TE::Reduce(90), 86 => TE::Reduce(90), 87 => TE::Reduce(90), 88 => TE::Reduce(90), 89 => TE::Reduce(90), 90 => TE::Reduce(90), 91 => TE::Reduce(90), 92 => TE::Reduce(90), 93 => TE::Reduce(90), 94 => TE::Reduce(90), 95 => TE::Reduce(90), 96 => TE::Reduce(90), 97 => TE::Reduce(90), 99 => TE::Reduce(90) },
    hashmap! { 29 => TE::Transit(150), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 86 => TE::Shift(105), 87 => TE::Shift(151), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(91), 41 => TE::Reduce(91), 45 => TE::Reduce(91), 57 => TE::Reduce(91), 58 => TE::Reduce(91), 59 => TE::Reduce(91), 60 => TE::Reduce(91), 61 => TE::Reduce(91), 62 => TE::Reduce(91), 63 => TE::Reduce(91), 64 => TE::Reduce(91), 65 => TE::Reduce(91), 67 => TE::Reduce(91), 68 => TE::Reduce(91), 80 => TE::Reduce(91), 82 => TE::Reduce(91), 83 => TE::Reduce(91), 84 => TE::Reduce(91), 86 => TE::Reduce(91), 87 => TE::Reduce(91), 88 => TE::Reduce(91), 89 => TE::Reduce(91), 90 => TE::Reduce(91), 91 => TE::Reduce(91), 92 => TE::Reduce(91), 93 => TE::Reduce(91), 94 => TE::Reduce(91), 95 => TE::Reduce(91), 96 => TE::Reduce(91), 97 => TE::Reduce(91), 99 => TE::Reduce(91) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 82 => TE::Shift(154), 86 => TE::Shift(105), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 36 => TE::Shift(155) },
    hashmap! { 40 => TE::Reduce(92), 41 => TE::Reduce(92), 45 => TE::Reduce(92), 57 => TE::Reduce(92), 58 => TE::Reduce(92), 59 => TE::Reduce(92), 60 => TE::Reduce(92), 61 => TE::Reduce(92), 62 => TE::Reduce(92), 63 => TE::Reduce(92), 64 => TE::Reduce(92), 65 => TE::Reduce(92), 67 => TE::Reduce(92), 68 => TE::Reduce(92), 80 => TE::Reduce(92), 82 => TE::Reduce(92), 83 => TE::Reduce(92), 84 => TE::Reduce(92), 86 => TE::Reduce(92), 87 => TE::Reduce(92), 88 => TE::Reduce(92), 89 => TE::Reduce(92), 90 => TE::Reduce(92), 91 => TE::Reduce(92), 92 => TE::Reduce(92), 93 => TE::Reduce(92), 94 => TE::Reduce(92), 95 => TE::Reduce(92), 96 => TE::Reduce(92), 97 => TE::Reduce(92), 99 => TE::Reduce(92) },
    hashmap! { 82 => TE::Shift(156) },
    hashmap! { 29 => TE::Transit(157), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 40 => TE::Reduce(105), 41 => TE::Reduce(105), 45 => TE::Reduce(105), 57 => TE::Reduce(105), 58 => TE::Reduce(105), 59 => TE::Reduce(105), 60 => TE::Reduce(105), 61 => TE::Reduce(105), 62 => TE::Reduce(105), 63 => TE::Reduce(105), 64 => TE::Reduce(105), 65 => TE::Reduce(105), 67 => TE::Reduce(105), 68 => TE::Reduce(105), 80 => TE::Reduce(105), 82 => TE::Reduce(105), 83 => TE::Reduce(105), 84 => TE::Reduce(105), 86 => TE::Reduce(105), 87 => TE::Reduce(105), 88 => TE::Reduce(105), 89 => TE::Reduce(105), 90 => TE::Reduce(105), 91 => TE::Reduce(105), 92 => TE::Reduce(105), 93 => TE::Reduce(105), 94 => TE::Reduce(105), 95 => TE::Reduce(105), 96 => TE::Reduce(105), 97 => TE::Reduce(105), 99 => TE::Reduce(105) },
    hashmap! { 40 => TE::Reduce(93), 41 => TE::Reduce(93), 45 => TE::Reduce(93), 57 => TE::Reduce(93), 58 => TE::Reduce(93), 59 => TE::Reduce(93), 60 => TE::Reduce(93), 61 => TE::Reduce(93), 62 => TE::Reduce(93), 63 => TE::Reduce(93), 64 => TE::Reduce(93), 65 => TE::Reduce(93), 80 => TE::Reduce(93), 82 => TE::Reduce(93), 83 => TE::Reduce(93), 84 => TE::Reduce(93), 86 => TE::Shift(105), 87 => TE::Reduce(93), 88 => TE::Reduce(93), 89 => TE::Reduce(93), 90 => TE::Reduce(93), 91 => TE::Reduce(93), 92 => TE::Reduce(93), 93 => TE::Reduce(93), 94 => TE::Reduce(93), 95 => TE::Reduce(93), 96 => TE::Reduce(93), 97 => TE::Reduce(93), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(94), 41 => TE::Reduce(94), 45 => TE::Reduce(94), 57 => TE::Reduce(94), 58 => TE::Reduce(94), 59 => TE::Reduce(94), 60 => TE::Reduce(94), 61 => TE::Reduce(94), 62 => TE::Reduce(94), 63 => TE::Reduce(94), 64 => TE::Reduce(94), 65 => TE::Reduce(94), 80 => TE::Reduce(94), 82 => TE::Reduce(94), 83 => TE::Reduce(94), 84 => TE::Reduce(94), 86 => TE::Shift(105), 87 => TE::Reduce(94), 88 => TE::Reduce(94), 89 => TE::Reduce(94), 90 => TE::Reduce(94), 91 => TE::Reduce(94), 92 => TE::Reduce(94), 93 => TE::Reduce(94), 94 => TE::Reduce(94), 95 => TE::Reduce(94), 96 => TE::Reduce(94), 97 => TE::Reduce(94), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(95), 41 => TE::Reduce(95), 45 => TE::Reduce(95), 57 => TE::Reduce(95), 58 => TE::Reduce(95), 59 => TE::Reduce(95), 60 => TE::Reduce(95), 61 => TE::Reduce(95), 62 => TE::Reduce(95), 63 => TE::Reduce(95), 64 => TE::Reduce(95), 65 => TE::Reduce(95), 80 => TE::Reduce(95), 82 => TE::Reduce(95), 83 => TE::Reduce(95), 84 => TE::Reduce(95), 86 => TE::Shift(105), 87 => TE::Reduce(95), 88 => TE::Reduce(95), 89 => TE::Reduce(95), 90 => TE::Reduce(95), 91 => TE::Reduce(95), 92 => TE::Reduce(95), 93 => TE::Reduce(95), 94 => TE::Reduce(95), 95 => TE::Reduce(95), 96 => TE::Reduce(95), 97 => TE::Reduce(95), 99 => TE::Shift(108) },
    hashmap! { 40 => TE::Reduce(96), 41 => TE::Reduce(96), 45 => TE::Reduce(96), 57 => TE::Reduce(96), 58 => TE::Reduce(96), 59 => TE::Reduce(96), 60 => TE::Reduce(96), 61 => TE::Reduce(96), 62 => TE::Reduce(96), 63 => TE::Reduce(96), 64 => TE::Reduce(96), 65 => TE::Reduce(96), 80 => TE::Reduce(96), 82 => TE::Reduce(96), 83 => TE::Reduce(96), 84 => TE::Reduce(96), 86 => TE::Shift(105), 87 => TE::Reduce(96), 88 => TE::Reduce(96), 89 => TE::Reduce(96), 90 => TE::Reduce(96), 91 => TE::Reduce(96), 92 => TE::Reduce(96), 93 => TE::Reduce(96), 94 => TE::Reduce(96), 95 => TE::Reduce(96), 96 => TE::Reduce(96), 97 => TE::Reduce(96), 99 => TE::Shift(108) },
    hashmap! { 82 => TE::Shift(163) },
    hashmap! { 40 => TE::Reduce(99), 41 => TE::Reduce(99), 45 => TE::Reduce(99), 57 => TE::Reduce(99), 58 => TE::Reduce(99), 59 => TE::Reduce(99), 60 => TE::Reduce(99), 61 => TE::Reduce(99), 62 => TE::Reduce(99), 63 => TE::Reduce(99), 64 => TE::Reduce(99), 65 => TE::Reduce(99), 67 => TE::Reduce(99), 68 => TE::Reduce(99), 80 => TE::Reduce(99), 82 => TE::Reduce(99), 83 => TE::Reduce(99), 84 => TE::Reduce(99), 86 => TE::Reduce(99), 87 => TE::Reduce(99), 88 => TE::Reduce(99), 89 => TE::Reduce(99), 90 => TE::Reduce(99), 91 => TE::Reduce(99), 92 => TE::Reduce(99), 93 => TE::Reduce(99), 94 => TE::Reduce(99), 95 => TE::Reduce(99), 96 => TE::Reduce(99), 97 => TE::Reduce(99), 99 => TE::Reduce(99) },
    hashmap! { 82 => TE::Shift(165) },
    hashmap! { 40 => TE::Reduce(100), 41 => TE::Reduce(100), 45 => TE::Reduce(100), 57 => TE::Reduce(100), 58 => TE::Reduce(100), 59 => TE::Reduce(100), 60 => TE::Reduce(100), 61 => TE::Reduce(100), 62 => TE::Reduce(100), 63 => TE::Reduce(100), 64 => TE::Reduce(100), 65 => TE::Reduce(100), 67 => TE::Reduce(100), 68 => TE::Reduce(100), 80 => TE::Reduce(100), 82 => TE::Reduce(100), 83 => TE::Reduce(100), 84 => TE::Reduce(100), 86 => TE::Reduce(100), 87 => TE::Reduce(100), 88 => TE::Reduce(100), 89 => TE::Reduce(100), 90 => TE::Reduce(100), 91 => TE::Reduce(100), 92 => TE::Reduce(100), 93 => TE::Reduce(100), 94 => TE::Reduce(100), 95 => TE::Reduce(100), 96 => TE::Reduce(100), 97 => TE::Reduce(100), 99 => TE::Reduce(100) },
    hashmap! { 81 => TE::Shift(168) },
    hashmap! { 86 => TE::Shift(170) },
    hashmap! { 82 => TE::Shift(169) },
    hashmap! { 40 => TE::Reduce(102), 41 => TE::Reduce(102), 45 => TE::Reduce(102), 57 => TE::Reduce(102), 58 => TE::Reduce(102), 59 => TE::Reduce(102), 60 => TE::Reduce(102), 61 => TE::Reduce(102), 62 => TE::Reduce(102), 63 => TE::Reduce(102), 64 => TE::Reduce(102), 65 => TE::Reduce(102), 67 => TE::Reduce(102), 68 => TE::Reduce(102), 80 => TE::Reduce(102), 82 => TE::Reduce(102), 83 => TE::Reduce(102), 84 => TE::Reduce(102), 86 => TE::Reduce(102), 87 => TE::Reduce(102), 88 => TE::Reduce(102), 89 => TE::Reduce(102), 90 => TE::Reduce(102), 91 => TE::Reduce(102), 92 => TE::Reduce(102), 93 => TE::Reduce(102), 94 => TE::Reduce(102), 95 => TE::Reduce(102), 96 => TE::Reduce(102), 97 => TE::Reduce(102), 99 => TE::Reduce(102) },
    hashmap! { 29 => TE::Transit(171), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 87 => TE::Shift(172), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 86 => TE::Shift(105), 87 => TE::Shift(173), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 36 => TE::Reduce(118), 86 => TE::Reduce(118) },
    hashmap! { 40 => TE::Reduce(103), 41 => TE::Reduce(103), 45 => TE::Reduce(103), 57 => TE::Reduce(103), 58 => TE::Reduce(103), 59 => TE::Reduce(103), 60 => TE::Reduce(103), 61 => TE::Reduce(103), 62 => TE::Reduce(103), 63 => TE::Reduce(103), 64 => TE::Reduce(103), 65 => TE::Reduce(103), 67 => TE::Reduce(103), 68 => TE::Reduce(103), 80 => TE::Reduce(103), 82 => TE::Reduce(103), 83 => TE::Reduce(103), 84 => TE::Reduce(103), 86 => TE::Reduce(103), 87 => TE::Reduce(103), 88 => TE::Reduce(103), 89 => TE::Reduce(103), 90 => TE::Reduce(103), 91 => TE::Reduce(103), 92 => TE::Reduce(103), 93 => TE::Reduce(103), 94 => TE::Reduce(103), 95 => TE::Reduce(103), 96 => TE::Reduce(103), 97 => TE::Reduce(103), 99 => TE::Reduce(103) },
    hashmap! { 29 => TE::Transit(175), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 83 => TE::Shift(176), 86 => TE::Shift(105), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 36 => TE::Shift(177) },
    hashmap! { 82 => TE::Shift(178) },
    hashmap! { 40 => TE::Reduce(104), 41 => TE::Reduce(104), 45 => TE::Reduce(104), 57 => TE::Reduce(104), 58 => TE::Reduce(104), 59 => TE::Reduce(104), 60 => TE::Reduce(104), 61 => TE::Reduce(104), 62 => TE::Reduce(104), 63 => TE::Reduce(104), 64 => TE::Reduce(104), 65 => TE::Reduce(104), 67 => TE::Reduce(104), 68 => TE::Reduce(104), 80 => TE::Reduce(104), 82 => TE::Reduce(104), 83 => TE::Reduce(104), 84 => TE::Reduce(104), 86 => TE::Reduce(104), 87 => TE::Reduce(104), 88 => TE::Reduce(104), 89 => TE::Reduce(104), 90 => TE::Reduce(104), 91 => TE::Reduce(104), 92 => TE::Reduce(104), 93 => TE::Reduce(104), 94 => TE::Reduce(104), 95 => TE::Reduce(104), 96 => TE::Reduce(104), 97 => TE::Reduce(104), 99 => TE::Reduce(104) },
    hashmap! { 36 => TE::Reduce(117), 86 => TE::Reduce(117) },
    hashmap! { 29 => TE::Transit(181), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 40 => TE::Reduce(89), 41 => TE::Reduce(89), 45 => TE::Reduce(89), 57 => TE::Reduce(89), 58 => TE::Reduce(89), 59 => TE::Reduce(89), 60 => TE::Reduce(89), 61 => TE::Reduce(89), 62 => TE::Reduce(89), 63 => TE::Reduce(89), 64 => TE::Reduce(89), 65 => TE::Reduce(89), 67 => TE::Reduce(89), 68 => TE::Reduce(89), 80 => TE::Reduce(89), 82 => TE::Reduce(89), 83 => TE::Reduce(89), 84 => TE::Reduce(89), 87 => TE::Reduce(89), 88 => TE::Reduce(89), 89 => TE::Reduce(89), 90 => TE::Reduce(89), 91 => TE::Reduce(89), 92 => TE::Reduce(89), 93 => TE::Reduce(89), 94 => TE::Reduce(89), 95 => TE::Reduce(89), 96 => TE::Reduce(89), 97 => TE::Reduce(89) },
    hashmap! { 80 => TE::Reduce(112), 82 => TE::Reduce(112), 85 => TE::Shift(183) },
    hashmap! { 29 => TE::Transit(184), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(56), 82 => TE::Reduce(56), 86 => TE::Shift(105), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 85 => TE::Shift(186) },
    hashmap! { 29 => TE::Transit(187), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(57), 82 => TE::Reduce(57), 86 => TE::Shift(105), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 29 => TE::Transit(190), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 23 => TE::Transit(230), 24 => TE::Transit(231), 29 => TE::Transit(232), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 79 => TE::Reduce(47), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 82 => TE::Shift(191), 86 => TE::Shift(105), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 9 => TE::Transit(47), 11 => TE::Transit(193), 12 => TE::Transit(192), 13 => TE::Transit(39), 14 => TE::Transit(40), 15 => TE::Transit(45), 16 => TE::Transit(43), 17 => TE::Transit(38), 19 => TE::Transit(44), 22 => TE::Transit(46), 25 => TE::Transit(41), 26 => TE::Transit(42), 28 => TE::Transit(37), 29 => TE::Transit(52), 30 => TE::Transit(48), 31 => TE::Transit(53), 33 => TE::Transit(51), 34 => TE::Transit(49), 35 => TE::Shift(21), 36 => TE::Reduce(109), 40 => TE::Shift(71), 41 => TE::Shift(72), 42 => TE::Shift(77), 44 => TE::Shift(75), 45 => TE::Shift(70), 47 => TE::Shift(76), 48 => TE::Shift(50), 50 => TE::Shift(73), 51 => TE::Shift(74), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 78 => TE::Shift(33), 80 => TE::Reduce(60), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 18 => TE::Transit(194), 35 => TE::Reduce(39), 36 => TE::Reduce(39), 40 => TE::Reduce(39), 41 => TE::Reduce(39), 42 => TE::Reduce(39), 44 => TE::Reduce(39), 45 => TE::Reduce(39), 46 => TE::Shift(195), 47 => TE::Reduce(39), 48 => TE::Reduce(39), 49 => TE::Reduce(39), 50 => TE::Reduce(39), 51 => TE::Reduce(39), 52 => TE::Reduce(39), 53 => TE::Reduce(39), 54 => TE::Reduce(39), 55 => TE::Reduce(39), 56 => TE::Reduce(39), 67 => TE::Reduce(39), 68 => TE::Reduce(39), 69 => TE::Reduce(39), 70 => TE::Reduce(39), 71 => TE::Reduce(39), 72 => TE::Reduce(39), 73 => TE::Reduce(39), 74 => TE::Reduce(39), 75 => TE::Reduce(39), 76 => TE::Reduce(39), 77 => TE::Reduce(39), 78 => TE::Reduce(39), 79 => TE::Reduce(39), 80 => TE::Reduce(39), 81 => TE::Reduce(39), 86 => TE::Reduce(39), 89 => TE::Reduce(39), 98 => TE::Reduce(39) },
    hashmap! { 35 => TE::Reduce(32), 36 => TE::Reduce(32), 40 => TE::Reduce(32), 41 => TE::Reduce(32), 42 => TE::Reduce(32), 44 => TE::Reduce(32), 45 => TE::Reduce(32), 46 => TE::Reduce(32), 47 => TE::Reduce(32), 48 => TE::Reduce(32), 49 => TE::Reduce(32), 50 => TE::Reduce(32), 51 => TE::Reduce(32), 52 => TE::Reduce(32), 53 => TE::Reduce(32), 54 => TE::Reduce(32), 55 => TE::Reduce(32), 56 => TE::Reduce(32), 67 => TE::Reduce(32), 68 => TE::Reduce(32), 69 => TE::Reduce(32), 70 => TE::Reduce(32), 71 => TE::Reduce(32), 72 => TE::Reduce(32), 73 => TE::Reduce(32), 74 => TE::Reduce(32), 75 => TE::Reduce(32), 76 => TE::Reduce(32), 77 => TE::Reduce(32), 78 => TE::Reduce(32), 79 => TE::Reduce(32), 80 => TE::Reduce(32), 81 => TE::Reduce(32), 86 => TE::Reduce(32), 89 => TE::Reduce(32), 98 => TE::Reduce(32) },
    hashmap! { 35 => TE::Reduce(37), 36 => TE::Reduce(37), 40 => TE::Reduce(37), 41 => TE::Reduce(37), 42 => TE::Reduce(37), 44 => TE::Reduce(37), 45 => TE::Reduce(37), 46 => TE::Reduce(37), 47 => TE::Reduce(37), 48 => TE::Reduce(37), 49 => TE::Reduce(37), 50 => TE::Reduce(37), 51 => TE::Reduce(37), 52 => TE::Reduce(37), 53 => TE::Reduce(37), 54 => TE::Reduce(37), 55 => TE::Reduce(37), 56 => TE::Reduce(37), 67 => TE::Reduce(37), 68 => TE::Reduce(37), 69 => TE::Reduce(37), 70 => TE::Reduce(37), 71 => TE::Reduce(37), 72 => TE::Reduce(37), 73 => TE::Reduce(37), 74 => TE::Reduce(37), 75 => TE::Reduce(37), 76 => TE::Reduce(37), 77 => TE::Reduce(37), 78 => TE::Reduce(37), 79 => TE::Reduce(37), 80 => TE::Reduce(37), 81 => TE::Reduce(37), 86 => TE::Reduce(37), 89 => TE::Reduce(37), 98 => TE::Reduce(37) },
    hashmap! { 9 => TE::Transit(47), 11 => TE::Transit(193), 12 => TE::Transit(196), 13 => TE::Transit(39), 14 => TE::Transit(40), 15 => TE::Transit(45), 16 => TE::Transit(43), 17 => TE::Transit(38), 19 => TE::Transit(44), 22 => TE::Transit(46), 25 => TE::Transit(41), 26 => TE::Transit(42), 28 => TE::Transit(37), 29 => TE::Transit(52), 30 => TE::Transit(48), 31 => TE::Transit(53), 33 => TE::Transit(51), 34 => TE::Transit(49), 35 => TE::Shift(21), 36 => TE::Reduce(109), 40 => TE::Shift(71), 41 => TE::Shift(72), 42 => TE::Shift(77), 44 => TE::Shift(75), 45 => TE::Shift(70), 47 => TE::Shift(76), 48 => TE::Shift(50), 50 => TE::Shift(73), 51 => TE::Shift(74), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 78 => TE::Shift(33), 80 => TE::Reduce(60), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 35 => TE::Reduce(38), 36 => TE::Reduce(38), 40 => TE::Reduce(38), 41 => TE::Reduce(38), 42 => TE::Reduce(38), 44 => TE::Reduce(38), 45 => TE::Reduce(38), 46 => TE::Reduce(38), 47 => TE::Reduce(38), 48 => TE::Reduce(38), 49 => TE::Reduce(38), 50 => TE::Reduce(38), 51 => TE::Reduce(38), 52 => TE::Reduce(38), 53 => TE::Reduce(38), 54 => TE::Reduce(38), 55 => TE::Reduce(38), 56 => TE::Reduce(38), 67 => TE::Reduce(38), 68 => TE::Reduce(38), 69 => TE::Reduce(38), 70 => TE::Reduce(38), 71 => TE::Reduce(38), 72 => TE::Reduce(38), 73 => TE::Reduce(38), 74 => TE::Reduce(38), 75 => TE::Reduce(38), 76 => TE::Reduce(38), 77 => TE::Reduce(38), 78 => TE::Reduce(38), 79 => TE::Reduce(38), 80 => TE::Reduce(38), 81 => TE::Reduce(38), 86 => TE::Reduce(38), 89 => TE::Reduce(38), 98 => TE::Reduce(38) },
    hashmap! { 29 => TE::Transit(198), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 82 => TE::Shift(199), 86 => TE::Shift(105), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 9 => TE::Transit(47), 11 => TE::Transit(193), 12 => TE::Transit(200), 13 => TE::Transit(39), 14 => TE::Transit(40), 15 => TE::Transit(45), 16 => TE::Transit(43), 17 => TE::Transit(38), 19 => TE::Transit(44), 22 => TE::Transit(46), 25 => TE::Transit(41), 26 => TE::Transit(42), 28 => TE::Transit(37), 29 => TE::Transit(52), 30 => TE::Transit(48), 31 => TE::Transit(53), 33 => TE::Transit(51), 34 => TE::Transit(49), 35 => TE::Shift(21), 36 => TE::Reduce(109), 40 => TE::Shift(71), 41 => TE::Shift(72), 42 => TE::Shift(77), 44 => TE::Shift(75), 45 => TE::Shift(70), 47 => TE::Shift(76), 48 => TE::Shift(50), 50 => TE::Shift(73), 51 => TE::Shift(74), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 78 => TE::Shift(33), 80 => TE::Reduce(60), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 35 => TE::Reduce(33), 36 => TE::Reduce(33), 40 => TE::Reduce(33), 41 => TE::Reduce(33), 42 => TE::Reduce(33), 44 => TE::Reduce(33), 45 => TE::Reduce(33), 46 => TE::Reduce(33), 47 => TE::Reduce(33), 48 => TE::Reduce(33), 49 => TE::Reduce(33), 50 => TE::Reduce(33), 51 => TE::Reduce(33), 52 => TE::Reduce(33), 53 => TE::Reduce(33), 54 => TE::Reduce(33), 55 => TE::Reduce(33), 56 => TE::Reduce(33), 67 => TE::Reduce(33), 68 => TE::Reduce(33), 69 => TE::Reduce(33), 70 => TE::Reduce(33), 71 => TE::Reduce(33), 72 => TE::Reduce(33), 73 => TE::Reduce(33), 74 => TE::Reduce(33), 75 => TE::Reduce(33), 76 => TE::Reduce(33), 77 => TE::Reduce(33), 78 => TE::Reduce(33), 79 => TE::Reduce(33), 80 => TE::Reduce(33), 81 => TE::Reduce(33), 86 => TE::Reduce(33), 89 => TE::Reduce(33), 98 => TE::Reduce(33) },
    hashmap! { 28 => TE::Transit(202), 29 => TE::Transit(52), 30 => TE::Transit(48), 31 => TE::Transit(53), 33 => TE::Transit(51), 34 => TE::Transit(49), 35 => TE::Shift(21), 36 => TE::Reduce(109), 48 => TE::Shift(50), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 80 => TE::Reduce(60), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 80 => TE::Shift(203) },
    hashmap! { 29 => TE::Transit(204), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Shift(205), 86 => TE::Shift(105), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 28 => TE::Transit(206), 29 => TE::Transit(52), 30 => TE::Transit(48), 31 => TE::Transit(53), 33 => TE::Transit(51), 34 => TE::Transit(49), 35 => TE::Shift(21), 36 => TE::Reduce(109), 48 => TE::Shift(50), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 81 => TE::Shift(60), 82 => TE::Reduce(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 82 => TE::Shift(207) },
    hashmap! { 9 => TE::Transit(47), 11 => TE::Transit(193), 12 => TE::Transit(208), 13 => TE::Transit(39), 14 => TE::Transit(40), 15 => TE::Transit(45), 16 => TE::Transit(43), 17 => TE::Transit(38), 19 => TE::Transit(44), 22 => TE::Transit(46), 25 => TE::Transit(41), 26 => TE::Transit(42), 28 => TE::Transit(37), 29 => TE::Transit(52), 30 => TE::Transit(48), 31 => TE::Transit(53), 33 => TE::Transit(51), 34 => TE::Transit(49), 35 => TE::Shift(21), 36 => TE::Reduce(109), 40 => TE::Shift(71), 41 => TE::Shift(72), 42 => TE::Shift(77), 44 => TE::Shift(75), 45 => TE::Shift(70), 47 => TE::Shift(76), 48 => TE::Shift(50), 50 => TE::Shift(73), 51 => TE::Shift(74), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 78 => TE::Shift(33), 80 => TE::Reduce(60), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 35 => TE::Reduce(34), 36 => TE::Reduce(34), 40 => TE::Reduce(34), 41 => TE::Reduce(34), 42 => TE::Reduce(34), 44 => TE::Reduce(34), 45 => TE::Reduce(34), 46 => TE::Reduce(34), 47 => TE::Reduce(34), 48 => TE::Reduce(34), 49 => TE::Reduce(34), 50 => TE::Reduce(34), 51 => TE::Reduce(34), 52 => TE::Reduce(34), 53 => TE::Reduce(34), 54 => TE::Reduce(34), 55 => TE::Reduce(34), 56 => TE::Reduce(34), 67 => TE::Reduce(34), 68 => TE::Reduce(34), 69 => TE::Reduce(34), 70 => TE::Reduce(34), 71 => TE::Reduce(34), 72 => TE::Reduce(34), 73 => TE::Reduce(34), 74 => TE::Reduce(34), 75 => TE::Reduce(34), 76 => TE::Reduce(34), 77 => TE::Reduce(34), 78 => TE::Reduce(34), 79 => TE::Reduce(34), 80 => TE::Reduce(34), 81 => TE::Reduce(34), 86 => TE::Reduce(34), 89 => TE::Reduce(34), 98 => TE::Reduce(34) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 80 => TE::Reduce(50), 86 => TE::Shift(105), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 27 => TE::Transit(211), 29 => TE::Transit(137), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 82 => TE::Shift(212), 83 => TE::Shift(139) },
    hashmap! { 80 => TE::Reduce(52) },
    hashmap! { 36 => TE::Shift(214) },
    hashmap! { 83 => TE::Shift(215) },
    hashmap! { 29 => TE::Transit(216), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 82 => TE::Shift(217), 86 => TE::Shift(105), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 80 => TE::Reduce(40) },
    hashmap! { 20 => TE::Transit(219), 34 => TE::Transit(221), 35 => TE::Shift(21), 48 => TE::Shift(220), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20) },
    hashmap! { 36 => TE::Shift(222) },
    hashmap! { 36 => TE::Reduce(41) },
    hashmap! { 36 => TE::Reduce(42), 86 => TE::Shift(25) },
    hashmap! { 43 => TE::Shift(223) },
    hashmap! { 29 => TE::Transit(224), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 21 => TE::Transit(225), 40 => TE::Shift(226), 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 82 => TE::Reduce(44), 86 => TE::Shift(105), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 82 => TE::Shift(227) },
    hashmap! { 29 => TE::Transit(229), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 9 => TE::Transit(47), 11 => TE::Transit(193), 12 => TE::Transit(228), 13 => TE::Transit(39), 14 => TE::Transit(40), 15 => TE::Transit(45), 16 => TE::Transit(43), 17 => TE::Transit(38), 19 => TE::Transit(44), 22 => TE::Transit(46), 25 => TE::Transit(41), 26 => TE::Transit(42), 28 => TE::Transit(37), 29 => TE::Transit(52), 30 => TE::Transit(48), 31 => TE::Transit(53), 33 => TE::Transit(51), 34 => TE::Transit(49), 35 => TE::Shift(21), 36 => TE::Reduce(109), 40 => TE::Shift(71), 41 => TE::Shift(72), 42 => TE::Shift(77), 44 => TE::Shift(75), 45 => TE::Shift(70), 47 => TE::Shift(76), 48 => TE::Shift(50), 50 => TE::Shift(73), 51 => TE::Shift(74), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 78 => TE::Shift(33), 80 => TE::Reduce(60), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 35 => TE::Reduce(35), 36 => TE::Reduce(35), 40 => TE::Reduce(35), 41 => TE::Reduce(35), 42 => TE::Reduce(35), 44 => TE::Reduce(35), 45 => TE::Reduce(35), 46 => TE::Reduce(35), 47 => TE::Reduce(35), 48 => TE::Reduce(35), 49 => TE::Reduce(35), 50 => TE::Reduce(35), 51 => TE::Reduce(35), 52 => TE::Reduce(35), 53 => TE::Reduce(35), 54 => TE::Reduce(35), 55 => TE::Reduce(35), 56 => TE::Reduce(35), 67 => TE::Reduce(35), 68 => TE::Reduce(35), 69 => TE::Reduce(35), 70 => TE::Reduce(35), 71 => TE::Reduce(35), 72 => TE::Reduce(35), 73 => TE::Reduce(35), 74 => TE::Reduce(35), 75 => TE::Reduce(35), 76 => TE::Reduce(35), 77 => TE::Reduce(35), 78 => TE::Reduce(35), 79 => TE::Reduce(35), 80 => TE::Reduce(35), 81 => TE::Reduce(35), 86 => TE::Reduce(35), 89 => TE::Reduce(35), 98 => TE::Reduce(35) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 82 => TE::Reduce(43), 86 => TE::Shift(105), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 79 => TE::Shift(233) },
    hashmap! { 49 => TE::Shift(234), 79 => TE::Reduce(46) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 84 => TE::Shift(238), 86 => TE::Shift(105), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 35 => TE::Reduce(45), 36 => TE::Reduce(45), 40 => TE::Reduce(45), 41 => TE::Reduce(45), 42 => TE::Reduce(45), 44 => TE::Reduce(45), 45 => TE::Reduce(45), 46 => TE::Reduce(45), 47 => TE::Reduce(45), 48 => TE::Reduce(45), 49 => TE::Reduce(45), 50 => TE::Reduce(45), 51 => TE::Reduce(45), 52 => TE::Reduce(45), 53 => TE::Reduce(45), 54 => TE::Reduce(45), 55 => TE::Reduce(45), 56 => TE::Reduce(45), 67 => TE::Reduce(45), 68 => TE::Reduce(45), 69 => TE::Reduce(45), 70 => TE::Reduce(45), 71 => TE::Reduce(45), 72 => TE::Reduce(45), 73 => TE::Reduce(45), 74 => TE::Reduce(45), 75 => TE::Reduce(45), 76 => TE::Reduce(45), 77 => TE::Reduce(45), 78 => TE::Reduce(45), 79 => TE::Reduce(45), 80 => TE::Reduce(45), 81 => TE::Reduce(45), 86 => TE::Reduce(45), 89 => TE::Reduce(45), 98 => TE::Reduce(45) },
    hashmap! { 29 => TE::Transit(235), 30 => TE::Transit(85), 31 => TE::Transit(53), 36 => TE::Reduce(109), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 57 => TE::Shift(91), 58 => TE::Shift(92), 59 => TE::Shift(95), 60 => TE::Shift(96), 61 => TE::Shift(97), 62 => TE::Shift(98), 63 => TE::Shift(99), 64 => TE::Shift(103), 65 => TE::Shift(104), 67 => TE::Shift(106), 68 => TE::Shift(107), 84 => TE::Shift(236), 86 => TE::Shift(105), 88 => TE::Shift(86), 89 => TE::Shift(87), 90 => TE::Shift(88), 91 => TE::Shift(89), 92 => TE::Shift(90), 93 => TE::Shift(93), 94 => TE::Shift(94), 95 => TE::Shift(100), 96 => TE::Shift(101), 97 => TE::Shift(102), 99 => TE::Shift(108) },
    hashmap! { 9 => TE::Transit(47), 11 => TE::Transit(193), 12 => TE::Transit(237), 13 => TE::Transit(39), 14 => TE::Transit(40), 15 => TE::Transit(45), 16 => TE::Transit(43), 17 => TE::Transit(38), 19 => TE::Transit(44), 22 => TE::Transit(46), 25 => TE::Transit(41), 26 => TE::Transit(42), 28 => TE::Transit(37), 29 => TE::Transit(52), 30 => TE::Transit(48), 31 => TE::Transit(53), 33 => TE::Transit(51), 34 => TE::Transit(49), 35 => TE::Shift(21), 36 => TE::Reduce(109), 40 => TE::Shift(71), 41 => TE::Shift(72), 42 => TE::Shift(77), 44 => TE::Shift(75), 45 => TE::Shift(70), 47 => TE::Shift(76), 48 => TE::Shift(50), 50 => TE::Shift(73), 51 => TE::Shift(74), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 78 => TE::Shift(33), 80 => TE::Reduce(60), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 49 => TE::Reduce(48), 79 => TE::Reduce(48) },
    hashmap! { 9 => TE::Transit(47), 11 => TE::Transit(193), 12 => TE::Transit(239), 13 => TE::Transit(39), 14 => TE::Transit(40), 15 => TE::Transit(45), 16 => TE::Transit(43), 17 => TE::Transit(38), 19 => TE::Transit(44), 22 => TE::Transit(46), 25 => TE::Transit(41), 26 => TE::Transit(42), 28 => TE::Transit(37), 29 => TE::Transit(52), 30 => TE::Transit(48), 31 => TE::Transit(53), 33 => TE::Transit(51), 34 => TE::Transit(49), 35 => TE::Shift(21), 36 => TE::Reduce(109), 40 => TE::Shift(71), 41 => TE::Shift(72), 42 => TE::Shift(77), 44 => TE::Shift(75), 45 => TE::Shift(70), 47 => TE::Shift(76), 48 => TE::Shift(50), 50 => TE::Shift(73), 51 => TE::Shift(74), 52 => TE::Shift(54), 53 => TE::Shift(55), 54 => TE::Shift(56), 55 => TE::Shift(57), 56 => TE::Shift(59), 67 => TE::Shift(63), 68 => TE::Shift(64), 69 => TE::Shift(65), 70 => TE::Shift(66), 71 => TE::Shift(67), 72 => TE::Shift(68), 73 => TE::Shift(69), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 78 => TE::Shift(33), 80 => TE::Reduce(60), 81 => TE::Shift(60), 86 => TE::Shift(58), 89 => TE::Shift(61), 98 => TE::Shift(62) },
    hashmap! { 49 => TE::Reduce(49), 79 => TE::Reduce(49) },
    hashmap! { 33 => TE::Transit(241), 34 => TE::Transit(30), 35 => TE::Shift(21), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20) },
    hashmap! { 82 => TE::Reduce(16), 83 => TE::Reduce(16) },
    hashmap! { 82 => TE::Reduce(112), 83 => TE::Reduce(112) },
    hashmap! { 36 => TE::Shift(244), 86 => TE::Shift(25) },
    hashmap! { 81 => TE::Shift(245) },
    hashmap! { 7 => TE::Transit(246), 8 => TE::Transit(28), 33 => TE::Transit(29), 34 => TE::Transit(30), 35 => TE::Shift(21), 74 => TE::Shift(17), 75 => TE::Shift(18), 76 => TE::Shift(19), 77 => TE::Shift(20), 82 => TE::Reduce(15) },
    hashmap! { 82 => TE::Shift(247) },
    hashmap! { 9 => TE::Transit(248), 78 => TE::Shift(33) },
    hashmap! { 35 => TE::Reduce(12), 39 => TE::Reduce(12), 74 => TE::Reduce(12), 75 => TE::Reduce(12), 76 => TE::Reduce(12), 77 => TE::Reduce(12), 79 => TE::Reduce(12) },
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



use std::process;
use std::mem;
use std::ptr;
use std::default::Default as D;

use super::ast::*;
use super::types::*;
use super::loc::*;
use super::errors::*;
use super::print;

type Str = &'static str;

impl Parser {
  fn get_loc(&self) -> Loc {
    Loc(self.tokenizer.token_start_line, self.tokenizer.token_start_column + 1)
  }
}

impl Token {
  fn get_loc(&self) -> Loc {
    Loc(self.start_line, self.start_column + 1)
  }
}

fn gen_binary(l: Expr, opt: Token, r: Expr, op: Operator) -> Expr {
  Expr::new(opt.get_loc(),
            ExprData::Binary(Binary { op, l: Box::new(l), r: Box::new(r) }))
}

fn gen_unary(opt: Token, r: Expr, op: Operator) -> Expr {
  Expr::new(opt.get_loc(),
            ExprData::Unary(Unary { op, r: Box::new(r) }))
}

fn on_parse_error(parser: &Parser, token: &Token) {
  for error in &parser.errors { eprintln!("{}", error); }
  let loc = token.get_loc();
  eprintln!("*** Error at ({},{}): syntax error", loc.0, loc.1);
  process::exit(1);
}

fn on_lex_error(lex: &Tokenizer, slice: &str) {
  for error in lex.get_errors() { eprintln!("{}", error); }
  eprintln!("*** Error at ({},{}): unrecognized character '{}'", lex.current_line, lex.current_column + 1, slice);
  process::exit(1);
}

// Final result type returned from `parse` method call.
pub type TResult = Result<Program, Vec<Error>>;
// Error type
pub type TError = Error;

// some util types, only for convenience
type ClassList = Vec<ClassDef>;
type FieldList = Vec<FieldDef>;
type VarDefList = Vec<VarDef>;
type StmtList = Vec<Stmt>;
type ExprList = Vec<Expr>;
type GuardedList = Vec<(Expr, Block)>;
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

  handlers: [fn(&mut Tokenizer) -> &'static str; 98],
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

      handlers: [Tokenizer::_lex_rule0, Tokenizer::_lex_rule1, Tokenizer::_lex_rule2, Tokenizer::_lex_rule3, Tokenizer::_lex_rule4, Tokenizer::_lex_rule5, Tokenizer::_lex_rule6, Tokenizer::_lex_rule7, Tokenizer::_lex_rule8, Tokenizer::_lex_rule9, Tokenizer::_lex_rule10, Tokenizer::_lex_rule11, Tokenizer::_lex_rule12, Tokenizer::_lex_rule13, Tokenizer::_lex_rule14, Tokenizer::_lex_rule15, Tokenizer::_lex_rule16, Tokenizer::_lex_rule17, Tokenizer::_lex_rule18, Tokenizer::_lex_rule19, Tokenizer::_lex_rule20, Tokenizer::_lex_rule21, Tokenizer::_lex_rule22, Tokenizer::_lex_rule23, Tokenizer::_lex_rule24, Tokenizer::_lex_rule25, Tokenizer::_lex_rule26, Tokenizer::_lex_rule27, Tokenizer::_lex_rule28, Tokenizer::_lex_rule29, Tokenizer::_lex_rule30, Tokenizer::_lex_rule31, Tokenizer::_lex_rule32, Tokenizer::_lex_rule33, Tokenizer::_lex_rule34, Tokenizer::_lex_rule35, Tokenizer::_lex_rule36, Tokenizer::_lex_rule37, Tokenizer::_lex_rule38, Tokenizer::_lex_rule39, Tokenizer::_lex_rule40, Tokenizer::_lex_rule41, Tokenizer::_lex_rule42, Tokenizer::_lex_rule43, Tokenizer::_lex_rule44, Tokenizer::_lex_rule45, Tokenizer::_lex_rule46, Tokenizer::_lex_rule47, Tokenizer::_lex_rule48, Tokenizer::_lex_rule49, Tokenizer::_lex_rule50, Tokenizer::_lex_rule51, Tokenizer::_lex_rule52, Tokenizer::_lex_rule53, Tokenizer::_lex_rule54, Tokenizer::_lex_rule55, Tokenizer::_lex_rule56, Tokenizer::_lex_rule57, Tokenizer::_lex_rule58, Tokenizer::_lex_rule59, Tokenizer::_lex_rule60, Tokenizer::_lex_rule61, Tokenizer::_lex_rule62, Tokenizer::_lex_rule63, Tokenizer::_lex_rule64, Tokenizer::_lex_rule65, Tokenizer::_lex_rule66, Tokenizer::_lex_rule67, Tokenizer::_lex_rule68, Tokenizer::_lex_rule69, Tokenizer::_lex_rule70, Tokenizer::_lex_rule71, Tokenizer::_lex_rule72, Tokenizer::_lex_rule73, Tokenizer::_lex_rule74, Tokenizer::_lex_rule75, Tokenizer::_lex_rule76, Tokenizer::_lex_rule77, Tokenizer::_lex_rule78, Tokenizer::_lex_rule79, Tokenizer::_lex_rule80, Tokenizer::_lex_rule81, Tokenizer::_lex_rule82, Tokenizer::_lex_rule83, Tokenizer::_lex_rule84, Tokenizer::_lex_rule85, Tokenizer::_lex_rule86, Tokenizer::_lex_rule87, Tokenizer::_lex_rule88, Tokenizer::_lex_rule89, Tokenizer::_lex_rule90, Tokenizer::_lex_rule91, Tokenizer::_lex_rule92, Tokenizer::_lex_rule93, Tokenizer::_lex_rule94, Tokenizer::_lex_rule95, Tokenizer::_lex_rule96, Tokenizer::_lex_rule97],
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
      return self.to_token(EOF);
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
      }
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
    return "INC";
  }

  fn _lex_rule37(&mut self) -> &'static str {
    return "DEC";
  }

  fn _lex_rule38(&mut self) -> &'static str {
    return "SHL";
  }

  fn _lex_rule39(&mut self) -> &'static str {
    return "SHR";
  }

  fn _lex_rule40(&mut self) -> &'static str {
    return "'+'";
  }

  fn _lex_rule41(&mut self) -> &'static str {
    return "'-'";
  }

  fn _lex_rule42(&mut self) -> &'static str {
    return "'*'";
  }

  fn _lex_rule43(&mut self) -> &'static str {
    return "'/'";
  }

  fn _lex_rule44(&mut self) -> &'static str {
    return "'%'";
  }

  fn _lex_rule45(&mut self) -> &'static str {
    return "'&'";
  }

  fn _lex_rule46(&mut self) -> &'static str {
    return "'|'";
  }

  fn _lex_rule47(&mut self) -> &'static str {
    return "'^'";
  }

  fn _lex_rule48(&mut self) -> &'static str {
    return "'='";
  }

  fn _lex_rule49(&mut self) -> &'static str {
    return "'<'";
  }

  fn _lex_rule50(&mut self) -> &'static str {
    return "'>'";
  }

  fn _lex_rule51(&mut self) -> &'static str {
    return "'.'";
  }

  fn _lex_rule52(&mut self) -> &'static str {
    return "','";
  }

  fn _lex_rule53(&mut self) -> &'static str {
    return "';'";
  }

  fn _lex_rule54(&mut self) -> &'static str {
    return "'!'";
  }

  fn _lex_rule55(&mut self) -> &'static str {
    return "'('";
  }

  fn _lex_rule56(&mut self) -> &'static str {
    return "')'";
  }

  fn _lex_rule57(&mut self) -> &'static str {
    return "'['";
  }

  fn _lex_rule58(&mut self) -> &'static str {
    return "']'";
  }

  fn _lex_rule59(&mut self) -> &'static str {
    return "'{'";
  }

  fn _lex_rule60(&mut self) -> &'static str {
    return "'}'";
  }

  fn _lex_rule61(&mut self) -> &'static str {
    return "':'";
  }

  fn _lex_rule62(&mut self) -> &'static str {
    self.begin("S");
    self.string_builder.0.clear();
    self.string_builder.1 = self.token_start_line;
    self.string_builder.2 = self.token_start_column + 1;
    return "";
  }

  fn _lex_rule63(&mut self) -> &'static str {
    let loc = Loc(self.string_builder.1, self.string_builder.2);
    let string = print::quote(&self.string_builder.0.clone());
    self.report_error(Error::new(loc, NewlineInStr { string }));
    return "";
  }

  fn _lex_rule64(&mut self) -> &'static str {
    return "";
  }

  fn _lex_rule65(&mut self) -> &'static str {
    let loc = Loc(self.string_builder.1, self.string_builder.2);
    let string = print::quote(&self.string_builder.0.clone());
    self.report_error(Error::new(loc, UnterminatedStr { string }));
    self.begin("INITIAL");
    return "";
  }

  fn _lex_rule66(&mut self) -> &'static str {
    self.begin("INITIAL");
    return "STRING_CONST";
  }

  fn _lex_rule67(&mut self) -> &'static str {
    self.string_builder.0.push('\n');
    return "";
  }

  fn _lex_rule68(&mut self) -> &'static str {
    self.string_builder.0.push('\t');
    return "";
  }

  fn _lex_rule69(&mut self) -> &'static str {
    self.string_builder.0.push('"');
    return "";
  }

  fn _lex_rule70(&mut self) -> &'static str {
    self.string_builder.0.push('\\');
    return "";
  }

  fn _lex_rule71(&mut self) -> &'static str {
    self.string_builder.0.push_str(self.yytext);
    return "";
  }

  fn _lex_rule72(&mut self) -> &'static str {
    return "";
  }

  fn _lex_rule73(&mut self) -> &'static str {
    return "";
  }

  fn _lex_rule74(&mut self) -> &'static str {
    return "INT_CONST";
  }

  fn _lex_rule75(&mut self) -> &'static str {
    return "IDENTIFIER";
  }

  fn _lex_rule76(&mut self) -> &'static str {
    return "'{'";
  }

  fn _lex_rule77(&mut self) -> &'static str {
    return "'}'";
  }

  fn _lex_rule78(&mut self) -> &'static str {
    return "';'";
  }

  fn _lex_rule79(&mut self) -> &'static str {
    return "'('";
  }

  fn _lex_rule80(&mut self) -> &'static str {
    return "')'";
  }

  fn _lex_rule81(&mut self) -> &'static str {
    return "','";
  }

  fn _lex_rule82(&mut self) -> &'static str {
    return "':'";
  }

  fn _lex_rule83(&mut self) -> &'static str {
    return "'='";
  }

  fn _lex_rule84(&mut self) -> &'static str {
    return "'['";
  }

  fn _lex_rule85(&mut self) -> &'static str {
    return "']'";
  }

  fn _lex_rule86(&mut self) -> &'static str {
    return "'+'";
  }

  fn _lex_rule87(&mut self) -> &'static str {
    return "'-'";
  }

  fn _lex_rule88(&mut self) -> &'static str {
    return "'*'";
  }

  fn _lex_rule89(&mut self) -> &'static str {
    return "'/'";
  }

  fn _lex_rule90(&mut self) -> &'static str {
    return "'%'";
  }

  fn _lex_rule91(&mut self) -> &'static str {
    return "'<'";
  }

  fn _lex_rule92(&mut self) -> &'static str {
    return "'>'";
  }

  fn _lex_rule93(&mut self) -> &'static str {
    return "'&'";
  }

  fn _lex_rule94(&mut self) -> &'static str {
    return "'|'";
  }

  fn _lex_rule95(&mut self) -> &'static str {
    return "'^'";
  }

  fn _lex_rule96(&mut self) -> &'static str {
    return "'!'";
  }

  fn _lex_rule97(&mut self) -> &'static str {
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
  handlers: [fn(&mut Parser) -> SV; 119],
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

      handlers: [Parser::_handler0, Parser::_handler1, Parser::_handler2, Parser::_handler3, Parser::_handler4, Parser::_handler5, Parser::_handler6, Parser::_handler7, Parser::_handler8, Parser::_handler9, Parser::_handler10, Parser::_handler11, Parser::_handler12, Parser::_handler13, Parser::_handler14, Parser::_handler15, Parser::_handler16, Parser::_handler17, Parser::_handler18, Parser::_handler19, Parser::_handler20, Parser::_handler21, Parser::_handler22, Parser::_handler23, Parser::_handler24, Parser::_handler25, Parser::_handler26, Parser::_handler27, Parser::_handler28, Parser::_handler29, Parser::_handler30, Parser::_handler31, Parser::_handler32, Parser::_handler33, Parser::_handler34, Parser::_handler35, Parser::_handler36, Parser::_handler37, Parser::_handler38, Parser::_handler39, Parser::_handler40, Parser::_handler41, Parser::_handler42, Parser::_handler43, Parser::_handler44, Parser::_handler45, Parser::_handler46, Parser::_handler47, Parser::_handler48, Parser::_handler49, Parser::_handler50, Parser::_handler51, Parser::_handler52, Parser::_handler53, Parser::_handler54, Parser::_handler55, Parser::_handler56, Parser::_handler57, Parser::_handler58, Parser::_handler59, Parser::_handler60, Parser::_handler61, Parser::_handler62, Parser::_handler63, Parser::_handler64, Parser::_handler65, Parser::_handler66, Parser::_handler67, Parser::_handler68, Parser::_handler69, Parser::_handler70, Parser::_handler71, Parser::_handler72, Parser::_handler73, Parser::_handler74, Parser::_handler75, Parser::_handler76, Parser::_handler77, Parser::_handler78, Parser::_handler79, Parser::_handler80, Parser::_handler81, Parser::_handler82, Parser::_handler83, Parser::_handler84, Parser::_handler85, Parser::_handler86, Parser::_handler87, Parser::_handler88, Parser::_handler89, Parser::_handler90, Parser::_handler91, Parser::_handler92, Parser::_handler93, Parser::_handler94, Parser::_handler95, Parser::_handler96, Parser::_handler97, Parser::_handler98, Parser::_handler99, Parser::_handler100, Parser::_handler101, Parser::_handler102, Parser::_handler103, Parser::_handler104, Parser::_handler105, Parser::_handler106, Parser::_handler107, Parser::_handler108, Parser::_handler109, Parser::_handler110, Parser::_handler111, Parser::_handler112, Parser::_handler113, Parser::_handler114, Parser::_handler115, Parser::_handler116, Parser::_handler117, Parser::_handler118],
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
        }

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
        }

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
        }

        _ => unreachable!(),
      }
    }

    unreachable!();
  }

  fn unexpected_token(&mut self, token: &Token) {
    on_parse_error(self, &token);
  }

  fn _handler0(&mut self) -> SV {
    let mut _1 = self.values_stack.pop().unwrap();
    let _0 = _1;
    _0
  }

  fn _handler1(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _1);
    let _0 = if self.errors.is_empty() {
      Ok(Program { class: _1, ..D::default() })
    } else {
      Err(mem::replace(&mut self.errors, Vec::new()))
    };
    SV::_2(_0)
  }

  fn _handler2(&mut self) -> SV {
    let mut _2 = pop!(self.values_stack, _3);
    let mut _1 = pop!(self.values_stack, _1);
    _1.push(_2);
    let _0 = _1;
    SV::_1(_0)
  }

  fn _handler3(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _3);
    let _0 = vec![_1];
    SV::_1(_0)
  }

  fn _handler4(&mut self) -> SV {
    self.values_stack.pop();
    let mut _6 = pop!(self.values_stack, _6);
    self.values_stack.pop();
    let mut _4 = pop!(self.values_stack, _5);
    let mut _3 = pop!(self.values_stack, _0);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _4);
    let _0 = ClassDef {
      loc: _2.get_loc(),
      name: _3.value,
      parent: _4,
      field: _6,
      sealed: _1,
      ..D::default()
    };
    SV::_3(_0)
  }

  fn _handler5(&mut self) -> SV {
    self.values_stack.pop();
    let _0 = true;
    SV::_4(_0)
  }

  fn _handler6(&mut self) -> SV {
    let _0 = false;
    SV::_4(_0)
  }

  fn _handler7(&mut self) -> SV {
    let mut _2 = pop!(self.values_stack, _0);
    self.values_stack.pop();
    let _0 = Some(_2.value);
    SV::_5(_0)
  }

  fn _handler8(&mut self) -> SV {
    let _0 = None;
    SV::_5(_0)
  }

  fn _handler9(&mut self) -> SV {
    self.values_stack.pop();
    let mut _2 = pop!(self.values_stack, _7);
    let mut _1 = pop!(self.values_stack, _6);
    _1.push(FieldDef::VarDef(_2));
    let _0 = _1;
    SV::_6(_0)
  }

  fn _handler10(&mut self) -> SV {
    let mut _2 = pop!(self.values_stack, _8);
    let mut _1 = pop!(self.values_stack, _6);
    _1.push(FieldDef::MethodDef(_2));
    let _0 = _1;
    SV::_6(_0)
  }

  fn _handler11(&mut self) -> SV {
    let _0 = Vec::new();
    SV::_6(_0)
  }

  fn _handler12(&mut self) -> SV {
    let mut _7 = pop!(self.values_stack, _11);
    self.values_stack.pop();
    let mut _5 = pop!(self.values_stack, _10);
    self.values_stack.pop();
    let mut _3 = pop!(self.values_stack, _0);
    let mut _2 = pop!(self.values_stack, _9);
    self.values_stack.pop();
    let _0 = MethodDef {
      loc: _3.get_loc(),
      name: _3.value,
      ret_t: _2,
      param: _5,
      static_: true,
      body: _7,
      scope: D::default(),
      class: ptr::null(),
      offset: -1,
    };
    SV::_8(_0)
  }

  fn _handler13(&mut self) -> SV {
    let mut _6 = pop!(self.values_stack, _11);
    self.values_stack.pop();
    let mut _4 = pop!(self.values_stack, _10);
    self.values_stack.pop();
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _9);
    let _0 = MethodDef {
      loc: _2.get_loc(),
      name: _2.value,
      ret_t: _1,
      param: _4,
      static_: false,
      body: _6,
      scope: D::default(),
      class: ptr::null(),
      offset: -1,
    };
    SV::_8(_0)
  }

  fn _handler14(&mut self) -> SV {
    let mut _1 = self.values_stack.pop().unwrap();
    let _0 = _1;
    _0
  }

  fn _handler15(&mut self) -> SV {
    let _0 = Vec::new();
    SV::_10(_0)
  }

  fn _handler16(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _7);
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _10);
    _1.push(_3);
    let _0 = _1;
    SV::_10(_0)
  }

  fn _handler17(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _7);
    let _0 = vec![_1];
    SV::_10(_0)
  }

  fn _handler18(&mut self) -> SV {
    self.values_stack.pop();
    let mut _2 = pop!(self.values_stack, _12);
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Block {
      loc: _1.get_loc(),
      stmt: _2,
      ..D::default()
    };
    SV::_11(_0)
  }

  fn _handler19(&mut self) -> SV {
    let mut _2 = pop!(self.values_stack, _13);
    let mut _1 = pop!(self.values_stack, _12);
    _1.push(_2);
    let _0 = _1;
    SV::_12(_0)
  }

  fn _handler20(&mut self) -> SV {
    let _0 = Vec::new();
    SV::_12(_0)
  }

  fn _handler21(&mut self) -> SV {
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _14);
    let _0 = Stmt::Simple(_1);
    SV::_13(_0)
  }

  fn _handler22(&mut self) -> SV {
    let mut _1 = self.values_stack.pop().unwrap();
    let _0 = _1;
    _0
  }

  fn _handler23(&mut self) -> SV {
    let mut _1 = self.values_stack.pop().unwrap();
    let _0 = _1;
    _0
  }

  fn _handler24(&mut self) -> SV {
    let mut _1 = self.values_stack.pop().unwrap();
    let _0 = _1;
    _0
  }

  fn _handler25(&mut self) -> SV {
    self.values_stack.pop();
    let mut _1 = self.values_stack.pop().unwrap();
    let _0 = _1;
    _0
  }

  fn _handler26(&mut self) -> SV {
    self.values_stack.pop();
    let mut _1 = self.values_stack.pop().unwrap();
    let _0 = _1;
    _0
  }

  fn _handler27(&mut self) -> SV {
    self.values_stack.pop();
    let mut _1 = self.values_stack.pop().unwrap();
    let _0 = _1;
    _0
  }

  fn _handler28(&mut self) -> SV {
    self.values_stack.pop();
    let mut _1 = self.values_stack.pop().unwrap();
    let _0 = _1;
    _0
  }

  fn _handler29(&mut self) -> SV {
    let mut _1 = self.values_stack.pop().unwrap();
    let _0 = _1;
    _0
  }

  fn _handler30(&mut self) -> SV {
    let mut _1 = self.values_stack.pop().unwrap();
    let _0 = _1;
    _0
  }

  fn _handler31(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _11);
    let _0 = Stmt::Block(_1);
    SV::_13(_0)
  }

  fn _handler32(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _13);
    let _0 = match _1 {
      Stmt::Block(block) => block,
      stmt => Block {
        loc: NO_LOC,
        stmt: vec![stmt],
        ..D::default()
      }
    };
    SV::_11(_0)
  }

  fn _handler33(&mut self) -> SV {
    let mut _5 = pop!(self.values_stack, _11);
    self.values_stack.pop();
    let mut _3 = pop!(self.values_stack, _15);
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Stmt::While(While {
      loc: _1.get_loc(),
      cond: _3,
      body: _5,
    });
    SV::_13(_0)
  }

  fn _handler34(&mut self) -> SV {
    let mut _9 = pop!(self.values_stack, _11);
    self.values_stack.pop();
    let mut _7 = pop!(self.values_stack, _14);
    self.values_stack.pop();
    let mut _5 = pop!(self.values_stack, _15);
    self.values_stack.pop();
    let mut _3 = pop!(self.values_stack, _14);
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Stmt::For(For {
      loc: _1.get_loc(),
      init: _3,
      cond: _5,
      update: _7,
      body: _9,
    });
    SV::_13(_0)
  }

  fn _handler35(&mut self) -> SV {
    let mut _9 = pop!(self.values_stack, _11);
    self.values_stack.pop();
    let mut _7 = pop!(self.values_stack, _16);
    let mut _6 = pop!(self.values_stack, _15);
    self.values_stack.pop();
    let mut _4 = pop!(self.values_stack, _0);
    let mut _3 = pop!(self.values_stack, _9);
    self.values_stack.pop();
    self.values_stack.pop();
    let _0 = Stmt::Foreach(Foreach {
      def: VarDef {
        loc: _4.get_loc(),
        name: _4.value,
        type_: _3,
        finish_loc: _4.get_loc(),
        src: None,
        scope: ptr::null(),
        index: D::default(),
        offset: -1,
      },
      arr: _6,
      cond: _7,
      body: _9,
    });
    SV::_13(_0)
  }

  fn _handler36(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Stmt::Break(Break { loc: _1.get_loc() });
    SV::_13(_0)
  }

  fn _handler37(&mut self) -> SV {
    let mut _6 = pop!(self.values_stack, _17);
    let mut _5 = pop!(self.values_stack, _11);
    self.values_stack.pop();
    let mut _3 = pop!(self.values_stack, _15);
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Stmt::If(If {
      loc: _1.get_loc(),
      cond: _3,
      on_true: _5,
      on_false: _6,
    });
    SV::_13(_0)
  }

  fn _handler38(&mut self) -> SV {
    let mut _2 = pop!(self.values_stack, _11);
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Some(_2);
    SV::_17(_0)
  }

  fn _handler39(&mut self) -> SV {
    let _0 = None;
    SV::_17(_0)
  }

  fn _handler40(&mut self) -> SV {
    self.values_stack.pop();
    let mut _5 = pop!(self.values_stack, _15);
    self.values_stack.pop();
    let mut _3 = pop!(self.values_stack, _0);
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Stmt::SCopy(SCopy {
      loc: _1.get_loc(),
      dst_loc: _3.get_loc(),
      dst: _3.value,
      dst_sym: ptr::null(),
      src: _5,
    });
    SV::_13(_0)
  }

  fn _handler41(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Type { loc: _1.get_loc(), sem: VAR };
    SV::_9(_0)
  }

  fn _handler42(&mut self) -> SV {
    let mut _1 = self.values_stack.pop().unwrap();
    let _0 = _1;
    _0
  }

  fn _handler43(&mut self) -> SV {
    let mut _2 = pop!(self.values_stack, _15);
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Some(_2);
    SV::_16(_0)
  }

  fn _handler44(&mut self) -> SV {
    let _0 = None;
    SV::_16(_0)
  }

  fn _handler45(&mut self) -> SV {
    self.values_stack.pop();
    let mut _3 = pop!(self.values_stack, _18);
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Stmt::Guarded(Guarded {
      loc: _1.get_loc(),
      guarded: _3,
    });
    SV::_13(_0)
  }

  fn _handler46(&mut self) -> SV {
    let mut _1 = self.values_stack.pop().unwrap();
    let _0 = _1;
    _0
  }

  fn _handler47(&mut self) -> SV {
    let _0 = Vec::new();
    SV::_18(_0)
  }

  fn _handler48(&mut self) -> SV {
    let mut _5 = pop!(self.values_stack, _11);
    self.values_stack.pop();
    let mut _3 = pop!(self.values_stack, _15);
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _18);
    _1.push((_3, _5));
    let _0 = _1;
    SV::_18(_0)
  }

  fn _handler49(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _11);
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = vec![(_1, _3)];
    SV::_18(_0)
  }

  fn _handler50(&mut self) -> SV {
    let mut _2 = pop!(self.values_stack, _15);
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Stmt::Return(Return {
      loc: _1.get_loc(),
      expr: Some(_2),
    });
    SV::_13(_0)
  }

  fn _handler51(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Stmt::Return(Return {
      loc: _1.get_loc(),
      expr: None,
    });
    SV::_13(_0)
  }

  fn _handler52(&mut self) -> SV {
    self.values_stack.pop();
    let mut _3 = pop!(self.values_stack, _19);
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Stmt::Print(Print {
      loc: _1.get_loc(),
      print: _3,
    });
    SV::_13(_0)
  }

  fn _handler53(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _19);
    _1.push(_3);
    let _0 = _1;
    SV::_19(_0)
  }

  fn _handler54(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = vec![_1];
    SV::_19(_0)
  }

  fn _handler55(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = Simple::Assign(Assign {
      loc: _2.get_loc(),
      dst: _1,
      src: _3,
    });
    SV::_14(_0)
  }

  fn _handler56(&mut self) -> SV {
    let mut _4 = pop!(self.values_stack, _15);
    let mut _3 = pop!(self.values_stack, _0);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _9);
    let _0 = Simple::VarDef(VarDef {
      loc: _2.get_loc(),
      name: _2.value,
      type_: _1,
      finish_loc: self.get_loc(),
      src: Some(_4),
      scope: ptr::null(),
      index: D::default(),
      offset: -1,
    });
    SV::_14(_0)
  }

  fn _handler57(&mut self) -> SV {
    let mut _4 = pop!(self.values_stack, _15);
    let mut _3 = pop!(self.values_stack, _0);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Simple::VarDef(VarDef {
      loc: _2.get_loc(),
      name: _2.value,
      type_: Type { loc: _1.get_loc(), sem: VAR },
      finish_loc: self.get_loc(),
      src: Some(_4),
      scope: ptr::null(),
      index: D::default(),
      offset: -1,
    });
    SV::_14(_0)
  }

  fn _handler58(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _7);
    let _0 = Simple::VarDef(_1);
    SV::_14(_0)
  }

  fn _handler59(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = Simple::Expr(_1);
    SV::_14(_0)
  }

  fn _handler60(&mut self) -> SV {
    let _0 = Simple::Skip;
    SV::_14(_0)
  }

  fn _handler61(&mut self) -> SV {
    let mut _1 = self.values_stack.pop().unwrap();
    let _0 = _1;
    _0
  }

  fn _handler62(&mut self) -> SV {
    self.values_stack.pop();
    let mut _4 = pop!(self.values_stack, _19);
    self.values_stack.pop();
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _16);
    let _0 = Expr::new(_2.get_loc(), ExprData::Call(Call {
      owner: _1.map(|s| Box::new(s)),
      name: _2.value,
      arg: _4,
      is_arr_len: false,
      method: ptr::null(),
    }));
    SV::_15(_0)
  }

  fn _handler63(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Expr::with_type(_1.get_loc(), INT, ExprData::IntConst(_1.value.parse::<i32>().unwrap_or_else(|_| {
      self.errors.push(Error::new(_1.get_loc(), IntTooLarge { string: _1.value.to_string() }));
      0
    })));
    SV::_15(_0)
  }

  fn _handler64(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Expr::with_type(_1.get_loc(), BOOL, ExprData::BoolConst(true));
    SV::_15(_0)
  }

  fn _handler65(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Expr::with_type(_1.get_loc(), BOOL, ExprData::BoolConst(false));
    SV::_15(_0)
  }

  fn _handler66(&mut self) -> SV {
    self.values_stack.pop();
    let _0 = Expr::with_type(Loc(self.tokenizer.string_builder.1, self.tokenizer.string_builder.2),
                             STRING, ExprData::StringConst(self.tokenizer.string_builder.0.clone()));
    SV::_15(_0)
  }

  fn _handler67(&mut self) -> SV {
    self.values_stack.pop();
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _19);
    let _0 = Expr::new(self.get_loc(), ExprData::ArrayConst(_1));
    SV::_15(_0)
  }

  fn _handler68(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Expr::new(_1.get_loc(), ExprData::Null);
    SV::_15(_0)
  }

  fn _handler69(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::Add);
    SV::_15(_0)
  }

  fn _handler70(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::Sub);
    SV::_15(_0)
  }

  fn _handler71(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::Mul);
    SV::_15(_0)
  }

  fn _handler72(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::Div);
    SV::_15(_0)
  }

  fn _handler73(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::Mod);
    SV::_15(_0)
  }

  fn _handler74(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::Eq);
    SV::_15(_0)
  }

  fn _handler75(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::Ne);
    SV::_15(_0)
  }

  fn _handler76(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::Lt);
    SV::_15(_0)
  }

  fn _handler77(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::Gt);
    SV::_15(_0)
  }

  fn _handler78(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::Le);
    SV::_15(_0)
  }

  fn _handler79(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::Ge);
    SV::_15(_0)
  }

  fn _handler80(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::And);
    SV::_15(_0)
  }

  fn _handler81(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::Or);
    SV::_15(_0)
  }

  fn _handler82(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::Repeat);
    SV::_15(_0)
  }

  fn _handler83(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::BAnd);
    SV::_15(_0)
  }

  fn _handler84(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::BOr);
    SV::_15(_0)
  }

  fn _handler85(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::BXor);
    SV::_15(_0)
  }

  fn _handler86(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::Shl);
    SV::_15(_0)
  }

  fn _handler87(&mut self) -> SV {
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_binary(_1, _2, _3, Operator::Shr);
    SV::_15(_0)
  }

  fn _handler88(&mut self) -> SV {
    self.values_stack.pop();
    let mut _5 = pop!(self.values_stack, _15);
    self.values_stack.pop();
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = Expr::new(_2.get_loc(),
                       ExprData::Range(Range { arr: Box::new(_1), lb: Box::new(_3), ub: Box::new(_5) }));
    SV::_15(_0)
  }

  fn _handler89(&mut self) -> SV {
    let mut _6 = pop!(self.values_stack, _15);
    self.values_stack.pop();
    self.values_stack.pop();
    let mut _3 = pop!(self.values_stack, _15);
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = Expr::new(_2.get_loc(),
                       ExprData::Default(Default { arr: Box::new(_1), idx: Box::new(_3), dft: Box::new(_6) }));
    SV::_15(_0)
  }

  fn _handler90(&mut self) -> SV {
    self.values_stack.pop();
    let mut _6 = pop!(self.values_stack, _15);
    self.values_stack.pop();
    let mut _4 = pop!(self.values_stack, _0);
    self.values_stack.pop();
    let mut _2 = pop!(self.values_stack, _15);
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Expr::new(_1.get_loc(), ExprData::Comprehension(Comprehension {
      expr: Box::new(_2),
      name: _4.value,
      arr: Box::new(_6),
      cond: None,
    }));
    SV::_15(_0)
  }

  fn _handler91(&mut self) -> SV {
    self.values_stack.pop();
    let mut _8 = pop!(self.values_stack, _15);
    self.values_stack.pop();
    let mut _6 = pop!(self.values_stack, _15);
    self.values_stack.pop();
    let mut _4 = pop!(self.values_stack, _0);
    self.values_stack.pop();
    let mut _2 = pop!(self.values_stack, _15);
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Expr::new(_1.get_loc(), ExprData::Comprehension(Comprehension {
      expr: Box::new(_2),
      name: _4.value,
      arr: Box::new(_6),
      cond: Some(Box::new(_8)),
    }));
    SV::_15(_0)
  }

  fn _handler92(&mut self) -> SV {
    self.values_stack.pop();
    let mut _2 = self.values_stack.pop().unwrap();
    self.values_stack.pop();
    let _0 = _2;
    _0
  }

  fn _handler93(&mut self) -> SV {
    let mut _2 = pop!(self.values_stack, _15);
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = gen_unary(_1, _2, Operator::Neg);
    SV::_15(_0)
  }

  fn _handler94(&mut self) -> SV {
    let mut _2 = pop!(self.values_stack, _15);
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = gen_unary(_1, _2, Operator::Not);
    SV::_15(_0)
  }

  fn _handler95(&mut self) -> SV {
    let mut _2 = pop!(self.values_stack, _15);
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = gen_unary(_1, _2, Operator::PreInc);
    SV::_15(_0)
  }

  fn _handler96(&mut self) -> SV {
    let mut _2 = pop!(self.values_stack, _15);
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = gen_unary(_1, _2, Operator::PreDec);
    SV::_15(_0)
  }

  fn _handler97(&mut self) -> SV {
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_unary(_2, _1, Operator::PostInc);
    SV::_15(_0)
  }

  fn _handler98(&mut self) -> SV {
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = gen_unary(_2, _1, Operator::PostDec);
    SV::_15(_0)
  }

  fn _handler99(&mut self) -> SV {
    self.values_stack.pop();
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Expr::with_type(_1.get_loc(), INT, ExprData::ReadInt);
    SV::_15(_0)
  }

  fn _handler100(&mut self) -> SV {
    self.values_stack.pop();
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Expr::with_type(_1.get_loc(), STRING, ExprData::ReadLine);
    SV::_15(_0)
  }

  fn _handler101(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Expr::new(_1.get_loc(), ExprData::This);
    SV::_15(_0)
  }

  fn _handler102(&mut self) -> SV {
    self.values_stack.pop();
    self.values_stack.pop();
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Expr::new(_1.get_loc(), ExprData::NewClass { name: _2.value });
    SV::_15(_0)
  }

  fn _handler103(&mut self) -> SV {
    self.values_stack.pop();
    let mut _4 = pop!(self.values_stack, _15);
    self.values_stack.pop();
    let mut _2 = pop!(self.values_stack, _9);
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Expr::new(_1.get_loc(), ExprData::NewArray { elem_t: _2, len: Box::new(_4) });
    SV::_15(_0)
  }

  fn _handler104(&mut self) -> SV {
    self.values_stack.pop();
    let mut _5 = pop!(self.values_stack, _0);
    self.values_stack.pop();
    let mut _3 = pop!(self.values_stack, _15);
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Expr::new(_1.get_loc(), ExprData::TypeTest { expr: Box::new(_3), name: _5.value });
    SV::_15(_0)
  }

  fn _handler105(&mut self) -> SV {
    let mut _5 = pop!(self.values_stack, _15);
    self.values_stack.pop();
    let mut _3 = pop!(self.values_stack, _0);
    self.values_stack.pop();
    self.values_stack.pop();
    let _0 = Expr::new(_5.loc, ExprData::TypeCast { name: _3.value, expr: Box::new(_5) });
    SV::_15(_0)
  }

  fn _handler106(&mut self) -> SV {
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _16);
    let _0 = Expr::new(_2.get_loc(), ExprData::Id(Id {
      owner: _1.map(|e| Box::new(e)),
      name: _2.value,
      symbol: ptr::null(),
      for_assign: D::default(),
    }));
    SV::_15(_0)
  }

  fn _handler107(&mut self) -> SV {
    self.values_stack.pop();
    let mut _3 = pop!(self.values_stack, _15);
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = Expr::new(_1.loc, ExprData::Indexed(Indexed {
      arr: Box::new(_1),
      idx: Box::new(_3),
      for_assign: D::default(),
    }));
    SV::_15(_0)
  }

  fn _handler108(&mut self) -> SV {
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _15);
    let _0 = Some(_1);
    SV::_16(_0)
  }

  fn _handler109(&mut self) -> SV {
    let _0 = None;
    SV::_16(_0)
  }

  fn _handler110(&mut self) -> SV {
    let mut _1 = self.values_stack.pop().unwrap();
    let _0 = _1;
    _0
  }

  fn _handler111(&mut self) -> SV {
    let _0 = Vec::new();
    SV::_19(_0)
  }

  fn _handler112(&mut self) -> SV {
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _9);
    let _0 = VarDef {
      loc: _2.get_loc(),
      name: _2.value,
      type_: _1,
      finish_loc: self.get_loc(),
      src: None,
      scope: ptr::null(),
      index: D::default(),
      offset: -1,
    };
    SV::_7(_0)
  }

  fn _handler113(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Type { loc: _1.get_loc(), sem: INT };
    SV::_9(_0)
  }

  fn _handler114(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Type { loc: _1.get_loc(), sem: VOID };
    SV::_9(_0)
  }

  fn _handler115(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Type { loc: _1.get_loc(), sem: BOOL };
    SV::_9(_0)
  }

  fn _handler116(&mut self) -> SV {
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Type { loc: _1.get_loc(), sem: STRING };
    SV::_9(_0)
  }

  fn _handler117(&mut self) -> SV {
    let mut _2 = pop!(self.values_stack, _0);
    let mut _1 = pop!(self.values_stack, _0);
    let _0 = Type { loc: _2.get_loc(), sem: SemanticType::Named(_2.value) };
    SV::_9(_0)
  }

  fn _handler118(&mut self) -> SV {
    self.values_stack.pop();
    self.values_stack.pop();
    let mut _1 = pop!(self.values_stack, _9);
    let _0 = Type { loc: _1.loc, sem: SemanticType::Array(Box::new(_1.sem)) };
    SV::_9(_0)
  }
}
