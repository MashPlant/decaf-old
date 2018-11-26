use super::loc::*;
use super::symbol::*;
use super::types::*;
use std::default::Default as D;
use std::ptr;
use std::ops::Deref;

#[derive(Debug)]
pub struct Program {
  pub class: Vec<ClassDef>,
  pub scope: Scope,
  pub main: *const ClassDef,
}

impl D for Program {
  fn default() -> Self {
    Program {
      class: D::default(),
      scope: Scope { symbols: D::default(), kind: ScopeKind::Global },
      main: ptr::null(),
    }
  }
}

#[derive(Debug)]
pub struct ClassDef {
  // syntax part
  pub loc: Loc,
  pub name: &'static str,
  pub parent: Option<&'static str>,
  pub field: Vec<FieldDef>,
  pub sealed: bool,
  // semantic part
  // to calculate inheritance order and determine cyclic inheritance
  pub order: i32,
  // to avoid duplicate override check
  pub checked: bool,
  pub p_ptr: *mut ClassDef,
  pub scope: Scope,
}

impl D for ClassDef {
  fn default() -> Self {
    ClassDef {
      loc: D::default(),
      name: D::default(),
      parent: D::default(),
      field: D::default(),
      sealed: D::default(),
      order: -1,
      checked: D::default(),
      p_ptr: ptr::null_mut(),
      scope: D::default(),
    }
  }
}

impl ClassDef {
  pub fn lookup(&self, name: &'static str) -> Option<Symbol> {
    unsafe {
      let mut class = self as *const ClassDef;
      while !class.is_null() {
        if let Some(symbol) = (*class).scope.get(name) {
          return Some(*symbol);
        }
        class = (*class).p_ptr;
      }
      None
    }
  }

  pub fn extends(&self, other: *const ClassDef) -> bool {
    let mut class = self as *const ClassDef;
    while !class.is_null() {
      if class == other {
        return true;
      }
      class = unsafe { (*class).p_ptr };
    }
    false
  }

  pub fn get_class_type(&self) -> SemanticType {
    SemanticType::Class(self)
  }

  pub fn get_object_type(&self) -> SemanticType {
    SemanticType::Object(self.name, self)
  }
}

#[derive(Debug)]
pub enum FieldDef {
  MethodDef(MethodDef),
  VarDef(VarDef),
}

impl FieldDef {
  pub fn get_loc(&self) -> Loc {
    match self {
      FieldDef::MethodDef(method_def) => method_def.loc,
      FieldDef::VarDef(var_def) => var_def.loc,
    }
  }
}

#[derive(Debug, Default)]
pub struct MethodDef {
  pub loc: Loc,
  pub name: &'static str,
  pub ret_t: Type,
  pub param: Vec<VarDef>,
  pub static_: bool,
  // body contains the scope of stack variables
  pub body: Block,
  // scope for parameters
  pub scope: Scope,
}

#[derive(Debug)]
pub struct VarDef {
  pub loc: Loc,
  pub name: &'static str,
  pub type_: Type,
  pub scope: *const Scope,
}

#[derive(Debug, Default)]
pub struct Type {
  pub loc: Loc,
  pub sem: SemanticType,
}

impl Deref for Type {
  type Target = SemanticType;

  fn deref(&self) -> &SemanticType {
    &self.sem
  }
}

#[derive(Debug)]
pub enum Stmt {
  VarDef(VarDef),
  Simple(Simple),
  If(If),
  While(While),
  For(For),
  Return(Return),
  Print(Print),
  Break(Break),
  SCopy(SCopy),
  Foreach(Foreach),
  Guarded(Guarded),
  Block(Block),
}

#[derive(Debug)]
pub enum Simple {
  Assign(Assign),
  VarAssign(VarAssign),
  Expr(Expr),
  Skip(Skip),
}

#[derive(Debug, Default)]
pub struct Block {
  // syntax part
  pub loc: Loc,
  pub stmt: Vec<Stmt>,
  // semantic part
  pub is_method: bool,
  pub scope: Scope,
}

#[derive(Debug)]
pub struct If {
  pub loc: Loc,
  pub cond: Expr,
  pub on_true: Block,
  pub on_false: Option<Block>,
}

#[derive(Debug)]
pub struct While {
  pub loc: Loc,
  pub cond: Expr,
  pub body: Block,
}

#[derive(Debug)]
pub struct For {
  pub loc: Loc,
  // Skip for no init or update
  pub init: Simple,
  pub cond: Expr,
  pub update: Simple,
  pub body: Block,
}

#[derive(Debug)]
pub struct Foreach {
  pub def: VarDef,
  pub arr: Expr,
  pub cond: Option<Expr>,
  pub body: Block,
}

#[derive(Debug)]
pub struct Return {
  pub loc: Loc,
  pub expr: Option<Expr>,
}

#[derive(Debug)]
pub struct Print {
  pub loc: Loc,
  pub print: Vec<Expr>,
}

#[derive(Debug)]
pub struct Break {
  pub loc: Loc,
}

#[derive(Debug)]
pub struct SCopy {
  pub loc: Loc,
  pub dst_loc: Loc,
  pub dst: &'static str,
  pub src: Expr,
}

#[derive(Debug)]
pub struct Guarded {
  pub loc: Loc,
  pub guarded: Vec<(Expr, Block)>,
}

#[derive(Debug)]
pub struct Assign {
  pub loc: Loc,
  pub dst: Expr,
  pub src: Expr,
}

#[derive(Debug)]
pub struct VarAssign {
  pub loc: Loc,
  pub name: &'static str,
  pub src: Expr,
  pub scope: *const Scope,
  // determined during type check
  pub type_: SemanticType,
}

#[derive(Debug)]
pub struct Skip {
  pub loc: Loc,
}

#[derive(Debug, Copy, Clone)]
pub enum Operator { Neg, Not, Add, Sub, Mul, Div, Mod, And, Or, Eq, Ne, Lt, Le, Gt, Ge, Repeat, Concat }

impl Operator {
  pub fn to_str(&self) -> &'static str {
    use self::Operator::*;
    match self {
      Neg => "-",
      Not => "!",
      Add => "+",
      Sub => "-",
      Mul => "*",
      Div => "/",
      Mod => "%",
      And => "&&",
      Or => "||",
      Eq => "==",
      Ne => "!=",
      Lt => "<",
      Le => "<=",
      Gt => ">",
      Ge => ">=",
      Repeat => "%%",
      Concat => "++"
    }
  }
}

#[derive(Debug)]
pub enum Expr {
  Identifier(Identifier),
  Indexed(Indexed),
  Const(Const),
  Call(Call),
  Unary(Unary),
  Binary(Binary),
  This(This),
  ReadInt(ReadInt),
  ReadLine(ReadLine),
  NewClass(NewClass),
  NewArray(NewArray),
  TypeTest(TypeTest),
  TypeCast(TypeCast),
  Range(Range),
  Default(Default),
  Comprehension(Comprehension),
}

impl Expr {
  pub fn get_loc(&self) -> Loc {
    use self::Expr::*;
    match &self {
      Identifier(identifier) => identifier.loc,
      Indexed(indexed) => indexed.loc,
      Const(const_) => const_.get_loc(),
      Call(call) => call.loc,
      Unary(unary) => unary.loc,
      Binary(binary) => binary.loc,
      This(this) => this.loc,
      ReadInt(read_int) => read_int.loc,
      ReadLine(read_line) => read_line.loc,
      NewClass(new_class) => new_class.loc,
      NewArray(new_array) => new_array.loc,
      TypeTest(type_test) => type_test.loc,
      TypeCast(type_cast) => type_cast.loc,
      Range(range) => range.loc,
      Default(default) => default.loc,
      Comprehension(comprehension) => comprehension.loc,
    }
  }

  pub fn get_type(&self) -> &SemanticType {
    use self::Expr::*;
    match &self {
      Identifier(identifier) => &identifier.type_,
      Indexed(indexed) => &indexed.type_,
      Const(const_) => const_.get_type(),
      Call(call) => &call.type_,
      Unary(unary) => &unary.type_,
      Binary(binary) => &binary.type_,
      This(this) => &this.type_,
      ReadInt(_) => &INT,
      ReadLine(_) => &STRING,
      NewClass(new_class) => &new_class.type_,
      NewArray(new_array) => &new_array.type_,
      TypeTest(_) => &BOOL,
      TypeCast(type_cast) => &type_cast.type_,
      Range(range) => &range.type_,
      Default(default) => &default.type_,
      Comprehension(comprehension) => &comprehension.type_,
    }
  }
}

#[derive(Debug)]
pub struct Indexed {
  pub loc: Loc,
  pub arr: Box<Expr>,
  pub idx: Box<Expr>,
  pub type_: SemanticType,
}

#[derive(Debug, Default)]
pub struct Identifier {
  pub loc: Loc,
  pub owner: Option<Box<Expr>>,
  pub name: &'static str,
  pub type_: SemanticType,
}

#[derive(Debug)]
pub enum Const {
  IntConst(IntConst),
  BoolConst(BoolConst),
  StringConst(StringConst),
  ArrayConst(ArrayConst),
  Null(Null),
}

impl Const {
  pub fn get_loc(&self) -> Loc {
    use self::Const::*;
    match self {
      IntConst(int_const) => int_const.loc,
      BoolConst(bool_const) => bool_const.loc,
      StringConst(string_const) => string_const.loc,
      ArrayConst(array_const) => array_const.loc,
      Null(null) => null.loc,
    }
  }

  pub fn get_type(&self) -> &SemanticType {
    use self::Const::*;
    match self {
      IntConst(_) => &INT,
      BoolConst(_) => &BOOL,
      StringConst(_) => &STRING,
      ArrayConst(array_const) => &array_const.type_,
      Null(_) => &NULL,
    }
  }
}

#[derive(Debug)]
pub struct IntConst {
  pub loc: Loc,
  pub value: i32,
}

#[derive(Debug)]
pub struct BoolConst {
  pub loc: Loc,
  pub value: bool,
}

#[derive(Debug)]
pub struct StringConst {
  pub loc: Loc,
  pub value: String,
}

#[derive(Debug)]
pub struct ArrayConst {
  pub loc: Loc,
  pub value: Vec<Const>,
  pub type_: SemanticType,
}

#[derive(Debug)]
pub struct Null {
  pub loc: Loc,
}

#[derive(Debug)]
pub struct Call {
  pub loc: Loc,
  pub owner: Option<Box<Expr>>,
  pub name: &'static str,
  pub arg: Vec<Expr>,
  pub type_: SemanticType,
  pub method: *const MethodDef,
}

#[derive(Debug)]
pub struct Unary {
  pub loc: Loc,
  pub op: Operator,
  pub r: Box<Expr>,
  pub type_: SemanticType,
}

#[derive(Debug)]
pub struct Binary {
  pub loc: Loc,
  pub op: Operator,
  pub l: Box<Expr>,
  pub r: Box<Expr>,
  pub type_: SemanticType,
}

#[derive(Debug)]
pub struct This {
  pub loc: Loc,
  pub type_: SemanticType,
}

#[derive(Debug)]
pub struct ReadInt {
  pub loc: Loc,
}

#[derive(Debug)]
pub struct ReadLine {
  pub loc: Loc,
}

#[derive(Debug)]
pub struct NewClass {
  pub loc: Loc,
  pub name: &'static str,
  pub type_: SemanticType,
}

#[derive(Debug)]
pub struct NewArray {
  pub loc: Loc,
  pub elem_t: Type,
  pub len: Box<Expr>,
  pub type_: SemanticType,
}

#[derive(Debug)]
pub struct TypeTest {
  pub loc: Loc,
  pub expr: Box<Expr>,
  pub name: &'static str,
}

#[derive(Debug)]
pub struct TypeCast {
  pub loc: Loc,
  pub name: &'static str,
  pub expr: Box<Expr>,
  pub type_: SemanticType,
}


#[derive(Debug)]
pub struct Range {
  pub loc: Loc,
  pub arr: Box<Expr>,
  pub lb: Box<Expr>,
  pub ub: Box<Expr>,
  pub type_: SemanticType,
}

#[derive(Debug)]
pub struct Default {
  pub loc: Loc,
  pub arr: Box<Expr>,
  pub idx: Box<Expr>,
  pub dft: Box<Expr>,
  pub type_: SemanticType,
}

#[derive(Debug)]
pub struct Comprehension {
  pub loc: Loc,
  pub expr: Box<Expr>,
  pub name: &'static str,
  pub arr: Box<Expr>,
  pub cond: Option<Box<Expr>>,
  pub type_: SemanticType,
}

pub trait Visitor {
  fn visit_program(&mut self, _program: &mut Program) {}

  fn visit_class_def(&mut self, _class_def: &mut ClassDef) {}

  fn visit_method_def(&mut self, _method_def: &mut MethodDef) {}

  fn visit_field_def(&mut self, field_def: &mut FieldDef) {
    match field_def {
      FieldDef::MethodDef(method_def) => self.visit_method_def(method_def),
      FieldDef::VarDef(var_def) => self.visit_var_def(var_def),
    };
  }

  fn visit_stmt(&mut self, stmt: &mut Stmt) {
    use self::Stmt::*;
    match stmt {
      VarDef(var_def) => self.visit_var_def(var_def),
      Simple(simple) => self.visit_simple(simple),
      If(if_) => self.visit_if(if_),
      While(while_) => self.visit_while(while_),
      For(for_) => self.visit_for(for_),
      Return(return_) => self.visit_return(return_),
      Print(print) => self.visit_print(print),
      Break(break_) => self.visit_break(break_),
      SCopy(s_copy) => self.visit_s_copy(s_copy),
      Foreach(foreach) => self.visit_foreach(foreach),
      Guarded(guarded) => self.visit_guarded(guarded),
      Block(block) => self.visit_block(block),
    };
  }

  fn visit_simple(&mut self, simple: &mut Simple) {
    match simple {
      Simple::Assign(assign) => self.visit_assign(assign),
      Simple::VarAssign(var_assign) => self.visit_var_assign(var_assign),
      Simple::Expr(expr) => self.visit_expr(expr),
      Simple::Skip(skip) => self.visit_skip(skip),
    }
  }

  fn visit_var_def(&mut self, _var_def: &mut VarDef) {}

  fn visit_var_assign(&mut self, _var_assign: &mut VarAssign) {}

  fn visit_skip(&mut self, _skip: &mut Skip) {}

  fn visit_block(&mut self, _block: &mut Block) {}

  fn visit_while(&mut self, _while: &mut While) {}

  fn visit_for(&mut self, _for: &mut For) {}

  fn visit_if(&mut self, _if: &mut If) {}

  fn visit_break(&mut self, _break: &mut Break) {}

  fn visit_return(&mut self, _return: &mut Return) {}

  fn visit_s_copy(&mut self, _s_copy: &mut SCopy) {}

  fn visit_foreach(&mut self, _foreach: &mut Foreach) {}

  fn visit_guarded(&mut self, _guarded: &mut Guarded) {}

  fn visit_new_class(&mut self, _new_class: &mut NewClass) {}

  fn visit_new_array(&mut self, _new_array: &mut NewArray) {}

  fn visit_assign(&mut self, _assign: &mut Assign) {}

  fn visit_expr(&mut self, expr: &mut Expr) {
    use self::Expr::*;
    match expr {
      Identifier(identifier) => self.visit_identifier(identifier),
      Indexed(indexed) => self.visit_indexed(indexed),
      Const(const_) => self.visit_const(const_),
      Call(call) => self.visit_call(call),
      Unary(unary) => self.visit_unary(unary),
      Binary(binary) => self.visit_binary(binary),
      This(this) => self.visit_this(this),
      ReadInt(read_int) => self.visit_read_int(read_int),
      ReadLine(read_line) => self.visit_read_line(read_line),
      NewClass(new_class) => self.visit_new_class(new_class),
      NewArray(new_array) => self.visit_new_array(new_array),
      TypeTest(type_test) => self.visit_type_test(type_test),
      TypeCast(type_cast) => self.visit_type_cast(type_cast),
      Range(range) => self.visit_range(range),
      Default(default) => self.visit_default(default),
      Comprehension(comprehension) => self.visit_comprehension(comprehension),
    };
  }

  fn visit_const(&mut self, _const_: &mut Const) {}

  fn visit_unary(&mut self, _unary: &mut Unary) {}

  fn visit_binary(&mut self, _binary: &mut Binary) {}

  fn visit_call(&mut self, _call: &mut Call) {}

  fn visit_read_int(&mut self, _read_int: &mut ReadInt) {}

  fn visit_read_line(&mut self, _read_line: &mut ReadLine) {}

  fn visit_print(&mut self, _print: &mut Print) {}

  fn visit_this(&mut self, _this: &mut This) {}

  fn visit_type_cast(&mut self, _type_cast: &mut TypeCast) {}

  fn visit_type_test(&mut self, _type_test: &mut TypeTest) {}

  fn visit_indexed(&mut self, _indexed: &mut Indexed) {}

  fn visit_identifier(&mut self, _identifier: &mut Identifier) {}

  fn visit_range(&mut self, _range: &mut Range) {}

  fn visit_default(&mut self, _default: &mut Default) {}

  fn visit_comprehension(&mut self, _comprehension: &mut Comprehension) {}

  fn visit_type(&mut self, _type: &mut Type) {}
}