// syntax-cli -g parser/parser.rs.g -m lalr1 -o parser/src/lib.rs

%lex

%%

// keywords
"void"              return "VOID";
"int"               return "INT";
"bool"              return "BOOL";
"string"            return "STRING";    
"new"               return "NEW";
"null"              return "NULL";
"true"              return "TRUE";
"false"             return "FALSE";
"class"             return "CLASS";
"extends"           return "EXTENDS";
"this"              return "THIS";
"while"             return "WHILE";
"foreach"           return "FOREACH";
"for"               return "FOR";
"if"                return "IF";        
"else"              return "ELSE";
"return"            return "RETURN";    
"break"             return "BREAK";
"Print"             return "PRINT";
"ReadInteger"       return "READ_INTEGER";
"ReadLine"          return "READ_LINE";
"static"            return "STATIC";
"instanceof"        return "INSTANCEOF";
"scopy"             return "SCOPY";
"sealed"            return "SEALED";
"var"               return "VAR";
"default"           return "DEFAULT";
"in"                return "IN";

// operators
"|||"               return "GUARD_SPLIT";
"<="                return "LESS_EQUAL";
">="                return "GREATER_EQUAL";
"=="                return "EQUAL";
"!="                return "NOT_EQUAL";
"&&"                return "AND";
"||"                return "OR";
"%%"                return "REPEAT";
"++"                return "CONCAT";

// simple operators
"+"                 return "'+'";
"-"                 return "'-'";
"*"                 return "'*'";
"%"                 return "'%'";
"="                 return "'='";
"<"                 return "'<'";
">"                 return "'>'";
"."                 return "'.'";
","                 return "','";
";"                 return "';'";
"!"                 return "'!'";
"("                 return "'('";
")"                 return "')'";
"["                 return "'['";
"]"                 return "']'";
"{"                 return "'{'";
"}"                 return "'}'";
":"                 return "':'";

\s+                 return "";

\d+                 return "INT_CONST";

[A-Za-z][_0-9A-Za-z]* return "IDENTIFIER";

/lex

%left OR
%left AND
%nonassoc EQUAL NOT_EQUAL
%nonassoc LESS_EQUAL GREATER_EQUAL '<' '>'
%right CONCAT
%left REPEAT
%left  '+' '-'
%left  '*' '%'
%nonassoc UMINUS '!'
%nonassoc '[' '.' DEFAULT
%nonassoc ')' EMPTY
%nonassoc ELSE

%{

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

%}

%%

Program
    : ClassList {
        |$1: Sem| -> Program;
        $$ = Program {
            classes: get_move!($1, ClassList),
        };
    }
    ;

ClassList
    : ClassList ClassDef {
        |$1: Sem, $2: Sem| -> Sem;
        let mut ret = $1;
        get_ref!(ret, ClassList).push(get_move!($2, ClassDef));
        $$ = ret;
    }
    | ClassDef {
        |$1: Sem| -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::ClassList(vec![get_move!($1, ClassDef)]),
        }
    }
    ;

ClassDef
    : MaybeSealed CLASS Identifier MaybeExtends  '{' FieldList '}' {
        |$1: Sem, $2: Token, $3: Sem, $4: Sem, $6: Sem| -> Sem;
        $$ = Sem {
            loc: $2.get_loc(),
            value: SemValue::ClassDef(ClassDef {
                loc: $2.get_loc(),
                name: get_move!($3, Identifier),
                parent: match $4.value {
                    SemValue::Identifier(name) => Some(name),
                    SemValue::None => None,
                    _ => unreachable!(),
                },
                fields: get_move!($6, FieldList),
                sealed: get_move!($1, Sealed),
            })
        }
    }
    ;

MaybeSealed
    : SEALED {
        || -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::Sealed(true),
        };
    }
    | /* empty */ {
        || -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::Sealed(false),
        };
    }
    ;

MaybeExtends
    : EXTENDS Identifier {
        |$2: Sem| -> Sem;
        $$ = Sem {
            loc: $2.loc,
            value: SemValue::Identifier(get_move!($2, Identifier)),
        };
    }
    | /* empty */ {
        || -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::None,
        };
    }
    ;

FieldList
    : FieldList VarDef {
        |$1: Sem, $2: Sem| -> Sem;
        let mut ret = $1;
        get_ref!(ret, FieldList).push(FieldDef::VarDef(get_move!($2, VarDef)));
        $$ = ret;
    }
    | FieldList MethodDef {
        |$1: Sem, $2: Sem| -> Sem;
        let mut ret = $1;
        get_ref!(ret, FieldList).push(FieldDef::MethodDef(get_move!($2, MethodDef)));
        $$ = ret;
    }
    | /* empty */ {
        || -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::FieldList(Vec::new())
        };
    }
    ;

// I don't know why use 'MaybeStatic -> eps | STATIC' will cause shift-reduce conflict
MethodDef
    : STATIC Type Identifier '(' VariableListOrEmpty ')' Block {
        |$1:Token, $2: Sem, $3: Sem, $5: Sem, $7: Sem| -> Sem;
        $$ = Sem {
            loc: $3.loc,
            value: SemValue::MethodDef(MethodDef {
                loc: $3.loc,
                name: get_move!($3, Identifier),
                return_type: get_move!($2, Type),
                parameters: get_move!($5, VarDefList),
                static_: true,
                body: get_move!($7, Block),
            })
        };
    }
    | Type Identifier '(' VariableListOrEmpty ')' Block {
        |$1: Sem, $2: Sem, $4: Sem, $6: Sem| -> Sem;
        $$ = Sem {
            loc: $2.loc,
            value: SemValue::MethodDef(MethodDef {
                loc: $2.loc,
                name: get_move!($2, Identifier),
                return_type: get_move!($1, Type),
                parameters: get_move!($4, VarDefList),
                static_: false,
                body: get_move!($6, Block),
            })
        };
    }
    ;

VariableListOrEmpty
    : VariableList {
        |$1: Sem| -> Sem;
        $$ = $1;
    }
    | /* empty */ {
        || -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::VarDefList(Vec::new()),
        };
    }
    ;

VariableList
    : VariableList ',' Variable {
        |$1: Sem, $3: Sem| -> Sem;
        let mut ret = $1;
        get_ref!(ret, VarDefList).push(get_move!($3, VarDef));
        $$ = ret;
    }
    | Variable {
        |$1: Sem| -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::VarDefList(vec!(get_move!($1, VarDef))),
        };
    }
    ;

Block
    : '{' StatementList '}' {
        |$1: Token, $2: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Block(Block {
                loc: $1.get_loc(),
                statements: get_move!($2, StatementList),
            }),
        };
    }
    ;

StatementList
    : StatementList Statement {
        |$1: Sem, $2: Sem| -> Sem;
        let mut ret = $1;
        get_ref!(ret, StatementList).push(get_move!($2, Statement));
        $$ = ret;
    }
    | /* empty */ {
        || -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::StatementList(Vec::new()),
        };
    }
    ;

Statement
    : VarDef {
        |$1: Sem| -> Sem;
        $$ = Sem {
            loc: $1.loc,
            value: SemValue::Statement(Statement::VarDef(get_move!($1, VarDef))),
        };
    }
    | Simple ';' {
        |$1: Sem| -> Sem;
        $$ = $1;
    }
    | If {
        |$1: Sem| -> Sem;
        $$ = $1;
    }
    | While {
        |$1: Sem| -> Sem;
        $$ = $1;
    }
    | For {
        |$1: Sem| -> Sem;
        $$ = $1;
    }
    | Return ';' {
        |$1: Sem| -> Sem;
        $$ = $1;
    }
    | Print ';' {
        |$1: Sem| -> Sem;
        $$ = $1;
    }
    | Break ';' {
        |$1: Sem| -> Sem;
        $$ = $1;
    }
    | ObjectCopy ';' {
        |$1: Sem| -> Sem;
        $$ = $1;
    }
    | Foreach {
        |$1: Sem| -> Sem;
        $$ = $1;
    }
    | Guarded {
        |$1: Sem| -> Sem;
        $$ = $1;
    }
    | Block {
        |$1: Sem| -> Sem;
        $$ = Sem {
            loc: $1.loc,
            value: SemValue::Statement(Statement::Block(get_move!($1, Block))),
        };
    }
    ;

While
    : WHILE '(' Expr ')' Statement {
        |$1: Token, $3: Sem, $5: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Statement(Statement::While(While {
                loc: $1.get_loc(),
                cond: get_move!($3, Expr),
                body: Box::new(get_move!($5, Statement)),
            })),
        };
    }
    ;

For
    : FOR '(' Simple ';' Expr ';' Simple ')' Statement {
        |$1: Token, $3: Sem, $5: Sem, $7: Sem, $9: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Statement(Statement::For(For {
                loc: $1.get_loc(),
                init: match get_move!($3, Statement) {
                    Statement::Simple(simple) => simple,
                    _ => unreachable!(),
                },
                cond: get_move!($5, Expr),
                update: match get_move!($7, Statement) {
                    Statement::Simple(simple) => simple,
                    _ => unreachable!(),
                },
                body: Box::new(get_move!($9, Statement)),
            })),
        };
    }
    ;

Break
    : BREAK {
        |$1: Token| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Statement(Statement::Break(Break {
                loc: $1.get_loc(),
            })),
        };
    }
    ;

If
    : IF '(' Expr ')' Statement MaybeElse {
        |$1: Token, $3: Sem, $5: Sem, $6: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Statement(Statement::If(If {
                loc: $1.get_loc(),
                cond: get_move!($3, Expr),
                on_true: Box::new(get_move!($5, Statement)),
                on_false: match $6.value {
                    SemValue::Statement(statement) => {Some(Box::new(statement))}
                    SemValue::None => {None}
                    _ => unreachable!(),
                },
            })),
        }
    }
    ;

MaybeElse
    : ELSE Statement {
        |$1: Token, $2: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Statement(get_move!($2, Statement)),
        };
    }
    | /* empty */  {
        || -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::None,
        };
    }
    ;

ObjectCopy
    : SCOPY '(' Identifier ',' Expr ')' {
        |$1: Token, $3: Sem, $5: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Statement(Statement::ObjectCopy(ObjectCopy {
                loc: $1.get_loc(),
                dst: get_move!($3, Identifier),
                src: get_move!($5, Expr),
            })),
        }
    }
    ;

Foreach
    : FOREACH '(' TypeOrVar Identifier IN Expr MaybeForeachCond ')' Statement {
        |$1: Token, $3: Sem, $4: Sem, $6: Sem, $7: Sem, $9: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Statement(Statement::Foreach(Foreach {
                loc: $1.get_loc(),
                type_: get_move!($3, Type),
                name: get_move!($4, Identifier),
                array: get_move!($6, Expr),
                cond: match $7.value {
                    SemValue::Expr(expr) => Some(expr),
                    SemValue::None => None,
                    _ => unreachable!(),
                },
                body: Box::new(get_move!($9, Statement)),
            })),
        }
    }
    ;

TypeOrVar
    : VAR {
        |$1: Token| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Type(Type::Var),
        };
    }
    | Type {
        |$1: Sem| -> Sem;
        $$ = Sem {
            loc: $1.loc,
            value: SemValue::Type(get_move!($1, Type)),
        };
    }
    ;

MaybeForeachCond
    : WHILE Expr {
        |$1: Token, $2: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Expr(get_move!($2, Expr)),
        };
    }
    | /* empty */ {
        || -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::None,
        };
    }
    ;

Guarded
    : IF '{' GuardedBranchesOrEmpty '}' {
        |$1: Token, $3: Sem|-> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Statement(Statement::Guarded(Guarded {
                loc: $1.get_loc(),
                guarded: get_move!($3, GuardedList),
            }))
        };
    }
    ;

GuardedBranchesOrEmpty
    :  GuardedBranches {
        |$1: Sem| -> Sem;
        $$ = $1;
    }
    | /* empty */ {
        || -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::GuardedList(Vec::new()),
        };
    }
    ;

GuardedBranches
    : GuardedBranches GUARD_SPLIT Expr ':' Statement {
        |$1: Sem, $3: Sem, $5: Sem| -> Sem;
        let mut ret = $1;
        get_ref!(ret, GuardedList).push((get_move!($3, Expr), get_move!($5, Statement)));
        $$ = ret;
    }
    | Expr ':' Statement {
        |$1: Sem, $3: Sem| -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::GuardedList(vec![(get_move!($1, Expr), get_move!($3, Statement))]),
        };
    }
    ;

Return
    : RETURN Expr {
        |$1: Token, $2: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Statement(Statement::Return(Return {
                loc: $1.get_loc(),
                expr: Some(get_move!($2, Expr)),
            }))
        };
    }
    | RETURN {
        |$1: Token| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Statement(Statement::Return(Return {
                loc: $1.get_loc(),
                expr: None,
            }))
        };
    }
    ;


Print
    : PRINT '(' ExprList ')' {
        |$1: Token, $3: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Statement(Statement::Print(Print {
                loc: $1.get_loc(),
                print: get_move!($3, ExprList),
            }))
        };
    }
    ;

ExprList
    : ExprList ',' Expr {
        |$1: Sem, $3: Sem| -> Sem;
        let mut ret = $1;
        get_ref!(ret, ExprList).push(get_move!($3, Expr));
        $$ = ret;
    }
    | Expr {
        |$1: Sem| -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::ExprList(vec!(get_move!($1, Expr))),
        };
    }
    ;

Simple
    : LValue '=' Expr {
        |$1: Sem, $2: Token, $3: Sem| -> Sem;
        $$ = Sem {
            loc: $2.get_loc(),
            value: SemValue::Statement(Statement::Simple(Simple::Assign(Assign {
                loc: $2.get_loc(),
                dst: get_move!($1, LValue),
                src: get_move!($3, Expr),
            }))),
        };
    }
    | VAR Identifier '=' Expr {
        |$2: Sem, $3: Token, $4: Sem| -> Sem;
        $$ = Sem {
            loc: $3.get_loc(),
            value: SemValue::Statement(Statement::Simple(Simple::VarAssign(VarAssign {
                loc: $3.get_loc(),
                name: get_move!($2, Identifier),
                src: get_move!($4, Expr),
            }))),
        };
    }
    | Expr {
        |$1: Sem| -> Sem;
        $$ = Sem {
            loc: $1.loc,
            value: SemValue::Statement(Statement::Simple(Simple::Expr(get_move!($1, Expr)))),
        };
    }
    | /* empty */ {
        || -> Sem;
        $$ = Sem {
            loc: self.get_loc(),
            value: SemValue::Statement(Statement::Simple(Simple::Skip(Skip {
                loc: self.get_loc(),
            }))),
        };
    }
    ;

Expr
    : LValue {
        |$1: Sem| -> Sem;
        $$ = Sem {
            loc: $1.loc,
            value: SemValue::Expr(Expr::LValue(get_move!($1, LValue))),
        }
    }
    | Call {
        |$1: Sem| -> Sem;
        $$ = $1;
    }
    | Const {
        |$1: Sem| -> Sem;
        $$ = Sem {
            loc: $1.loc,
            value: SemValue::Expr(Expr::Const(get_move!($1, Const))),
        }
    }
    | Expr '+' Expr {
        |$1: Sem, $2: Token, $3: Sem| -> Sem;
        $$ = gen_binary($1, $2, $3, Operator::Add);
    }
    | Expr '-' Expr {
        |$1: Sem, $2: Token, $3: Sem| -> Sem;
        $$ = gen_binary($1, $2, $3, Operator::Sub);
    }
    | Expr '*' Expr {
        |$1: Sem, $2: Token, $3: Sem| -> Sem;
        $$ = gen_binary($1, $2, $3, Operator::Mul);
    }
    | Expr '%' Expr {
        |$1: Sem, $2: Token, $3: Sem| -> Sem;
        $$ = gen_binary($1, $2, $3, Operator::Mod);
    }
    | Expr EQUAL Expr {
        |$1: Sem, $2: Token, $3: Sem| -> Sem;
        $$ = gen_binary($1, $2, $3, Operator::Eq);
    }
    | Expr NOT_EQUAL Expr {
        |$1: Sem, $2: Token, $3: Sem| -> Sem;
        $$ = gen_binary($1, $2, $3, Operator::Ne);
    }
    | Expr '<' Expr {
        |$1: Sem, $2: Token, $3: Sem| -> Sem;
        $$ = gen_binary($1, $2, $3, Operator::Lt);
    }
    | Expr '>' Expr {
        |$1: Sem, $2: Token, $3: Sem| -> Sem;
        $$ = gen_binary($1, $2, $3, Operator::Gt);
    }
    | Expr LESS_EQUAL Expr {
        |$1: Sem, $2: Token, $3: Sem| -> Sem;
        $$ = gen_binary($1, $2, $3, Operator::Le);
    }
    | Expr GREATER_EQUAL Expr {
        |$1: Sem, $2: Token, $3: Sem| -> Sem;
        $$ = gen_binary($1, $2, $3, Operator::Ge);
    }
    | Expr AND Expr {
        |$1: Sem, $2: Token, $3: Sem| -> Sem;
        $$ = gen_binary($1, $2, $3, Operator::And);
    }
    | Expr OR Expr {
        |$1: Sem, $2: Token, $3: Sem| -> Sem;
        $$ = gen_binary($1, $2, $3, Operator::Or);
    }
    | Expr REPEAT Expr {
        |$1: Sem, $2: Token, $3: Sem| -> Sem;
        $$ = gen_binary($1, $2, $3, Operator::Repeat);
    }
    | Expr CONCAT Expr {
        |$1: Sem, $2: Token, $3: Sem| -> Sem;
        $$ = gen_binary($1, $2, $3, Operator::Concat);
    }
    | Expr '[' Expr ':' Expr ']' {
        |$1: Sem, $2: Token, $3: Sem, $5: Sem| -> Sem;
        $$ = Sem {
            loc: $2.get_loc(),
            value: SemValue::Expr(Expr::Range(Range {
                loc: $2.get_loc(),
                array: Box::new(get_move!($1, Expr)),
                lower: Box::new(get_move!($3, Expr)),
                upper: Box::new(get_move!($5, Expr)),
            }))
        };
    }
    | Expr '[' Expr ']' DEFAULT Expr {
        |$1: Sem, $2: Token, $3: Sem, $6: Sem| -> Sem;
        $$ = Sem {
            loc: $2.get_loc(),
            value: SemValue::Expr(Expr::Default(Default {
                loc: $2.get_loc(),
                array: Box::new(get_move!($1, Expr)),
                index: Box::new(get_move!($3, Expr)),
                default: Box::new(get_move!($6, Expr)),
            }))
        };
    }
    | '[' Expr FOR Identifier IN Expr ']' {
        |$1: Token, $2: Sem, $4: Sem, $6: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Expr(Expr::Comprehension(Comprehension {
                loc: $1.get_loc(),
                expr: Box::new(get_move!($2, Expr)),
                name: get_move!($4, Identifier),
                array: Box::new(get_move!($6, Expr)),
                cond: None,
            })),
        };
    }
    | '[' Expr FOR Identifier IN Expr IF Expr ']' {
        |$1: Token, $2: Sem, $4: Sem, $6: Sem, $8: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Expr(Expr::Comprehension(Comprehension {
                loc: $1.get_loc(),
                expr: Box::new(get_move!($2, Expr)),
                name: get_move!($4, Identifier),
                array: Box::new(get_move!($6, Expr)),
                cond: Some(Box::new(get_move!($8, Expr))),
            })),
        };
    }
    | '(' Expr ')' {
        |$2: Sem| -> Sem;
        $$ = $2;
    }
    | '-' Expr %prec UMINUS {
        |$1: Token, $2: Sem| -> Sem;
        $$ = gen_unary($1, $2, Operator::Neg);
    }
    | '!' Expr {
        |$1: Token, $2: Sem| -> Sem;
        $$ = gen_unary($1, $2, Operator::Not);
    }
    | READ_INTEGER '(' ')' {
        |$1: Token| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Expr(Expr::ReadInt(ReadInt {
                loc: $1.get_loc(),
            }))
        };
    }
    | READ_LINE '(' ')' {
        |$1: Token| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Expr(Expr::ReadLine(ReadLine {
                loc: $1.get_loc(),
            }))
        };
    }
    | THIS {
        |$1: Token| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Expr(Expr::This(This {
                loc: $1.get_loc(),
            }))
        };
    }
    | NEW Identifier '(' ')' {
        |$1: Token, $2: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Expr(Expr::NewClass(NewClass {
                loc: $1.get_loc(),
                name: get_move!($2, Identifier),
            }))
        };
    }
    | NEW Type '[' Expr ']' {
        |$1: Token, $2: Sem, $4: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Expr(Expr::NewArray(NewArray {
                loc: $1.get_loc(),
                type_: get_move!($2, Type),
                len: Box::new(get_move!($4, Expr)),
            }))
        };
    }
    | INSTANCEOF '(' Expr ',' Identifier ')' {
        |$1: Token, $3: Sem, $5: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Expr(Expr::TypeTest(TypeTest {
                loc: $1.get_loc(),
                expr: Box::new(get_move!($3, Expr)),
                name: get_move!($5, Identifier),
            }))
        };
    }
    | '(' CLASS Identifier ')' Expr {
        |$3: Sem, $5: Sem| -> Sem;
        $$ = Sem {
            loc: $3.loc,
            value: SemValue::Expr(Expr::TypeCast(TypeCast {
                loc: $3.loc,
                name: get_move!($3, Identifier),
                expr: Box::new(get_move!($5, Expr)),
            }))
        };
    }
    ;

LValue
    : MaybeReceiver Identifier {
        |$1: Sem, $2: Sem| -> Sem;
        $$ = Sem {
            loc: $2.loc,
            value: SemValue::LValue(LValue::Identifier(Identifier {
                loc: $2.loc,
                owner: match $1.value {
                    SemValue::Expr(expr) => {Some(Box::new(expr))}
                    SemValue::None => {None}
                    _ => unreachable!(),
                },
                name: get_move!($2, Identifier),
            })),
        };
    }
    | Expr '[' Expr ']' {
        |$1: Sem, $3: Sem| -> Sem;
        $$ = Sem {
            loc: $1.loc,
            value: SemValue::LValue(LValue::Indexed(Indexed {
                loc: $1.loc,
                array: Box::new(get_move!($1, Expr)),
                index: Box::new(get_move!($3, Expr)),
            }))
        }
    }
    ;

MaybeReceiver
    : Expr '.' {
        |$1: Sem| -> Sem;
        $$ = $1;
    }
    | /* empty */ {
        || -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::None,
        };
    }
    ;

Call
    : MaybeReceiver Identifier '(' ExprListOrEmpty ')' {
        |$1: Sem, $2: Sem, $4: Sem| -> Sem;
        $$ = Sem {
            loc: $2.loc,
            value: SemValue::Expr(Expr::Call(Call {
                loc: $2.loc,
                receiver: match $1.value {
                    SemValue::Expr(expr) => Some(Box::new(expr)),
                    SemValue::None => None,
                    _ => unreachable!(),
                },
                name: get_move!($2, Identifier),
                arguments: get_move!($4, ExprList),
            })),
        }
    }
    ;
                
Const        
    : INT_CONST {
        |$1: Token| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Const(Const::IntConst(IntConst {
                loc: $1.get_loc(),
                value: yytext.parse::<i32>().unwrap(),
            })),
        };
    }
    | TRUE {
        |$1: Token| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Const(Const::BoolConst(BoolConst {
                loc: $1.get_loc(),
                value: true,
            })),
        };
    }
    | FALSE {
        |$1: Token| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Const(Const::BoolConst(BoolConst {
                loc: $1.get_loc(),
                value: false,
            })),
        };
    }
    | ArrayConst {
        |$1: Sem| -> Sem;
        $$ = $1;
    }
    | NULL {
        |$1: Token| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Const(Const::Null(Null {
                loc: $1.get_loc(),
            })),
        };
    }
    ;

ArrayConst
    : '[' ConstList ']' {
        |$2: Sem| -> Sem;
        $$ = $2;
    }
    | '[' ']' {
        || -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::Const(Const::ArrayConst(ArrayConst {
                loc: NO_LOCATION,
                value: Vec::new(),
            })),
        };
    }
    ;

ConstList
    : ConstList ',' Const {
        |$1: Sem, $3: Sem| -> Sem;
        let mut ret = $1;
        match get_ref!(ret, Const) {
            Const::ArrayConst(array_const) => array_const.value.push(get_move!($3, Const)),
            _ => unreachable!(),
        };
        $$ = ret;
    }
    | Const {
        |$1: Sem| -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::Const(Const::ArrayConst(ArrayConst {
                loc: NO_LOCATION,
                value: vec![get_move!($1, Const)],
            })),
        };
    }
    ;


ExprListOrEmpty
    : ExprList {
        |$1: Sem| -> Sem;
        $$ = $1;
    }
    | /* empty */ {
        || -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::ExprList(Vec::new()),
        };
    }
    ;

VarDef
    : Variable ';' {
        $$ = $1;
    }
    ;

Variable
    : Type Identifier {
        |$1: Sem, $2: Sem| -> Sem;
        $$ = Sem {
            loc: $2.loc,
            value: SemValue::VarDef(VarDef {
                loc: $2.loc,
                name: get_move!($2, Identifier),
                type_: get_move!($1, Type),
            })
        };
    }
    ;
                
Type
    : INT {
        |$1: Token| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Type(Type::Basic("int")),
        };
    }
    | VOID {
        |$1: Token| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Type(Type::Basic("void")),
        };
    }
    | BOOL {
        |$1: Token| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Type(Type::Basic("bool")),
        };
    }
    | STRING {
        |$1: Token| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Type(Type::Basic("string")),
        };
    }
    | CLASS Identifier  {
        |$1: Token, $2: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::Type(Type::Class(get_move!($2, Identifier))),
        };
    }
    | Type '[' ']' {
        |$1: Sem| -> Sem;
        $$ = Sem {
            loc: $1.loc,
            value: SemValue::Type(Type::Array(Some(Box::new(get_move!($1, Type))))),
        };
    }
    ;

Identifier
    : IDENTIFIER {
        |$1: Token| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            // yytext.to_string() return s the current name
            value: SemValue::Identifier(yytext.to_string()),
        }
    }
    ;