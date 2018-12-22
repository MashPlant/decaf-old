use super::loc::*;
use super::symbol::*;
use super::types::*;
use super::util::*;
use super::tac::VTable;

use std::default::Default as D;
use std::ptr;
use std::ops::Deref;

use llvm_sys::prelude::*;

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
  // default field_cnt is -1, for `not resolved`
  pub field_cnt: i32,
  pub v_tbl: VTable,
  pub llvm_t: LLVMTypeRef,
  pub llvm_v_tbl_t: LLVMTypeRef,
  pub llvm_v_tbl: LLVMValueRef,
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
      field_cnt: -1,
      v_tbl: VTable { class: ptr::null(), methods: Vec::new() },
      llvm_t: ptr::null_mut(),
      llvm_v_tbl_t: ptr::null_mut(),
      llvm_v_tbl: ptr::null_mut(),
    }
  }
}

impl ClassDef {
  pub fn lookup(&self, name: &'static str) -> Option<Symbol> {
    let mut class = self as *const ClassDef;
    while !class.is_null() {
      if let Some(symbol) = class.get().scope.get(name) {
        return Some(*symbol);
      }
      class = class.get().p_ptr;
    }
    None
  }

  pub fn extends(&self, other: *const ClassDef) -> bool {
    let mut class = self as *const ClassDef;
    while !class.is_null() {
      if class == other {
        return true;
      }
      class = class.get().p_ptr;
    }
    false
  }

  pub fn get_object_type(&self) -> SemanticType {
    SemanticType::Object(self)
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

#[derive(Debug)]
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
  pub class: *const ClassDef,
  // tac & llvm: the offset in v-table
  pub offset: i32,
  // POINTER of function type
  pub llvm_t: LLVMTypeRef,
  pub llvm_val: LLVMValueRef,
}

impl MethodDef {
  pub fn new(loc: Loc, name: &'static str, ret_t: Type, param: Vec<VarDef>, static_: bool, body: Block) -> MethodDef {
    MethodDef { loc, name, ret_t, param, static_, body, scope: D::default(), class: ptr::null(), offset: -1, llvm_t: ptr::null_mut(), llvm_val: ptr::null_mut() }
  }
}

// int x = 1;
// ---------^---- finish_loc
// so that int x = x will get an undeclared variable error
#[derive(Debug)]
pub struct VarDef {
  pub loc: Loc,
  pub name: &'static str,
  pub type_: Type,
  pub src: Option<Expr>,
  pub finish_loc: Loc,
  pub scope: *const Scope,
  // jvm: the index on stack, only valid for local & parameter variable
  pub jvm_index: u8,
  // tac: the offset in object OR the virtual register id
  // llvm: the offset in object
  pub offset: i32,
  pub llvm_val: LLVMValueRef,
}

impl VarDef {
  pub fn new(loc: Loc, name: &'static str, type_: Type, src: Option<Expr>, finish_loc: Loc) -> VarDef {
    VarDef { loc, name, type_, src, finish_loc, scope: ptr::null(), jvm_index: 255, offset: -1, llvm_val: ptr::null_mut() }
  }
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
  VarDef(VarDef),
  Expr(Expr),
  Skip,
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
  pub dst_sym: *const VarDef,
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Operator {
  Neg,
  Not,
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  And,
  Or,
  Eq,
  Ne,
  Lt,
  Le,
  Gt,
  Ge,
  Repeat,
  /* not implemented */ Concat,
  PreInc,
  PreDec,
  PostInc,
  PostDec,
  BAnd,
  BOr,
  BXor,
  Shl,
  Shr,
}

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
      /* not implemented */ Concat => "++",
      PreInc => "++",
      PreDec => "--",
      PostInc => "++",
      PostDec => "--",
      BAnd => "&",
      BOr => "|",
      BXor => "^",
      Shl => "<<",
      Shr => ">>",
    }
  }
}

#[derive(Debug)]
pub struct Expr {
  pub loc: Loc,
  pub type_: SemanticType,
  // virtual register id for tac code gen
  pub tac_reg: i32,
  // it is a ssa reg, not a pointer, so accessing it doesn't need LLVMBuildLoad
  pub llvm_val: LLVMValueRef,
  pub data: ExprData,
}

#[derive(Debug)]
pub enum ExprData {
  Id(Id),
  Indexed(Indexed),
  IntConst(i32),
  BoolConst(bool),
  StringConst(String),
  ArrayConst(Vec<Expr>),
  Null,
  Call(Call),
  Unary(Unary),
  Binary(Binary),
  This,
  ReadInt,
  ReadLine,
  NewClass { name: &'static str },
  NewArray {
    elem_t: Type,
    len: Box<Expr>,
  },
  TypeTest {
    expr: Box<Expr>,
    name: &'static str,
  },
  TypeCast {
    name: &'static str,
    expr: Box<Expr>,
  },
  Range(Range),
  Default(Default),
  Comprehension(Comprehension),
}

impl Expr {
  pub fn new(loc: Loc, data: ExprData) -> Expr {
    Expr { loc, type_: D::default(), tac_reg: -1, llvm_val: ptr::null_mut(), data }
  }

  pub fn with_type(loc: Loc, type_: SemanticType, data: ExprData) -> Expr {
    Expr { loc, type_, tac_reg: -1, llvm_val: ptr::null_mut(), data }
  }

  pub fn is_lvalue(&self) -> bool {
    match self.data {
      ExprData::Id(_) | ExprData::Indexed(_) => true,
      _ => false,
    }
  }
}

#[derive(Debug)]
pub struct Id {
  pub owner: Option<Box<Expr>>,
  pub name: &'static str,
  pub symbol: *const VarDef,
  pub for_assign: bool,
}

#[derive(Debug)]
pub struct Indexed {
  pub arr: Box<Expr>,
  pub idx: Box<Expr>,
  pub for_assign: bool,
}

#[derive(Debug)]
pub struct Call {
  pub owner: Option<Box<Expr>>,
  pub name: &'static str,
  pub arg: Vec<Expr>,
  pub is_arr_len: bool,
  pub method: *const MethodDef,
}

#[derive(Debug)]
pub struct Binary {
  pub op: Operator,
  pub l: Box<Expr>,
  pub r: Box<Expr>,
}

#[derive(Debug)]
pub struct Unary {
  pub op: Operator,
  pub r: Box<Expr>,
}

#[derive(Debug)]
pub struct Range {
  pub arr: Box<Expr>,
  pub lb: Box<Expr>,
  pub ub: Box<Expr>,
}

#[derive(Debug)]
pub struct Default {
  pub arr: Box<Expr>,
  pub idx: Box<Expr>,
  pub dft: Box<Expr>,
}

#[derive(Debug)]
pub struct Comprehension {
  pub expr: Box<Expr>,
  pub name: &'static str,
  pub arr: Box<Expr>,
  pub cond: Option<Box<Expr>>,
}