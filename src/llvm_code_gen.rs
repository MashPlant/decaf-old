use super::ast::*;
use super::types::*;
use super::util::*;
use super::tac_code_gen::resolve_field_order;

use llvm_sys::prelude::*;
use llvm_sys::support::*;
use llvm_sys::core::*;
use llvm_sys::execution_engine::*;
use llvm_sys::target::*;

use std::ffi::CString;
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
  pub fn gen(mut program: Program) {
    let mut code_gen = LLVMCodeGen {
      context: ptr::null_mut(),
      module: ptr::null_mut(),
      builder: ptr::null_mut(),
      i32_t: ptr::null_mut(),
      i8_t: ptr::null_mut(),
      void_t: ptr::null_mut(),
      str_t: ptr::null_mut(),
    };
    unsafe {
      code_gen.program(&mut program);
    }
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
      SemanticType::Object(class) => ptr_of(class.get().llvm_t),
      SemanticType::Array(elem) => ptr_of(self.type_of(elem)),
      _ => unreachable!(),
    }
  }

  // declare & define struct type
  // declare v table type & val
  unsafe fn make_struct_type(&self, class: &mut ClassDef) {
    if !class.llvm_t.is_null() { return; }
    if !class.p_ptr.is_null() { self.make_struct_type(class.p_ptr.get()); }
    class.llvm_t = LLVMStructCreateNamed(self.context, cstring!(class.name));
    class.llvm_v_tbl_t = LLVMStructCreateNamed(self.context, cstring!(format!("{}_VTableT", class.name)));
    let mut elem_t;
    if class.p_ptr.is_null() {
      elem_t = vec![ptr_of(class.llvm_v_tbl_t)];
    } else {
      elem_t = Vec::with_capacity(class.field_cnt as usize + 1);
      elem_t.set_len(class.p_ptr.get().field_cnt as usize + 1);
      LLVMGetStructElementTypes(class.p_ptr.get().llvm_t, elem_t.as_mut_ptr());
      elem_t[0] = ptr_of(class.llvm_v_tbl_t);
    }
    for field in &mut class.field {
      if let FieldDef::VarDef(var) = field {
        elem_t.push(self.type_of(&var.type_));
      }
    }
    LLVMStructSetBody(class.llvm_t, elem_t.as_mut_ptr(), elem_t.len() as u32, 0);
    class.llvm_v_tbl = LLVMAddGlobal(self.module, class.llvm_v_tbl_t, cstring!(format!("{}_VTable", class.name)));
  }

  unsafe fn program(&mut self, program: &mut Program) {
    let context = LLVMContextCreate();
    let module = LLVMModuleCreateWithNameInContext(cstr!("Decaf Program"), context);
    let builder = LLVMCreateBuilderInContext(context);
    (self.context = context, self.module = module, self.builder = builder);
    (self.i32_t = LLVMInt32TypeInContext(context), self.i8_t = LLVMInt8TypeInContext(context), self.void_t = LLVMVoidTypeInContext(context), self.str_t = ptr_of(self.i8_t));
    // build struct type
    for class in &mut program.class {
      resolve_field_order(class);
      self.make_struct_type(class);
    }
    // parsing method
    for class in &mut program.class {
      for field in &mut class.field {
        if let FieldDef::MethodDef(method) = field {
          self.method(method);
        }
      }
    }
    // build v table type & val, type & val of method is set above
    // class.llvm_v_tbl_t & class.llvm_v_tbl is declared above, but not defined
    for class in &mut program.class {
      // actually first element is not string, just a meaningless i8*
      let mut v_tbl_elem_t = vec![if class.p_ptr.is_null() { self.str_t } else { ptr_of(class.p_ptr.get().llvm_v_tbl_t) }, self.str_t];
      for method in &class.v_tbl.methods {
        v_tbl_elem_t.push(method.get().llvm_t);
      }
      LLVMStructSetBody(class.llvm_v_tbl_t, v_tbl_elem_t.as_mut_ptr(), v_tbl_elem_t.len() as u32, 0);
    }
    LLVMDisposeBuilder(builder);
    LLVMDumpModule(module);
  }

  unsafe fn method(&mut self, method: &mut MethodDef) {
    method.llvm_t = ptr_of({
      let mut param_t = method.param.iter().map(|a| self.type_of(&a.type_)).collect::<Vec<_>>();
      LLVMFunctionType(self.type_of(&method.ret_t), param_t.as_mut_ptr(), param_t.len() as u32, 0)
    });
    method.llvm_val = LLVMAddFunction(self.module, cstring!(format!("{}_{}", method.class.get().name, method.name)), method.llvm_t);

//    let child_ctor_t = LLVMFunctionType(void, [LLVMPointerType(child_t, 0)].as_mut_ptr() as *mut _, 1, 0);
//    child_ctor = LLVMAddFunction(module, b"Child_Constructor\0".as_ptr() as *const _, child_ctor_t);
//    let bb = LLVMAppendBasicBlockInContext(context, child_ctor, b"entry\0".as_ptr() as *const _);
//    LLVMPositionBuilderAtEnd(builder, bb);
//    let p = LLVMBuildStructGEP(builder, LLVMGetParam(child_ctor, 0), 0, T);
//    // now p is base
//    let p = LLVMBuildStructGEP(builder, p, 0, T);
//    // now p is v table slot
//    LLVMBuildStore(builder, LLVMBuildBitCast(builder, child_v_table, LLVMPointerType(base_v_table_t, 0), T), p);
//    LLVMBuildRetVoid(builder);
  }
}