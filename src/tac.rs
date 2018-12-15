use super::ast::*;
use super::print::quote;

use std::ptr;
use std::fmt;
use std::default::Default as D;

pub const INT_SIZE: i32 = 4;

#[derive(Debug, Clone)]
pub struct VTable {
  pub class: *const ClassDef,
  pub methods: Vec<*const MethodDef>,
}

#[derive(Debug)]
pub struct TacMethod {
  pub name: String,
  pub memo: String,
  pub code: Vec<Tac>,
  pub method: *const MethodDef,
}

impl D for TacMethod {
  fn default() -> Self {
    TacMethod {
      name: D::default(),
      memo: D::default(),
      code: D::default(),
      method: ptr::null(),
    }
  }
}

pub struct TacProgram {
  pub v_tables: Vec<VTable>,
  pub methods: Vec<TacMethod>,
  // there maybe labels / temps in the future
}

#[derive(Debug)]
pub enum Tac {
  Add(i32, i32, i32),
  Sub(i32, i32, i32),
  Mul(i32, i32, i32),
  Div(i32, i32, i32),
  Mod(i32, i32, i32),
  And(i32, i32, i32),
  Or(i32, i32, i32),
  Gt(i32, i32, i32),
  Ge(i32, i32, i32),
  Lt(i32, i32, i32),
  Le(i32, i32, i32),
  Eq(i32, i32, i32),
  Ne(i32, i32, i32),
  Neg(i32, i32),
  Not(i32, i32),
  Assign(i32, i32),
  LoadVTbl(i32, &'static str),
  IndirectCall(i32, i32),
  DirectCall(i32, String),
  Ret(i32),
  Jmp(i32),
  Je(i32, i32),
  Jne(i32, i32),
  // offset is int literal, not a virtual register
  Load { dst: i32, base: i32, offset: i32 },
  Store { base: i32, offset: i32, src: i32 },
  IntConst(i32, i32),
  StrConst(i32, String),
  Label(i32),
  Param(i32),
}

impl fmt::Display for Tac {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    use self::Tac::*;
    match self {
      Add(dst, l, r) => write!(f, "_T{} = (_T{} + _T{})", dst, l, r),
      Sub(dst, l, r) => write!(f, "_T{} = (_T{} - _T{})", dst, l, r),
      Mul(dst, l, r) => write!(f, "_T{} = (_T{} * _T{})", dst, l, r),
      Div(dst, l, r) => write!(f, "_T{} = (_T{} / _T{})", dst, l, r),
      Mod(dst, l, r) => write!(f, "_T{} = (_T{} % _T{})", dst, l, r),
      And(dst, l, r) => write!(f, "_T{} = (_T{} && _T{})", dst, l, r),
      Or(dst, l, r) => write!(f, "_T{} = (_T{} || _T{})", dst, l, r),
      Gt(dst, l, r) => write!(f, "_T{} = (_T{} > _T{})", dst, l, r),
      Ge(dst, l, r) => write!(f, "_T{} = (_T{} >= _T{})", dst, l, r),
      Lt(dst, l, r) => write!(f, "_T{} = (_T{} < _T{})", dst, l, r),
      Le(dst, l, r) => write!(f, "_T{} = (_T{} <= _T{})", dst, l, r),
      Eq(dst, l, r) => write!(f, "_T{} = (_T{} == _T{})", dst, l, r),
      Ne(dst, l, r) => write!(f, "_T{} = (_T{} != _T{})", dst, l, r),
      Neg(dst, r) => write!(f, "_T{} = - _T{} ", dst, r),
      Not(dst, r) => write!(f, "_T{} = ! _T{} ", dst, r),
      Assign(dst, r) => write!(f, "_T{} =  _T{} ", dst, r),
      LoadVTbl(dst, class_name) => write!(f, "_T{} = VTBL <_{}>", dst, class_name),
      IndirectCall(dst, func) => if *dst == -1 { write!(f, "call _T{}", func) } else { write!(f, "_T{} = call _T{}", dst, func) },
      DirectCall(dst, func) => if *dst == -1 { write!(f, "call {}", func) } else { write!(f, "_T{} = call {}", dst, func) },
      Ret(src) => if *src == -1 { write!(f, "return <empty>") } else { write!(f, "return _T{}", src) },
      Jmp(target) => write!(f, "branch _L{}", target),
      Je(cond, target) => write!(f, "if ({} == 0) branch _L{}", cond, target),
      Jne(cond, target) => write!(f, "if ({} != 0) branch _L{}", cond, target),
      Load { dst, base, offset } => if *offset > 0 { write!(f, "_T{} = *({} + {})", dst, base, offset) } else { write!(f, "{} = *({} - {})", dst, base, -offset) },
      Store { base, offset, src } => if *offset > 0 { write!(f, "*({} + {}) = _T{}", base, offset, src) } else { write!(f, "*({} - {}) = {}", base, -offset, src) },
      IntConst(dst, src) => write!(f, "_T{} = {}", dst, src),
      StrConst(dst, src) => write!(f, "_T{} = {}", dst, quote(src)),
      Label(label) => write!(f, "_L{}", label),
      Param(src) => write!(f, "parm _T{}", src),
    }
  }
}

#[derive(Debug, Copy, Clone)]
pub struct IntrinsicCall {
  pub name: &'static str,
  pub ret: bool,
}

pub const ALLOCATE: IntrinsicCall = IntrinsicCall { name: "_Alloc", ret: true };
pub const READ_LINE: IntrinsicCall = IntrinsicCall { name: "_ReadLine", ret: true };
pub const READ_INT: IntrinsicCall = IntrinsicCall { name: "_ReadInteger", ret: true };
pub const STRING_EQUAL: IntrinsicCall = IntrinsicCall { name: "_StringEqual", ret: true };
pub const PRINT_INT: IntrinsicCall = IntrinsicCall { name: "_PrintInt", ret: false };
pub const PRINT_STRING: IntrinsicCall = IntrinsicCall { name: "_PrintString", ret: false };
pub const PRINT_BOOL: IntrinsicCall = IntrinsicCall { name: "_PrintBool", ret: false };
pub const HALT: IntrinsicCall = IntrinsicCall { name: "_Halt", ret: false };

pub const ARRAY_INDEX_OUT_OF_BOUND: &'static str = "Decaf runtime error: Array subscript out of bounds\n";
pub const NEGATIVE_ARR_SIZE: &'static str = "Decaf runtime error: Cannot create negative-sized array\n";
pub const CLASS_CAST1: &'static str = "Decaf runtime error: ";
pub const CLASS_CAST2: &'static str = " cannot be cast to ";
pub const CLASS_CAST3: &'static str = "\n";
pub const DIV_0: &'static str = "Decaf runtime error: Division by zero error.\n";
pub const REPEAT_NEG: &'static str = "Decaf runtime error: The length of the created array should not be less than 0.\n";
