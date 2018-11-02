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
"class"             return "CLASS";
"extends"           return "EXTENDS";
"this"              return "THIS";
"while"             return "WHILE";
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
"foreach"           return "FOREACH";

// operators
"<="                return "LESS_EQUAL";
">="                return "GREATER_EQUAL";
"=="                return "EQUAL";
"!="                return "NOT_EQUAL";
"&&"                return "AND";
"||"                return "OR";
"%%"                return "REPEAT";
"++"                return "CONCAT";
"|||"               return "GUARD_SPLIT";
"["                 return "COMP_LEFT";
"]"                 return "COMP_RIGHT";
"+"                 return "+";
"*"                 return "*";
"("                 return "(";
")"                 return ")";

\s+                 return "";

\d+                 return "NUMBER";

[A-Za-z][_0-9A-Za-z]* return "IDENTIFIER";


/lex

%left + -
%left * /

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
    ClassDef(ClassDef),
    VarDef(VarDef),
    MethodDef(MethodDef),
    Type(Type),
    Statement(Statement),
    Block(Block),
    Expr(Expr),
    Sealed(bool),
    Static(bool),
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
    : CLASS Identifier '{' FieldList '}' {
        |$1: Token, $2: Sem, $4: Sem| -> Sem;
        $$ = Sem {
            loc: $1.get_loc(),
            value: SemValue::ClassDef(ClassDef {
                loc: $1.get_loc(),
                name: get_move!($2, Identifier),
                parent: None,
                fields: get_move!($4, FieldList),
                sealed: false,
            })
        }
    }
    ;

FieldList
    : FieldList VarDef {
        |$1: Sem, $2: Sem| -> Sem;
        let mut ret = $1;
        get_ref!(ret, FieldList).push(FieldDef::VarDef(get_move!($2, VarDef)));
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

MethodDef
    : MaybeStatic Type Identifier '(' Formals ')' Block {
        |$1:Sem, $2: Sem, $3: Sem, $5: Sem, $7: Sem| -> Sem;
        $$ = Sem {
            loc: $3.loc,
            value: SemValue::MethodDef(MethodDef {
                loc: $3.loc,
                name: get_move!($3, Identifier),
                return_type: get_move!($2, Type),
                parameters: get_move!($5, VarDefList),
                static_: get_move!($1, Static),
                body: get_move!($7, Block),
            })
        };
    }
    ;

MaybeStatic
    : STATIC {
        || -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::Static(true),
        };
    }
    | /* empty */ {
        || -> Sem;
        $$ = Sem {
            loc: NO_LOCATION,
            value: SemValue::Static(false),
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
            });
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
        $$.Statement = $1.vdef;
    }
    | SimpleStatement ';' {
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
            value: SemValue::Statement(Statement(get_move!($1, Block)));
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
        || -> Sem;
        $$ = Sem {
            loc: self.get_loc(),
            // yytext.to_string() return s the current name
            value: SemValue::Identifier(yytext.to_string()),
        }
    }
    ;