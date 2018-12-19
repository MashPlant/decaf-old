use super::ast::*;
use super::types::*;
use super::util::*;
use super::tac_code_gen::resolve_field_order;

use llvm_sys::prelude::*;
use llvm_sys::support::*;
use llvm_sys::core::*;
use llvm_sys::execution_engine::*;
use llvm_sys::target::*;

use std::ffi::{CStr, CString};
use std::ptr;

macro_rules! cstr {
  ($s: expr) => { concat!($s, "\0") as *const str as *const [libc::c_char] as *const libc::c_char }
}

macro_rules! cstring {
  ($s: expr) => { CString::new($s).unwrap().as_ptr() }
}

const T: *const libc::c_char = cstr!("");

pub struct LLVMContextWrapper {
  inner: LLVMContextRef,
}

impl Drop for LLVMContextWrapper {
  fn drop(&mut self) {
    unsafe {
      LLVMContextDispose(self.inner);
    }
  }
}

pub struct LLVMCodeGen {
  context: LLVMContextRef,
  module: LLVMModuleRef,
  builder: LLVMBuilderRef,
  i32_t: LLVMTypeRef,
  i8_t: LLVMTypeRef,
  void_t: LLVMTypeRef,
  str_t: LLVMTypeRef,
}

impl LLVMCodeGen {
  pub fn gen(mut program: Program) -> LLVMContextWrapper {
    unimplemented!()
  }
}

unsafe fn ptr_of(type_: LLVMTypeRef) -> LLVMTypeRef {
  LLVMPointerType(type_, 0)
}

impl LLVMCodeGen {
  unsafe fn type_of(&self, type_: &SemanticType) -> LLVMTypeRef {
    match type_ {
      SemanticType::Int => self.i32_t,
      SemanticType::Bool => self.i8_t,
      SemanticType::Void => self.void_t,
      SemanticType::String => self.str_t,
      SemanticType::Object(class) => class.get().llvm_t,
      SemanticType::Array(elem) => ptr_of(self.type_of(elem)),
      _ => unreachable!(),
    }
  }

  unsafe fn type_of_method(&self, method: &MethodDef) -> LLVMTypeRef {
    let mut param_t = method.param.iter().map(|a| self.type_of(&a.type_)).collect::<Vec<_>>();
    LLVMFunctionType(self.type_of(&method.ret_t), param_t.as_mut_ptr(), param_t.len() as u32, 0)
  }

  unsafe fn program(&mut self, program: &mut Program) {
    let context = LLVMContextCreate();
    let module = LLVMModuleCreateWithNameInContext(cstr!("Decaf Program"), context);
    let builder = LLVMCreateBuilderInContext(context);
    (self.context = context, self.module = module, self.builder = builder);
    (self.i32_t = LLVMInt32TypeInContext(context), self.i8_t = LLVMInt8TypeInContext(context), self.void_t = LLVMVoidTypeInContext(context), self.str_t = ptr_of(self.i8_t));

    for class in &mut program.class {
      resolve_field_order(class);
      let struct_t = LLVMStructCreateNamed(context, cstring!(class.name));
      let v_tbl_t = LLVMStructCreateNamed(context, cstring!(format!("{}_VTableT", class.name)));
      let mut elem_t = vec![v_tbl_t];
      for field in &mut class.field {
        match field {
          FieldDef::VarDef(var) => {
            elem_t.push(self.type_of(&var.type_));
          }
          FieldDef::MethodDef(method) => {
            self.method(method);
          }
        }
      }
      // actually first element is not string, just a meaningless i8*
      let mut v_tbl_elem_t = vec![if class.p_ptr.is_null() { self.str_t } else { class.p_ptr.get().llvm_t }, self.str_t];
      let mut v_tbl_elem = vec![if class.p_ptr.is_null() { LLVMConstNull(self.str_t) } else { class.p_ptr.get().name }, self.str_t]
      for method in &class.v_tbl.methods {
        v_tbl_elem_t.push(self.type_of_method(method.get()));
      }
      LLVMStructSetBody(struct_t, elem_t.as_mut_ptr(), elem_t.len() as u32, 0);
      LLVMStructSetBody(v_tbl_t, v_tbl_elem_t.as_mut_ptr(), v_tbl_elem_t.len() as u32, 0);
      let v_tbl = LLVMAddGlobal(module, v_tbl_t, cstring!(format!("{}_VTableT", class.name)));
    }
    LLVMDisposeBuilder(builder);
    LLVMDumpModule(module);
  }

  unsafe fn method(&mut self, method: &mut MethodDef) {
    unimplemented!();
  }
}