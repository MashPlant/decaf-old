%lex

%x S

%%

// keywords
"void"        return "VOID";
"int"         return "INT";
"bool"        return "BOOL";
"string"      return "STRING";
"new"         return "NEW";
"null"        return "NULL";
"true"        return "TRUE";
"false"       return "FALSE";
"class"       return "CLASS";
"extends"     return "EXTENDS";
"this"        return "THIS";
"while"       return "WHILE";
"foreach"     return "FOREACH";
"for"         return "FOR";
"if"          return "IF";
"else"        return "ELSE";
"return"      return "RETURN";
"break"       return "BREAK";
"Print"       return "PRINT";
"ReadInteger" return "READ_INTEGER";
"ReadLine"    return "READ_LINE";
"static"      return "STATIC";
"instanceof"  return "INSTANCEOF";
"scopy"       return "SCOPY";
"sealed"      return "SEALED";
"var"         return "VAR";
"default"     return "DEFAULT";
"in"          return "IN";

// operators
"|||"         return "GUARD_SPLIT";
"<="          return "LESS_EQUAL";
">="          return "GREATER_EQUAL";
"=="          return "EQUAL";
"!="          return "NOT_EQUAL";
"&&"          return "AND";
"||"          return "OR";
"%%"          return "REPEAT";
"++"          return "INC";
"--"          return "DEC";
"<<"          return "SHL";
">>"          return "SHR";

// simple operators
"+"           return "'+'";
"-"           return "'-'";
"*"           return "'*'";
"/"           return "'/'";
"%"           return "'%'";
"&"           return "'&'";
"|"           return "'|'";
"^"           return "'^'";
"="           return "'='";
"<"           return "'<'";
">"           return "'>'";
"."           return "'.'";
","           return "','";
";"           return "';'";
"!"           return "'!'";
"("           return "'('";
")"           return "')'";
"["           return "'['";
"]"           return "']'";
"{"           return "'{'";
"}"           return "'}'";
":"           return "':'";

<INITIAL>\"   {
                self.begin("S");
                self.string_builder.0.clear();
                self.string_builder.1 = self.token_start_line;
                self.string_builder.2 = self.token_start_column + 1;
                return "";
              }
<S>\n         {
                let loc = Loc(self.string_builder.1, self.string_builder.2);
                let string = print::quote(&self.string_builder.0.clone());
                self.report_error(Error::new(loc, NewlineInStr{ string }));
                return "";
                }
// it must be accompanied by \n, so no-op here
<S>\r         return "";
<S>$          {
                let loc = Loc(self.string_builder.1, self.string_builder.2);
                let string = print::quote(&self.string_builder.0.clone());
                self.report_error(Error::new(loc, UnterminatedStr{ string }));
                self.begin("INITIAL");
                return "";
              }
<S>\"         { self.begin("INITIAL"); return "STRING_CONST"; }
<S>"\n"       { self.string_builder.0.push('\n'); return "";  }
<S>"\t"       { self.string_builder.0.push('\t'); return "";  }
<S>\\\u0022   { self.string_builder.0.push('"');  return "";  }
<S>\\\\       { self.string_builder.0.push('\\'); return "";  }
<S>.          { self.string_builder.0.push_str(yytext); return ""; }


\u002f\u002f[^\n]*  return "";
\s+         return "";

\d+         return "INT_CONST";

[A-Za-z][_0-9A-Za-z]* return "IDENTIFIER";

/lex

%left OR
%left AND
%left '|'
%left '^'
%left '&'
%nonassoc EQUAL NOT_EQUAL
%nonassoc LESS_EQUAL GREATER_EQUAL '<' '>'
%left REPEAT
%left SHL SHR
%left '+' '-'
%left '*' '/' '%'
%nonassoc UMINUS '!' INC DEC
%nonassoc '[' '.' DEFAULT
%nonassoc ')' EMPTY
%nonassoc ELSE

%{

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
    ExprData::Binary(Binary { op, l: Box::new(l), r: Box::new(r), }))
}

fn gen_unary(opt: Token, r: Expr, op: Operator) -> Expr {
  Expr::new(opt.get_loc(),
    ExprData::Unary(Unary { op, r: Box::new(r), }))
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

%}

%%

Program
  : ClassList {
    |$1: ClassList| -> Result<Program, Vec<Error>>;
    $$ = if self.errors.is_empty() {
      Ok(Program { class: $1, ..D::default() })
    } else {
      Err(mem::replace(&mut self.errors, Vec::new()))
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
    |$1: Flag, $2: Token, $3: Token, $4: Option<Str>, $6: FieldList| -> ClassDef;
    $$ = ClassDef {
      loc: $2.get_loc(),
      name: $3.value,
      parent: $4,
      field: $6,
      sealed: $1,
      ..D::default()
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
    |$2: Token| -> Option<Str>;
    $$ = Some($2.value);
  }
  | /* empty */ {
    || -> Option<Str>;
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
      name: $3.value,
      ret_t: $2,
      param: $5,
      static_: true,
      body: $7,
      scope: D::default(),
      class: ptr::null(),
      offset: -1,
    };
  }
  | Type IDENTIFIER '(' VarDefListOrEmpty ')' Block {
    |$1: Type, $2: Token, $4: VarDefList, $6: Block| -> MethodDef;
    $$ = MethodDef {
      loc: $2.get_loc(),
      name: $2.value,
      ret_t: $1,
      param: $4,
      static_: false,
      body: $6,
      scope: D::default(),
      class: ptr::null(),
      offset: -1,
    };
  }
  ;

VarDefListOrEmpty
  : VarDefList {
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
  : '{' StmtList '}' {
    |$1: Token, $2: StmtList| -> Block;
    $$ = Block {
      loc: $1.get_loc(),
      stmt: $2,
      ..D::default()
    };
  }
  ;

StmtList
  : StmtList Stmt {
    |$1: StmtList, $2: Stmt| -> StmtList;
    $1.push($2);
    $$ = $1;
  }
  | /* empty */ {
    || -> StmtList;
    $$ = Vec::new();
  }
  ;

Stmt
  : Simple ';' {
    |$1: Simple| -> Stmt;
    $$ = Stmt::Simple($1);
  }
  | If {
    $$ = $1;
  }
  | While {
    $$ = $1;
  }
  | For {
    $$ = $1;
  }
  | Return ';' {
    $$ = $1;
  }
  | Print ';' {
    $$ = $1;
  }
  | Break ';' {
    $$ = $1;
  }
  | SCopy ';' {
    $$ = $1;
  }
  | Foreach {
    $$ = $1;
  }
  | Guarded {
    $$ = $1;
  }
  | Block {
    |$1: Block| -> Stmt;
    $$ = Stmt::Block($1);
  }
  ;

Blocked
  : Stmt {
    |$1: Stmt| -> Block;
    $$ = match $1 {
      Stmt::Block(block) => block,
      stmt => Block {
        loc: NO_LOC,
        stmt: vec![stmt],
        ..D::default()
      }
    }
  }
  ;

While
  : WHILE '(' Expr ')' Blocked {
    |$1: Token, $3: Expr, $5: Block| -> Stmt;
    $$ = Stmt::While(While {
      loc: $1.get_loc(),
      cond: $3,
      body: $5,
    });
  }
  ;

For
  : FOR '(' Simple ';' Expr ';' Simple ')' Blocked {
    |$1: Token, $3: Simple, $5: Expr, $7: Simple, $9: Block| -> Stmt;
    $$ = Stmt::For(For {
      loc: $1.get_loc(),
      init: $3,
      cond: $5,
      update: $7,
      body: $9,
    });
  }
  ;

Foreach
  : FOREACH '(' TypeOrVar IDENTIFIER IN Expr MaybeForeachCond ')' Blocked {
    |$3: Type, $4: Token, $6: Expr, $7: Option<Expr>, $9: Block| -> Stmt;
    $$ = Stmt::Foreach(Foreach {
      def: VarDef {
        loc: $4.get_loc(),
        name: $4.value,
        type_: $3,
        finish_loc: $4.get_loc(),
        src: None,
        scope: ptr::null(),
        index: D::default(),
        offset: -1,
      },
      arr: $6,
      cond: $7,
      body: $9,
    });
  }
  ;

Break
  : BREAK {
    |$1: Token| -> Stmt;
    $$ = Stmt::Break(Break { loc: $1.get_loc(), });
  }
  ;

If
  : IF '(' Expr ')' Blocked MaybeElse {
    |$1: Token, $3: Expr, $5: Block, $6: Option<Block>| -> Stmt;
    $$ = Stmt::If(If {
      loc: $1.get_loc(),
      cond: $3,
      on_true: $5,
      on_false: $6,
    });
  }
  ;

MaybeElse
  : ELSE Blocked {
    |$1: Token, $2: Block| -> Option<Block>;
    $$ = Some($2);
  }
  | /* empty */  {
    || -> Option<Block>;
    $$ = None;
  }
  ;

SCopy
  : SCOPY '(' IDENTIFIER ',' Expr ')' {
    |$1: Token, $3: Token, $5: Expr| -> Stmt;
    $$ = Stmt::SCopy(SCopy {
      loc: $1.get_loc(),
      dst_loc:$3.get_loc(),
      dst: $3.value,
      dst_sym: ptr::null(),
      src: $5,
    });
  }
  ;

TypeOrVar
  : VAR {
    |$1: Token| -> Type;
    $$ = Type { loc: $1.get_loc(), sem: VAR };
  }
  | Type {
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
    |$1: Token, $3: GuardedList|-> Stmt;
    $$ = Stmt::Guarded(Guarded {
      loc: $1.get_loc(),
      guarded: $3,
    });
  }
  ;

GuardedBranchesOrEmpty
  :  GuardedBranches {
    $$ = $1;
  }
  | /* empty */ {
    || -> GuardedList;
    $$ = Vec::new();
  }
  ;

GuardedBranches
  : GuardedBranches GUARD_SPLIT Expr ':' Blocked {
    |$1: GuardedList, $3: Expr, $5: Block| -> GuardedList;
    $1.push(($3, $5));
    $$ = $1;
  }
  | Expr ':' Blocked {
    |$1: Expr, $3: Block| -> GuardedList;
    $$ = vec![($1, $3)];
  }
  ;

Return
  : RETURN Expr {
    |$1: Token, $2: Expr| -> Stmt;
    $$ = Stmt::Return(Return {
      loc: $1.get_loc(),
      expr: Some($2),
    });
  }
  | RETURN {
    |$1: Token| -> Stmt;
    $$ = Stmt::Return(Return {
      loc: $1.get_loc(),
      expr: None,
    });
  }
  ;


Print
  : PRINT '(' ExprList ')' {
    |$1: Token, $3: ExprList| -> Stmt;
    $$ = Stmt::Print(Print {
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
    |$1: Expr, $2: Token, $3: Expr| -> Simple;
    $$ = Simple::Assign(Assign {
      loc: $2.get_loc(),
      dst: $1,
      src: $3,
    });
  }
  | Type IDENTIFIER '=' Expr {
    |$1: Type, $2: Token, $3: Token, $4: Expr| -> Simple;
    $$ = Simple::VarDef(VarDef {
      loc: $2.get_loc(),
      name: $2.value,
      type_: $1,
      finish_loc: self.get_loc(),
      src: Some($4),
      scope: ptr::null(),
      index: D::default(),
      offset: -1,
    });
  }
  | VAR IDENTIFIER '=' Expr {
    |$1: Token, $2: Token, $3: Token, $4: Expr| -> Simple;
    $$ = Simple::VarDef(VarDef {
      loc: $2.get_loc(),
      name: $2.value,
      type_: Type { loc: $1.get_loc(), sem: VAR },
      finish_loc: self.get_loc(),
      src: Some($4),
      scope: ptr::null(),
      index: D::default(),
      offset: -1,
    });
  }
  | VarDef {
    |$1: VarDef| -> Simple;
    $$ = Simple::VarDef($1);
  }
  | Expr {
    |$1: Expr| -> Simple;
    $$ = Simple::Expr($1);
  }
  | /* empty */ {
    || -> Simple;
    $$ = Simple::Skip;
  }
  ;

Expr
  : LValue {
    $$ = $1;
  }
  | MaybeReceiver IDENTIFIER '(' ExprListOrEmpty ')' {
    |$1: Option<Expr>, $2: Token, $4: ExprList| -> Expr;
    $$ = Expr::new($2.get_loc(), ExprData::Call(Call {
      owner: $1.map(|s| Box::new(s)),
      name: $2.value,
      arg: $4,
      is_arr_len: false,
      method: ptr::null(),
    }));
  }
  | INT_CONST {
    |$1: Token| -> Expr;
    $$ = Expr::with_type($1.get_loc(), INT, ExprData::IntConst($1.value.parse::<i32>().unwrap_or_else(|_| {
      self.errors.push(Error::new($1.get_loc(), IntTooLarge{ string: $1.value.to_string(), }));
      0
    })));
  }
  | TRUE {
    |$1: Token| -> Expr;
    $$ = Expr::with_type($1.get_loc(), BOOL, ExprData::BoolConst(true));
  }
  | FALSE {
    |$1: Token| -> Expr;
    $$ = Expr::with_type($1.get_loc(), BOOL, ExprData::BoolConst(false));
  }
  | STRING_CONST {
    || -> Expr;
    $$ = Expr::with_type(Loc(self.tokenizer.string_builder.1, self.tokenizer.string_builder.2),
        STRING, ExprData::StringConst(self.tokenizer.string_builder.0.clone()));
  }
  | '[' ExprList ']' {
    |$2: ExprList| -> Expr;
    $$ = Expr::new(self.get_loc(), ExprData::ArrayConst($1));
  }
  | NULL {
    |$1: Token| -> Expr;
    $$ = Expr::new($1.get_loc(), ExprData::Null);
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
  | Expr '&' Expr {
    |$1: Expr, $2: Token, $3: Expr| -> Expr;
    $$ = gen_binary($1, $2, $3, Operator::BAnd);
  }
  | Expr '|' Expr {
    |$1: Expr, $2: Token, $3: Expr| -> Expr;
    $$ = gen_binary($1, $2, $3, Operator::BOr);
  }
  | Expr '^' Expr {
    |$1: Expr, $2: Token, $3: Expr| -> Expr;
    $$ = gen_binary($1, $2, $3, Operator::BXor);
  }
  | Expr SHL Expr {
    |$1: Expr, $2: Token, $3: Expr| -> Expr;
    $$ = gen_binary($1, $2, $3, Operator::Shl);
  }
  | Expr SHR Expr {
    |$1: Expr, $2: Token, $3: Expr| -> Expr;
    $$ = gen_binary($1, $2, $3, Operator::Shr);
  }
  | Expr '[' Expr ':' Expr ']' {
    |$1: Expr, $2: Token, $3: Expr, $5: Expr| -> Expr;
    $$ = Expr::new($2.get_loc(),
      ExprData::Range(Range { arr: Box::new($1), lb: Box::new($3), ub: Box::new($5), }));
  }
  | Expr '[' Expr ']' DEFAULT Expr {
    |$1: Expr, $2: Token, $3: Expr, $6: Expr| -> Expr;
    $$ = Expr::new($2.get_loc(),
      ExprData::Default(Default { arr: Box::new($1), idx: Box::new($3), dft: Box::new($6), }));
  }
  | '[' Expr FOR IDENTIFIER IN Expr ']' {
    |$1: Token, $2: Expr, $4: Token, $6: Expr| -> Expr;
    $$ = Expr::new($1.get_loc(), ExprData::Comprehension(Comprehension {
      expr: Box::new($2),
      name: $4.value,
      arr: Box::new($6),
      cond: None,
    }));
  }
  | '[' Expr FOR IDENTIFIER IN Expr IF Expr ']' {
    |$1: Token, $2: Expr, $4: Token, $6: Expr, $8: Expr| -> Expr;
    $$ = Expr::new($1.get_loc(), ExprData::Comprehension(Comprehension {
      expr: Box::new($2),
      name: $4.value,
      arr: Box::new($6),
      cond: Some(Box::new($8)),
    }));
  }
  | '(' Expr ')' {
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
  | INC Expr {
    |$1: Token, $2: Expr| -> Expr;
    $$ = gen_unary($1, $2, Operator::PreInc);
  }
  | DEC Expr {
    |$1: Token, $2: Expr| -> Expr;
    $$ = gen_unary($1, $2, Operator::PreDec);
  }
  | Expr INC {
    |$1: Expr, $2: Token| -> Expr;
    $$ = gen_unary($2, $1, Operator::PostInc);
  }
  | Expr DEC {
    |$1: Expr, $2: Token| -> Expr;
    $$ = gen_unary($2, $1, Operator::PostDec);
  }
  | READ_INTEGER '(' ')' {
    |$1: Token| -> Expr;
    $$ = Expr::with_type($1.get_loc(), INT, ExprData::ReadInt);
  }
  | READ_LINE '(' ')' {
    |$1: Token| -> Expr;
    $$ = Expr::with_type($1.get_loc(), STRING, ExprData::ReadLine);
  }
  | THIS {
    |$1: Token| -> Expr;
    $$ = Expr::new($1.get_loc(), ExprData::This);
  }
  | NEW IDENTIFIER '(' ')' {
    |$1: Token, $2: Token| -> Expr;
    $$ = Expr::new($1.get_loc(), ExprData::NewClass { name: $2.value, });
  }
  | NEW Type '[' Expr ']' {
    |$1: Token, $2: Type, $4: Expr| -> Expr;
    $$ = Expr::new($1.get_loc(), ExprData::NewArray { elem_t: $2, len: Box::new($4), });
  }
  | INSTANCEOF '(' Expr ',' IDENTIFIER ')' {
    |$1: Token, $3: Expr, $5: Token| -> Expr;
    $$ = Expr::new($1.get_loc(), ExprData::TypeTest { expr: Box::new($3), name: $5.value, target_class: ptr::null() });
  }
  | '(' CLASS IDENTIFIER ')' Expr {
    |$3: Token, $5: Expr| -> Expr;
    $$ = Expr::new($5.loc, ExprData::TypeCast { name: $3.value, expr: Box::new($5), });
  }
  ;

LValue
  : MaybeReceiver IDENTIFIER {
    |$1: Option<Expr>, $2: Token| -> Expr;
    $$ = Expr::new($2.get_loc(), ExprData::Id(Id {
      owner: $1.map(|e| Box::new(e)),
      name: $2.value,
      symbol: ptr::null(),
      for_assign: D::default(),
    }));
  }
  | Expr '[' Expr ']' {
    |$1: Expr, $3: Expr| -> Expr;
    $$ = Expr::new($1.loc, ExprData::Indexed(Indexed {
      arr: Box::new($1),
      idx: Box::new($3),
      for_assign: D::default(),
    }));
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

ExprListOrEmpty
  : ExprList {
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
      name: $2.value,
      type_: $1,
      finish_loc: self.get_loc(),
      src: None,
      scope: ptr::null(),
      index: D::default(),
      offset: -1,
    };
  }
  ;
        
Type
  : INT {
    |$1: Token| -> Type;
    $$ = Type { loc: $1.get_loc(), sem: INT };
  }
  | VOID {
    |$1: Token| -> Type;
    $$ = Type { loc: $1.get_loc(), sem: VOID };
  }
  | BOOL {
    |$1: Token| -> Type;
    $$ = Type { loc: $1.get_loc(), sem: BOOL };
  }
  | STRING {
    |$1: Token| -> Type;
    $$ = Type { loc: $1.get_loc(), sem: STRING };
  }
  | CLASS IDENTIFIER  {
    |$1: Token, $2: Token| -> Type;
    $$ = Type { loc: $2.get_loc(), sem: SemanticType::Named($2.value) };
  }
  | Type '[' ']' {
    |$1: Type| -> Type;
    $$ = Type { loc: $1.loc, sem: SemanticType::Array(Box::new($1.sem)) };
  }
  ;