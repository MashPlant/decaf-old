use super::ast::*;
use super::types::*;

use llvm_sys::prelude::*;
use llvm_sys::support::*;
use llvm_sys::core::*;
use llvm_sys::execution_engine::*;
use llvm_sys::target::*;

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

pub struct LLVMCodeGen {}

impl LLVMCodeGen {
  pub fn gen(mut program: Program) -> LLVMContextWrapper {
    unimplemented!()
  }
}