// syntax-cli -g parser/parser.rs.g -m lalr1 -o parser/src/lib.rs --validate

%lex

%x S

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
"/"                 return "'/'";
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

<INITIAL>\"         {
                        self.begin("S");
                        self.string_builder.0.clear();
                        self.string_builder.1 = self.token_start_line;
                        self.string_builder.2 = self.token_start_column + 1;
                        return "";
                    }
<S>\n               {
                        let loc = Location(self.string_builder.1, self.string_builder.2);
                        let string = util::quote(&self.string_builder.0.clone());
                        self.report_error(Error::new(loc, NewlineInStr{ string }));
                        return "";
                    }
// it must be accompanied by \n, so no-op here
<S>\r               return "";
<S>$                {
                        let loc = Location(self.string_builder.1, self.string_builder.2);
                        let string = util::quote(&self.string_builder.0.clone());
                        self.report_error(Error::new(loc, UnterminatedStr{ string }));
                        self.begin("INITIAL");
                        return "";
                    }
<S>\"               { self.begin("INITIAL"); return "STRING_CONST"; }
<S>"\n"             { self.string_builder.0.push('\n'); return ""; }
<S>"\t"             { self.string_builder.0.push('\t'); return ""; }
<S>\\\u0022         { self.string_builder.0.push('"');  return ""; }
<S>\\\\             { self.string_builder.0.push('\\'); return ""; }
<S>.                { self.string_builder.0.push_str(yytext); return ""; }


\u002f\u002f[^\n]*  return "";
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
%left  '*' '/' '%'
%nonassoc UMINUS '!'
%nonassoc '[' '.' DEFAULT
%nonassoc ')' EMPTY
%nonassoc ELSE

%{

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

%}

%%

Program
    : ClassList {
        |$1: ClassList| -> Result<Program, Vec<Error>>;
        $$ = if self.errors.is_empty() {
            Ok(Program { classes: $1, })
        } else {
            Err(std::mem::replace(&mut self.errors, Vec::new()))
        }
    }
    ;

ClassList
    : ClassList ClassDef {
        |$1: ClassList, $2: ClassDef| -> ClassList;
        $1.push($2);
        $$ = $1;
    }
    | ClassDef {
        |$1: ClassDef| -> ClassList;
        $$ = vec![$1];
    }
    ;

ClassDef
    : MaybeSealed CLASS IDENTIFIER MaybeExtends  '{' FieldList '}' {
        |$1: Flag, $2: Token, $3: Token, $4: Option<String>, $6: FieldList| -> ClassDef;
        $$ = ClassDef {
            loc: $2.get_loc(),
            name: $3.get_id(),
            parent: $4,
            fields: $6,
            sealed: $1,
        };
    }
    ;

MaybeSealed
    : SEALED {
        || -> Flag;
        $$ = true;
    }
    | /* empty */ {
        || -> Flag;
        $$ = false;
    }
    ;

MaybeExtends
    : EXTENDS IDENTIFIER {
        |$2: Token| -> Option<String>;
        $$ = Some($2.get_id());
    }
    | /* empty */ {
        || -> Option<String>;
        $$ = None;
    }
    ;

FieldList
    : FieldList VarDef ';' {
        |$1: FieldList, $2: VarDef| -> FieldList;
        $1.push(FieldDef::VarDef($2));
        $$ = $1;
    }
    | FieldList MethodDef {
        |$1: FieldList, $2: MethodDef| -> FieldList;
        $1.push(FieldDef::MethodDef($2));
        $$ = $1;
    }
    | /* empty */ {
        || -> FieldList;
        $$ = Vec::new();
    }
    ;

// I don't know why use 'MaybeStatic -> eps | STATIC' will cause shift-reduce conflict
MethodDef
    : STATIC Type IDENTIFIER '(' VarDefListOrEmpty ')' Block {
        |$2: Type, $3: Token, $5: VarDefList, $7: Block| -> MethodDef;
        $$ = MethodDef {
            loc: $3.get_loc(),
            name: $3.get_id(),
            return_type: $2,
            parameters: $5,
            static_: true,
            body: $7,
        };
    }
    | Type IDENTIFIER '(' VarDefListOrEmpty ')' Block {
        |$1: Type, $2: Token, $4: VarDefList, $6: Block| -> MethodDef;
        $$ = MethodDef {
            loc: $2.get_loc(),
            name: $2.get_id(),
            return_type: $1,
            parameters: $4,
            static_: false,
            body: $6,
        };
    }
    ;

VarDefListOrEmpty
    : VarDefList {
        |$1: VarDefList| -> VarDefList;
        $$ = $1;
    }
    | /* empty */ {
        || -> VarDefList;
        $$ = Vec::new();
    }
    ;

VarDefList
    : VarDefList ',' VarDef {
        |$1: VarDefList, $3: VarDef| -> VarDefList;
        $1.push($3);
        $$ = $1;
    }
    | VarDef {
        |$1: VarDef| -> VarDefList;
        $$ = vec![$1];
    }
    ;

Block
    : '{' StatementList '}' {
        |$1: Token, $2: StatementList| -> Block;
        $$ = Block {
            loc: $1.get_loc(),
            statements: $2,
        };
    }
    ;

StatementList
    : StatementList Statement {
        |$1: StatementList, $2: Statement| -> StatementList;
        $1.push($2);
        $$ = $1;
    }
    | /* empty */ {
        || -> StatementList;
        $$ = Vec::new();
    }
    ;

Statement
    : VarDef ';' {
        |$1: VarDef| -> Statement;
        $$ = Statement::VarDef($1);
    }
    | Simple ';' {
        |$1: Simple| -> Statement;
        $$ = Statement::Simple($1);
    }
    | If {
        |$1: Statement| -> Statement;
        $$ = $1;
    }
    | While {
        |$1: Statement| -> Statement;
        $$ = $1;
    }
    | For {
        |$1: Statement| -> Statement;
        $$ = $1;
    }
    | Return ';' {
        |$1: Statement| -> Statement;
        $$ = $1;
    }
    | Print ';' {
        |$1: Statement| -> Statement;
        $$ = $1;
    }
    | Break ';' {
        |$1: Statement| -> Statement;
        $$ = $1;
    }
    | ObjectCopy ';' {
        |$1: Statement| -> Statement;
        $$ = $1;
    }
    | Foreach {
        |$1: Statement| -> Statement;
        $$ = $1;
    }
    | Guarded {
        |$1: Statement| -> Statement;
        $$ = $1;
    }
    | Block {
        |$1: Block| -> Statement;
        $$ = Statement::Block($1);
    }
    ;

While
    : WHILE '(' Expr ')' Statement {
        |$1: Token, $3: Expr, $5: Statement| -> Statement;
        $$ = Statement::While(While {
            loc: $1.get_loc(),
            cond: $3,
            body: Box::new($5),
        });
    }
    ;

For
    : FOR '(' Simple ';' Expr ';' Simple ')' Statement {
        |$1: Token, $3: Simple, $5: Expr, $7: Simple, $9: Statement| -> Statement;
        $$ = Statement::For(For {
            loc: $1.get_loc(),
            init: $3,
            cond: $5,
            update: $7,
            body: Box::new($9),
        });
    }
    ;

Break
    : BREAK {
        |$1: Token| -> Statement;
        $$ = Statement::Break(Break { loc: $1.get_loc(), });
    }
    ;

If
    : IF '(' Expr ')' Statement MaybeElse {
        |$1: Token, $3: Expr, $5: Statement, $6: Option<Statement>| -> Statement;
        $$ = Statement::If(If {
            loc: $1.get_loc(),
            cond: $3,
            on_true: Box::new($5),
            on_false: match $6 {
                Some(statement) => Some(Box::new(statement)),
                None => None,
            },
        });
    }
    ;

MaybeElse
    : ELSE Statement {
        |$1: Token, $2: Statement| -> Option<Statement>;
        $$ = Some($2);
    }
    | /* empty */  {
        || -> Option<Statement>;
        $$ = None;
    }
    ;

ObjectCopy
    : SCOPY '(' IDENTIFIER ',' Expr ')' {
        |$1: Token, $3: Token, $5: Expr| -> Statement;
        $$ = Statement::ObjectCopy(ObjectCopy {
            loc: $1.get_loc(),
            dst: $3.get_id(),
            src: $5,
        });
    }
    ;

Foreach
    : FOREACH '(' TypeOrVar IDENTIFIER IN Expr MaybeForeachCond ')' Statement {
        |$1: Token, $3: Type, $4: Token, $6: Expr, $7: Option<Expr>, $9: Statement| -> Statement;
        $$ = Statement::Foreach(Foreach {
            loc: $1.get_loc(),
            type_: $3,
            name: $4.get_id(),
            array: $6,
            cond: $7,
            body: Box::new($9),
        });
    }
    ;

TypeOrVar
    : VAR {
        || -> Type;
        $$ = Type::Var;
    }
    | Type {
        |$1: Type| -> Type;
        $$ = $1;
    }
    ;

MaybeForeachCond
    : WHILE Expr {
        |$1: Token, $2: Expr| -> Option<Expr>;
        $$ = Some($2);
    }
    | /* empty */ {
        || -> Option<Expr>;
        $$ = None;
    }
    ;

Guarded
    : IF '{' GuardedBranchesOrEmpty '}' {
        |$1: Token, $3: GuardedList|-> Statement;
        $$ = Statement::Guarded(Guarded {
            loc: $1.get_loc(),
            guarded: $3,
        });
    }
    ;

GuardedBranchesOrEmpty
    :  GuardedBranches {
        |$1: GuardedList| -> GuardedList;
        $$ = $1;
    }
    | /* empty */ {
        || -> GuardedList;
        $$ = Vec::new();
    }
    ;

GuardedBranches
    : GuardedBranches GUARD_SPLIT Expr ':' Statement {
        |$1: GuardedList, $3: Expr, $5: Statement| -> GuardedList;
        $1.push(($3, $5));
        $$ = $1;
    }
    | Expr ':' Statement {
        |$1: Expr, $3: Statement| -> GuardedList;
        $$ = vec![($1, $3)];
    }
    ;

Return
    : RETURN Expr {
        |$1: Token, $2: Expr| -> Statement;
        $$ = Statement::Return(Return {
            loc: $1.get_loc(),
            expr: Some($2),
        });
    }
    | RETURN {
        |$1: Token| -> Statement;
        $$ = Statement::Return(Return {
            loc: $1.get_loc(),
            expr: None,
        });
    }
    ;


Print
    : PRINT '(' ExprList ')' {
        |$1: Token, $3: ExprList| -> Statement;
        $$ = Statement::Print(Print {
            loc: $1.get_loc(),
            print: $3,
        });
    }
    ;

ExprList
    : ExprList ',' Expr {
        |$1: ExprList, $3: Expr| -> ExprList;
        $1.push($3);
        $$ = $1;
    }
    | Expr {
        |$1: Expr| -> ExprList;
        $$ = vec![$1];
    }
    ;

Simple
    : LValue '=' Expr {
        |$1: LValue, $2: Token, $3: Expr| -> Simple;
        $$ = Simple::Assign(Assign {
            loc: $2.get_loc(),
            dst: $1,
            src: $3,
        });
    }
    | VAR IDENTIFIER '=' Expr {
        |$2: Token, $3: Token, $4: Expr| -> Simple;
        $$ = Simple::VarAssign(VarAssign {
            loc: $3.get_loc(),
            name: $2.get_id(),
            src: $4,
        });
    }
    | Expr {
        |$1: Expr| -> Simple;
        $$ = Simple::Expr($1);
    }
    | /* empty */ {
        || -> Simple;
        $$ = Simple::Skip(Skip { loc: self.get_loc(), });
    }
    ;

Expr
    : LValue {
        |$1: LValue| -> Expr;
        $$ = Expr::LValue($1);
    }
    | Call {
        |$1: Expr| -> Expr;
        $$ = $1;
    }
    | Const {
        |$1: Const| -> Expr;
        $$ = Expr::Const($1);
    }
    | Expr '+' Expr {
        |$1: Expr, $2: Token, $3: Expr| -> Expr;
        $$ = gen_binary($1, $2, $3, Operator::Add);
    }
    | Expr '-' Expr {
        |$1: Expr, $2: Token, $3: Expr| -> Expr;
        $$ = gen_binary($1, $2, $3, Operator::Sub);
    }
    | Expr '*' Expr {
        |$1: Expr, $2: Token, $3: Expr| -> Expr;
        $$ = gen_binary($1, $2, $3, Operator::Mul);
    }
    | Expr '/' Expr {
        |$1: Expr, $2: Token, $3: Expr| -> Expr;
        $$ = gen_binary($1, $2, $3, Operator::Div);
    }
    | Expr '%' Expr {
        |$1: Expr, $2: Token, $3: Expr| -> Expr;
        $$ = gen_binary($1, $2, $3, Operator::Mod);
    }
    | Expr EQUAL Expr {
        |$1: Expr, $2: Token, $3: Expr| -> Expr;
        $$ = gen_binary($1, $2, $3, Operator::Eq);
    }
    | Expr NOT_EQUAL Expr {
        |$1: Expr, $2: Token, $3: Expr| -> Expr;
        $$ = gen_binary($1, $2, $3, Operator::Ne);
    }
    | Expr '<' Expr {
        |$1: Expr, $2: Token, $3: Expr| -> Expr;
        $$ = gen_binary($1, $2, $3, Operator::Lt);
    }
    | Expr '>' Expr {
        |$1: Expr, $2: Token, $3: Expr| -> Expr;
        $$ = gen_binary($1, $2, $3, Operator::Gt);
    }
    | Expr LESS_EQUAL Expr {
        |$1: Expr, $2: Token, $3: Expr| -> Expr;
        $$ = gen_binary($1, $2, $3, Operator::Le);
    }
    | Expr GREATER_EQUAL Expr {
        |$1: Expr, $2: Token, $3: Expr| -> Expr;
        $$ = gen_binary($1, $2, $3, Operator::Ge);
    }
    | Expr AND Expr {
        |$1: Expr, $2: Token, $3: Expr| -> Expr;
        $$ = gen_binary($1, $2, $3, Operator::And);
    }
    | Expr OR Expr {
        |$1: Expr, $2: Token, $3: Expr| -> Expr;
        $$ = gen_binary($1, $2, $3, Operator::Or);
    }
    | Expr REPEAT Expr {
        |$1: Expr, $2: Token, $3: Expr| -> Expr;
        $$ = gen_binary($1, $2, $3, Operator::Repeat);
    }
    | Expr CONCAT Expr {
        |$1: Expr, $2: Token, $3: Expr| -> Expr;
        $$ = gen_binary($1, $2, $3, Operator::Concat);
    }
    | Expr '[' Expr ':' Expr ']' {
        |$1: Expr, $2: Token, $3: Expr, $5: Expr| -> Expr;
        $$ = Expr::Range(Range {
            loc: $2.get_loc(),
            array: Box::new($1),
            lower: Box::new($3),
            upper: Box::new($5),
        });
    }
    | Expr '[' Expr ']' DEFAULT Expr {
        |$1: Expr, $2: Token, $3: Expr, $6: Expr| -> Expr;
        $$ = Expr::Default(Default {
            loc: $2.get_loc(),
            array: Box::new($1),
            index: Box::new($3),
            default: Box::new($6),
        });
    }
    | '[' Expr FOR IDENTIFIER IN Expr ']' {
        |$1: Token, $2: Expr, $4: Token, $6: Expr| -> Expr;
        $$ = Expr::Comprehension(Comprehension {
            loc: $1.get_loc(),
            expr: Box::new($2),
            name: $4.get_id(),
            array: Box::new($6),
            cond: None,
        });
    }
    | '[' Expr FOR IDENTIFIER IN Expr IF Expr ']' {
        |$1: Token, $2: Expr, $4: Token, $6: Expr, $8: Expr| -> Expr;
        $$ = Expr::Comprehension(Comprehension {
            loc: $1.get_loc(),
            expr: Box::new($2),
            name: $4.get_id(),
            array: Box::new($6),
            cond: Some(Box::new($8)),
        });
    }
    | '(' Expr ')' {
        |$2: Expr| -> Expr;
        $$ = $2;
    }
    | '-' Expr %prec UMINUS {
        |$1: Token, $2: Expr| -> Expr;
        $$ = gen_unary($1, $2, Operator::Neg);
    }
    | '!' Expr {
        |$1: Token, $2: Expr| -> Expr;
        $$ = gen_unary($1, $2, Operator::Not);
    }
    | READ_INTEGER '(' ')' {
        |$1: Token| -> Expr;
        $$ = Expr::ReadInt(ReadInt { loc: $1.get_loc(), });
    }
    | READ_LINE '(' ')' {
        |$1: Token| -> Expr;
        $$ = Expr::ReadLine(ReadLine { loc: $1.get_loc(), });
    }
    | THIS {
        |$1: Token| -> Expr;
        $$ = Expr::This(This { loc: $1.get_loc(), });
    }
    | NEW IDENTIFIER '(' ')' {
        |$1: Token, $2: Token| -> Expr;
        $$ = Expr::NewClass(NewClass {
            loc: $1.get_loc(),
            name: $2.get_id(),
        });
    }
    | NEW Type '[' Expr ']' {
        |$1: Token, $2: Type, $4: Expr| -> Expr;
        $$ = Expr::NewArray(NewArray {
            loc: $1.get_loc(),
            type_: $2,
            len: Box::new($4),
        });
    }
    | INSTANCEOF '(' Expr ',' IDENTIFIER ')' {
        |$1: Token, $3: Expr, $5: Token| -> Expr;
        $$ = Expr::TypeTest(TypeTest {
            loc: $1.get_loc(),
            expr: Box::new($3),
            name: $5.get_id(),
        });
    }
    | '(' CLASS IDENTIFIER ')' Expr {
        |$3: Token, $5: Expr| -> Expr;
        $$ = Expr::TypeCast(TypeCast {
            loc: $3.get_loc(),
            name: $3.get_id(),
            expr: Box::new($5),
        });
    }
    ;

LValue
    : MaybeReceiver IDENTIFIER {
        |$1: Option<Expr>, $2: Token| -> LValue;
        $$ = LValue::Identifier(Identifier {
            loc: $2.get_loc(),
            owner: match $1 {
                Some(expr) => Some(Box::new(expr)),
                None => None,
            },
            name: $2.get_id(),
        });
    }
    | Expr '[' Expr ']' {
        |$1: Expr, $3: Expr| -> LValue;
        $$ = LValue::Indexed(Indexed {
            loc: $1.get_loc(),
            array: Box::new($1),
            index: Box::new($3),
        });
    }
    ;

MaybeReceiver
    : Expr '.' {
        |$1: Expr| -> Option<Expr>;
        $$ = Some($1);
    }
    | /* empty */ {
        || -> Option<Expr>;
        $$ = None;
    }
    ;

Call
    : MaybeReceiver IDENTIFIER '(' ExprListOrEmpty ')' {
        |$1: Option<Expr>, $2: Token, $4: ExprList| -> Expr;
        $$ = Expr::Call(Call {
            loc: $2.get_loc(),
            receiver: match $1 {
                Some(expr) => Some(Box::new(expr)),
                None => None,
            },
            name: $2.get_id(),
            arguments: $4,
        });
    }
    ;
                
Const        
    : INT_CONST {
        |$1: Token| -> Const;
        $$ = Const::IntConst(IntConst {
            loc: $1.get_loc(),
            value: $1.value.parse::<i32>().unwrap_or_else(|_| {
                self.errors.push(Error::new($1.get_loc(), IntTooLarge{ string: $1.get_id(), }));
                0
            }),
        });
    }
    | TRUE {
        |$1: Token| -> Const;
        $$ = Const::BoolConst(BoolConst {
            loc: $1.get_loc(),
            value: true,
        });
    }
    | FALSE {
        |$1: Token| -> Const;
        $$ = Const::BoolConst(BoolConst {
            loc: $1.get_loc(),
            value: false,
        });
    }
    | STRING_CONST {
        || -> Const;
        $$ = Const::StringConst(StringConst {
            loc: Location(self.tokenizer.string_builder.1, self.tokenizer.string_builder.2),
            value: self.tokenizer.string_builder.0.clone(),
        });
    }
    | ArrayConst {
        |$1: ConstList| -> Const;
        $$ = Const::ArrayConst(ArrayConst {
            loc: self.get_loc(),
            value: $1,
        });
    }
    | NULL {
        |$1: Token| -> Const;
        $$ = Const::Null(Null { loc: $1.get_loc(), });
    }
    ;

ArrayConst
    : '[' ConstList ']' {
        |$2: ConstList| -> ConstList;
        $$ = $2;
    }
    | '[' ']' {
        || -> ConstList;
        $$ = Vec::new();
    }
    ;

ConstList
    : ConstList ',' Const {
        |$1: ConstList, $3: Const| -> ConstList;
        $1.push($3);
        $$ = $1;
    }
    | Const {
        |$1: Const| -> ConstList;
        $$ = vec![$1];
    }
    ;


ExprListOrEmpty
    : ExprList {
        |$1: ExprList| -> ExprList;
        $$ = $1;
    }
    | /* empty */ {
        || -> ExprList;
        $$ = Vec::new();
    }
    ;

// no ; followed
VarDef
    : Type IDENTIFIER {
        |$1: Type, $2: Token| -> VarDef;
        $$ = VarDef {
            loc: $2.get_loc(),
            name: $2.get_id(),
            type_: $1,
        };
    }
    ;
                
Type
    : INT {
        |$1: Token| -> Type;
        $$ = Type::Basic("int");
    }
    | VOID {
        |$1: Token| -> Type;
        $$ = Type::Basic("void");
    }
    | BOOL {
        |$1: Token| -> Type;
        $$ = Type::Basic("bool");
    }
    | STRING {
        |$1: Token| -> Type;
        $$ = Type::Basic("string");
    }
    | CLASS IDENTIFIER  {
        |$1: Token, $2: Token| -> Type;
        $$ = Type::Class($2.get_id());
    }
    | Type '[' ']' {
        |$1: Type| -> Type;
        $$ = Type::Array(Box::new($1));
    }
    ;
