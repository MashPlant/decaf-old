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

    pub fn parse_program(&self, string: &'static str) -> Program {
        match self.parse(string).value {
            Program(program) => program,
            _ => unreachable!(),
        }
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

%}

%%
Program
    : Program ClassDef {
        |$1: Tree, $2: Tree| -> Tree;
        let mut ret = $1;
        get_ref!(ret, Program).classes.push(get_move!($2, ClassDef));
        $$ = ret;
    }
    | ClassDef {
        |$1: Tree| -> Tree;
        $$ = Tree {
            loc: NO_LOCATION,
            data: TreeData::Program(Program {
                classes: vec![get_move!($1, ClassDef)],
            })
        }
    }
    ;

/*
ClassDef        :	CLASS IDENTIFIER ExtendsClause '{' FieldList '}'
					{
						$$.cdef = new Tree.ClassDef($2.ident, $3.ident, $5.flist, false, $1.loc);
					}
				| 	SEALED CLASS IDENTIFIER ExtendsClause '{' FieldList '}'
                    {
                    	$$.cdef = new Tree.ClassDef($3.ident, $4.ident, $6.flist, true, $1.loc);
                    }
                ;
*/
ClassDef
    : MaybeSealed CLASS Identifier MaybeExtends '{' FieldList '}' {
        |$1: Token, $2: Tree| -> Tree;
        $$ = Tree {
            loc: $1.get_loc(),
            data: TreeData::ClassDef(ClassDef {
                loc: $1.get_loc(),
                name: get_move!($2, Identifier),
                parent: None,
                fields: Vec::new(),
                sealed: false,
            })
        }
    }
    ;



VariableDef
    : Variable ';' {
        |$1: Tree| -> Tree;
        $$ = $1;
    }
    ;

Variable
    : Type Identifier {
        |$1: Tree, $2: Tree| -> Tree;
        $$ = Tree {
            loc: $2.loc,
            data: TreeData::VarDef(VarDef {
                loc: $2.loc,
                name: get_move!($2, Identifier),
                type_: get_move!($1, Type),
            })
        };
    }
    ;
                
Type
    : INT {
        |$1: Token| -> Tree;
        $$ = Tree {
            loc: $1.get_loc(),
            data: TreeData::Type(Type::Basic("int")),
        };
    }
    | VOID {
        |$1: Token| -> Tree;
        $$ = Tree {
            loc: $1.get_loc(),
            data: TreeData::Type(Type::Basic("void")),
        };
    }
    | BOOL {
        |$1: Token| -> Tree;
        $$ = Tree {
            loc: $1.get_loc(),
            data: TreeData::Type(Type::Basic("bool")),
        };
    }
    | STRING {
        |$1: Token| -> Tree;
        $$ = Tree {
            loc: $1.get_loc(),
            data: TreeData::Type(Type::Basic("string")),
        };
    }
    | CLASS Identifier  {
        |$1: Token, $2: Tree| -> Tree;
        $$ = Tree {
            loc: $1.get_loc(),
            data: TreeData::Type(Type::Class(get_move!($2, Identifier))),
        };
    }
    | Type '[' ']' {
        |$1: Tree| -> Tree;
        $$ = Tree {
            loc: $1.loc,
            data: TreeData::Type(Type::Array(Some(Box::new(get_move!($1, Type))))),
        };
    }
    ;

Identifier
    : IDENTIFIER {
        || -> Tree;
        $$ = Tree {
            loc: self.get_loc(),
            // yytext.to_string() return s the current name
            data: TreeData::Identifier(yytext.to_string()),
        }
    }
    ;