use super::ast::*;
use super::types::*;
use super::symbol::*;
use super::util::*;
use super::tac_code_gen::resolve_field_order;

use llvm_sys::*;
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

pub struct LLVMCodeGen {
  context: LLVMContextRef,
  module: LLVMModuleRef,
  builder: LLVMBuilderRef,
  i32_t: LLVMTypeRef,
  i8_t: LLVMTypeRef,
  void_t: LLVMTypeRef,
  str_t: LLVMTypeRef,
  i64_t: LLVMTypeRef,
  i32_0: LLVMValueRef,
  cur_this: LLVMValueRef,
  // c std library functions(malloc can be built without declare)
  printf: LLVMValueRef,
  scanf: LLVMValueRef,
  strcmp: LLVMValueRef,
  memset: LLVMValueRef,
}

impl LLVMCodeGen {
  pub fn gen(mut program: Program) {
    unsafe {
      let context = LLVMContextCreate();
      let module = LLVMModuleCreateWithNameInContext(cstr!("Decaf Program"), context);
      let builder = LLVMCreateBuilderInContext(context);
      let i32_t = LLVMInt32TypeInContext(context);
      let i8_t = LLVMInt8TypeInContext(context);
      let void_t = LLVMVoidTypeInContext(context);
      let str_t = ptr_of(i8_t);
      let i64_t = LLVMInt64TypeInContext(context);
      let i32_0 = LLVMConstInt(i32_t, 0, 0);
      let printf = LLVMAddFunction(module, cstr!("printf"), LLVMFunctionType(i32_t, [str_t].as_mut_ptr(), 1, 1));
      let scanf = LLVMAddFunction(module, cstr!("scanf"), LLVMFunctionType(i32_t, [str_t].as_mut_ptr(), 1, 1));
      let strcmp = LLVMAddFunction(module, cstr!("strcmp"), LLVMFunctionType(i32_t, [str_t, str_t].as_mut_ptr(), 2, 0));
      let memset = LLVMAddFunction(module, cstr!("memset"), LLVMFunctionType(str_t, [str_t, i32_t, i64_t].as_mut_ptr(), 3, 0));
      let mut code_gen = LLVMCodeGen { context, module, builder, i32_t, i8_t, void_t, str_t, i64_t, i32_0, cur_this: ptr::null_mut(), printf, scanf, strcmp, memset };
      code_gen.program(&mut program);
      LLVMDisposeBuilder(code_gen.builder);
      LLVMDumpModule(code_gen.module);
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
  }

  unsafe fn method(&mut self, method: &mut MethodDef) {
    let (context, builder) = (self.context, self.builder);
    method.llvm_t = ptr_of({
      let mut param_t = method.param.iter().map(|a| self.type_of(&a.type_)).collect::<Vec<_>>();
      LLVMFunctionType(self.type_of(&method.ret_t), param_t.as_mut_ptr(), param_t.len() as u32, 0)
    });
    method.llvm_val = LLVMAddFunction(self.module, cstring!(format!("{}_{}", method.class.get().name, method.name)), method.llvm_t);
    let bb = LLVMAppendBasicBlockInContext(context, method.llvm_val, cstr!("entry"));
    LLVMPositionBuilderAtEnd(builder, bb);
    for (index, param) in method.param.iter_mut().enumerate() {
      param.llvm_val = LLVMGetParam(method.llvm_val, index as u32);
    }
    if !method.static_ {
      self.cur_this = method.param[0].llvm_val;
    }
    self.block(&mut method.body);
    match &method.ret_t.sem {
      SemanticType::Int => LLVMBuildRet(builder, LLVMConstInt(self.i32_t, 0, 0)),
      SemanticType::Bool => LLVMBuildRetVoid(builder),
      SemanticType::String => LLVMBuildRetVoid(builder),
      SemanticType::Void => LLVMBuildRetVoid(builder),
      SemanticType::Object(class) => LLVMBuildRetVoid(builder),
      SemanticType::Array(elem) => LLVMBuildRetVoid(builder),
      _ => unreachable!(),
    };
  }

  unsafe fn stmt(&mut self, stmt: &mut Stmt) {
    match stmt {
      Stmt::Simple(simple) => self.simple(simple),
      Stmt::If(if_) => {}
      Stmt::While(while_) => {}
      Stmt::For(for_) => {}
      Stmt::Return(return_) => {}
      Stmt::Print(print) => {}
      Stmt::Break(break_) => {}
      Stmt::SCopy(s_copy) => {}
      Stmt::Foreach(foreach) => {}
      Stmt::Guarded(guarded) => {}
      Stmt::Block(block) => self.block(block),
    }
  }

  unsafe fn block(&mut self, block: &mut Block) {
    for stmt in &mut block.stmt {
      self.stmt(stmt);
    }
  }

  unsafe fn simple(&mut self, simple: &mut Simple) {
    let builder = self.builder;
    match simple {
      Simple::Assign(assign) => {
        self.expr(&mut assign.dst);
        self.expr(&mut assign.src);
        match &assign.dst.data {
          ExprData::Id(id) => {
            let var_def = id.symbol.get();
            match var_def.scope.get().kind {
              ScopeKind::Local(_) | ScopeKind::Parameter(_) => {
                LLVMBuildStore(builder, assign.src.llvm_val, var_def.llvm_val);
              }
              ScopeKind::Class(_) => {
                let base = id.owner.as_ref().unwrap().llvm_val;
                let elem_ptr = LLVMBuildStructGEP(builder, base, var_def.offset as u32 + 1, T);
                LLVMBuildStore(builder, assign.src.llvm_val, elem_ptr);
              }
              _ => unreachable!(),
            }
          }
          ExprData::Indexed(indexed) => {
            let ptr = LLVMBuildGEP(builder, indexed.arr.llvm_val,
                                   [LLVMConstInt(self.i32_t, 0, 0), indexed.idx.llvm_val].as_mut_ptr(), 2, T);
            LLVMBuildStore(builder, assign.src.llvm_val, ptr);
          }
          _ => unreachable!(),
        }
      }
      Simple::VarDef(var_def) => {
        var_def.llvm_val = LLVMBuildAlloca(builder, self.type_of(&var_def.type_), T);
        if let Some(src) = &mut var_def.src {
          self.expr(src);
          LLVMBuildStore(builder, src.llvm_val, var_def.llvm_val);
        }
      }
      Simple::Expr(expr) => self.expr(expr),
      Simple::Skip => {}
    }
  }

  unsafe fn expr(&self, expr: &mut Expr) {
    use ast::ExprData::*;
    let builder = self.builder;
    expr.llvm_val = match &mut expr.data {
      Id(id) => {
        let var_def = id.symbol.get();
        match var_def.scope.get().kind {
          ScopeKind::Local(_) | ScopeKind::Parameter(_) => LLVMBuildLoad(builder, var_def.llvm_val, T),
          ScopeKind::Class(_) => {
            let owner = id.owner.as_mut().unwrap();
            self.expr(owner);
            LLVMBuildLoad(builder, LLVMBuildStructGEP(builder, owner.llvm_val, var_def.offset as u32 + 1, T), T)
          }
          _ => unreachable!(),
        }
      }
      Indexed(indexed) => {
        self.expr(&mut indexed.arr);
        self.expr(&mut indexed.idx);
        let ptr = LLVMBuildGEP(builder, indexed.arr.llvm_val, [self.i32_0, indexed.idx.llvm_val].as_mut_ptr(), 2, T);
        LLVMBuildLoad(builder, ptr, T)
      }
      IntConst(v) => LLVMConstInt(self.i32_t, *v as u64, 0),
      BoolConst(v) => LLVMConstInt(self.i8_t, if *v { 1 } else { 0 }, 0),
      StringConst(v) => {
        let s = LLVMAddGlobal(self.module, LLVMArrayType(self.i8_t, v.len() as u32 + 1), cstr!("str"));
        LLVMSetInitializer(s, LLVMConstStringInContext(self.context, cstring!(v.clone()), v.len() as u32, 0));
        LLVMConstGEP(s, [self.i32_0, self.i32_0, ].as_mut_ptr(), 2)
      }
      ArrayConst(_) => unimplemented!(),
      Null => LLVMConstNull(self.str_t), // will be casted to other pointer type when using the value(assign, param, ...)
      Call(call) => {
        unimplemented!()
      }
      Unary(unary) => {
        self.expr(&mut unary.r);
        match unary.op {
          Operator::Neg => LLVMBuildNeg(builder, unary.r.llvm_val, T),
          Operator::Not => LLVMBuildNot(builder, unary.r.llvm_val, T),
          _ => unimplemented!(),
        }
      }
      Binary(binary) => {
        use ast::Operator::*;
        self.expr(&mut binary.l);
        self.expr(&mut binary.r);
        let (l, r) = (binary.l.llvm_val, binary.r.llvm_val);
        match binary.op {
          Add => LLVMBuildAdd(builder, l, r, T),
          Sub => LLVMBuildSub(builder, l, r, T),
          Mul => LLVMBuildMul(builder, l, r, T),
          Div => LLVMBuildSDiv(builder, l, r, T),
          Lt => LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSLT, l, r, T),
          Le => LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSLE, l, r, T),
          Gt => LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSGT, l, r, T),
          Ge => LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSGE, l, r, T),
          And => {
            unimplemented!();
          }
          Or => {
            unimplemented!();
          }
          Eq => {
            unimplemented!();
          }
          Ne => {
            unimplemented!();
          }
          Repeat => {
            unimplemented!();
          }
          _ => unimplemented!(),
        }
      }
      This => self.cur_this,
      ReadInt => {
        unimplemented!()
      }
      ReadLine => {
        unimplemented!()
      }
      NewClass { name } => {
        let obj_t = self.type_of(&expr.type_);
        let obj = LLVMBuildMalloc(builder, obj_t, T);
        LLVMBuildCall(builder, self.memset, [obj, self.i32_0, LLVMSizeOf(obj_t)].as_mut_ptr(), 3, T);
        let v_tbl = LLVMBuildStructGEP(builder, obj, 0, T);
        LLVMBuildStore(builder, expr.type_.get_class().llvm_v_tbl, v_tbl);
        obj
      }
      NewArray { elem_t: _, len } => {
        self.expr(len);
        unimplemented!()
      }
      TypeTest { expr: src, name } => {
        self.expr(src);
        unimplemented!()
      }
      TypeCast { name, expr: src } => {
        self.expr(src);
        unimplemented!()
      }
      Range(_) => unimplemented!(),
      Default(default) => {
        self.expr(&mut default.arr);
        self.expr(&mut default.idx);
        unimplemented!()
      }
      Comprehension(_) => unimplemented!(),
    };
  }
}