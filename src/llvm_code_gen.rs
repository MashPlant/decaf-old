use super::ast::*;
use super::types::*;
use super::symbol::*;
use super::util::*;
use super::tac_code_gen::resolve_field_order;

use llvm_sys::*;
use llvm_sys::prelude::*;
use llvm_sys::core::*;

use std::ffi::{CStr, CString};
use std::ptr;
use std::collections::HashMap;

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
  cur_fn: LLVMValueRef,
  // c std library functions
  malloc: LLVMValueRef,
  printf: LLVMValueRef,
  scanf: LLVMValueRef,
  strcmp: LLVMValueRef,
  memset: LLVMValueRef,
  memcpy: LLVMValueRef,
  string_pool: HashMap<String, LLVMValueRef>,
  break_stack: Vec<LLVMBasicBlockRef>,
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
      let malloc = LLVMAddFunction(module, cstr!("malloc"), LLVMFunctionType(str_t, [i64_t].as_mut_ptr(), 1, 0));
      let printf = LLVMAddFunction(module, cstr!("printf"), LLVMFunctionType(i32_t, [str_t].as_mut_ptr(), 1, 1));
      let scanf = LLVMAddFunction(module, cstr!("scanf"), LLVMFunctionType(i32_t, [str_t].as_mut_ptr(), 1, 1));
      let strcmp = LLVMAddFunction(module, cstr!("strcmp"), LLVMFunctionType(i32_t, [str_t, str_t].as_mut_ptr(), 2, 0));
      let memset = LLVMAddFunction(module, cstr!("memset"), LLVMFunctionType(str_t, [str_t, i32_t, i64_t].as_mut_ptr(), 3, 0));
      let memcpy = LLVMAddFunction(module, cstr!("memcpy"), LLVMFunctionType(str_t, [str_t, str_t, i64_t].as_mut_ptr(), 3, 0));
      let mut code_gen = LLVMCodeGen { context, module, builder, i32_t, i8_t, void_t, str_t, i64_t, i32_0, cur_fn: ptr::null_mut(), malloc, printf, scanf, strcmp, memset, memcpy, string_pool: HashMap::new(), break_stack: Vec::new() };
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
  unsafe fn to_i8_ptr(&self, val: LLVMValueRef) -> LLVMValueRef {
    LLVMBuildBitCast(self.builder, val, self.str_t, T)
  }

  // return POINTER to the first char of this string literal
  unsafe fn define_str(&mut self, s: &str) -> LLVMValueRef {
    // I hate borrow checker...
    let (module, context, i8_t, i32_0) = (self.module, self.context, self.i8_t, self.i32_0);
    *self.string_pool.entry(s.to_owned()).or_insert_with(|| {
      let g = LLVMAddGlobal(module, LLVMArrayType(i8_t, s.len() as u32 + 1), cstr!("str"));
      LLVMSetInitializer(g, LLVMConstStringInContext(context, cstring!(s), s.len() as u32, 0));
      LLVMConstGEP(g, [i32_0, i32_0].as_mut_ptr(), 2)
    })
  }

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

  // declare & define struct type; declare v table type & val
  unsafe fn make_struct_type(&mut self, class: &mut ClassDef) {
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
      match field {
        FieldDef::VarDef(var) => elem_t.push(self.type_of(&var.type_)),
        FieldDef::MethodDef(method) => {
          let mut param_t = method.param.iter().map(|a| self.type_of(&a.type_)).collect::<Vec<_>>();
          method.llvm_t = LLVMFunctionType(self.type_of(&method.ret_t), param_t.as_mut_ptr(), param_t.len() as u32, 0);
          method.llvm_val = LLVMAddFunction(self.module, cstring!(format!("{}_{}", method.class.get().name, method.name)), method.llvm_t);
        }
      }
    }
    LLVMStructSetBody(class.llvm_t, elem_t.as_mut_ptr(), elem_t.len() as u32, 0);
    class.llvm_v_tbl = LLVMAddGlobal(self.module, class.llvm_v_tbl_t, cstring!(format!("{}_VTable", class.name)));
    let mut v_tbl_elem_t = vec![if class.p_ptr.is_null() { self.str_t } else { ptr_of(class.p_ptr.get().llvm_v_tbl_t) }, self.str_t];
    let mut v_tbl_elem = vec![if class.p_ptr.is_null() { LLVMConstNull(self.str_t) } else { class.p_ptr.get().llvm_v_tbl },
                              self.define_str(class.name)];
    for method in &class.v_tbl.methods {
      v_tbl_elem_t.push(ptr_of(method.get().llvm_t));
      v_tbl_elem.push(method.get().llvm_val);
    }
    LLVMStructSetBody(class.llvm_v_tbl_t, v_tbl_elem_t.as_mut_ptr(), v_tbl_elem_t.len() as u32, 0);
    LLVMSetInitializer(class.llvm_v_tbl, LLVMConstStruct(v_tbl_elem.as_mut_ptr(), v_tbl_elem.len() as u32, 0));
  }

  unsafe fn program(&mut self, program: &mut Program) {
    for class in &mut program.class {
      resolve_field_order(class);
      self.make_struct_type(class);
    }
    // must visit methods after all v tables are determined
    for class in &mut program.class {
      for field in &mut class.field {
        if let FieldDef::MethodDef(method) = field { self.method(method); }
      }
    }
    // add main function
    let main_t = LLVMFunctionType(self.i32_t, [].as_mut_ptr(), 0, 0);
    let main = LLVMAddFunction(self.module, cstr!("main"), main_t);
    let bb = LLVMAppendBasicBlockInContext(self.context, main, cstr!("entry"));
    LLVMPositionBuilderAtEnd(self.builder, bb);
    LLVMBuildCall(self.builder, LLVMGetNamedFunction(self.module, cstr!("Main_main")), [].as_mut_ptr(), 0, T);
    LLVMBuildRet(self.builder, self.i32_0);
  }

  unsafe fn method(&mut self, method: &mut MethodDef) {
    let (context, builder) = (self.context, self.builder);
    let bb = LLVMAppendBasicBlockInContext(context, method.llvm_val, cstr!("entry"));
    LLVMPositionBuilderAtEnd(builder, bb);
    for (index, param) in method.param.iter_mut().enumerate() {
      param.llvm_val = LLVMGetParam(method.llvm_val, index as u32);
    }
    self.cur_fn = method.llvm_val;
    self.block(&mut method.body);
    match &method.ret_t.sem {
      SemanticType::Int => LLVMBuildRet(builder, self.i32_0),
      SemanticType::Bool => LLVMBuildRet(builder, LLVMConstInt(self.i8_t, 0, 0)),
      SemanticType::Void => LLVMBuildRetVoid(builder),
      SemanticType::String | SemanticType::Object(_) | SemanticType::Array(_) => LLVMBuildRet(builder, LLVMConstNull(self.type_of(&method.ret_t.sem))),
      _ => unreachable!(),
    };
  }

  unsafe fn stmt(&mut self, stmt: &mut Stmt) {
    let builder = self.builder;
    match stmt {
      Stmt::Simple(simple) => self.simple(simple),
      Stmt::If(if_) => {
        self.expr(&mut if_.cond);
        let (on_true, on_false, after) = (LLVMAppendBasicBlockInContext(self.context, self.cur_fn, T), LLVMAppendBasicBlockInContext(self.context, self.cur_fn, T), LLVMAppendBasicBlockInContext(self.context, self.cur_fn, T));
        LLVMBuildCondBr(builder, if_.cond.llvm_val, on_true, on_false);
        LLVMPositionBuilderAtEnd(builder, on_true);
        self.block(&mut if_.on_true);
        LLVMBuildBr(builder, after);
        LLVMPositionBuilderAtEnd(builder, on_false);
        if let Some(on_false) = &mut if_.on_false { self.block(on_false); }
        LLVMBuildBr(builder, after);
        LLVMPositionBuilderAtEnd(builder, after);
      }
      Stmt::While(while_) => {
        let (before_cond, before_body, after_body) = (LLVMAppendBasicBlockInContext(self.context, self.cur_fn, T), LLVMAppendBasicBlockInContext(self.context, self.cur_fn, T), LLVMAppendBasicBlockInContext(self.context, self.cur_fn, T));
        LLVMBuildBr(builder, before_cond);
        LLVMPositionBuilderAtEnd(builder, before_cond);
        self.expr(&mut while_.cond);
        LLVMBuildCondBr(builder, while_.cond.llvm_val, before_body, after_body);
        LLVMPositionBuilderAtEnd(builder, before_body);
        self.break_stack.push(after_body);
        self.block(&mut while_.body);
        self.break_stack.pop();
        LLVMBuildBr(builder, before_cond);
        LLVMPositionBuilderAtEnd(builder, after_body);
      }
      Stmt::For(for_) => {
        let (before_cond, before_body, after_body) = (LLVMAppendBasicBlockInContext(self.context, self.cur_fn, T), LLVMAppendBasicBlockInContext(self.context, self.cur_fn, T), LLVMAppendBasicBlockInContext(self.context, self.cur_fn, T));
        self.simple(&mut for_.init);
        LLVMBuildBr(builder, before_cond);
        LLVMPositionBuilderAtEnd(builder, before_cond);
        self.expr(&mut for_.cond);
        LLVMBuildCondBr(builder, for_.cond.llvm_val, before_body, after_body);
        LLVMPositionBuilderAtEnd(builder, before_body);
        self.break_stack.push(after_body);
        self.block(&mut for_.body);
        self.break_stack.pop();
        self.simple(&mut for_.update);
        LLVMBuildBr(builder, before_cond);
        LLVMPositionBuilderAtEnd(builder, after_body);
      }
      Stmt::Return(return_) => if let Some(expr) = &mut return_.expr {
        self.expr(expr);
        LLVMBuildRet(builder, expr.llvm_val);
      } else { LLVMBuildRetVoid(builder); }
      Stmt::Print(print) => for print in &mut print.print {
        self.expr(print);
        match &print.type_ {
          SemanticType::Int => LLVMBuildCall(builder, self.printf, [self.define_str("%d"), print.llvm_val].as_mut_ptr(), 2, T),
          SemanticType::String => LLVMBuildCall(builder, self.printf, [self.define_str("%s"), print.llvm_val].as_mut_ptr(), 2, T),
          SemanticType::Bool => {
            let tf = LLVMBuildSelect(builder, print.llvm_val, self.define_str("true"), self.define_str("false"), T);
            LLVMBuildCall(builder, self.printf, [tf].as_mut_ptr(), 1, T)
          }
          _ => unreachable!(),
        };
      }
      Stmt::Break(break_) => {
        let after_break = LLVMAppendBasicBlockInContext(self.context, self.cur_fn, T);
        LLVMBuildBr(builder, *self.break_stack.last().unwrap());
        LLVMPositionBuilderAtEnd(builder, after_break);
      }
      Stmt::SCopy(s_copy) => {
        self.expr(&mut s_copy.src);
        let obj_t = s_copy.src.type_.get_class().llvm_t;
        let obj = LLVMBuildMalloc(builder, obj_t, T);
        LLVMBuildCall(builder, self.memcpy, [self.to_i8_ptr(obj), self.to_i8_ptr(s_copy.src.llvm_val), LLVMSizeOf(obj_t)].as_mut_ptr(), 3, T);
        LLVMBuildStore(builder, obj, s_copy.dst_sym.get().llvm_val);
      }
      Stmt::Foreach(foreach) => {
        unimplemented!()
      }
      Stmt::Guarded(guarded) => {
        unimplemented!()
      }
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
            let ptr = LLVMBuildGEP(builder, indexed.arr.llvm_val, [indexed.idx.llvm_val].as_mut_ptr(), 1, T);
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

  unsafe fn expr(&mut self, expr: &mut Expr) {
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
        let ptr = LLVMBuildGEP(builder, indexed.arr.llvm_val, [indexed.idx.llvm_val].as_mut_ptr(), 1, T);
        LLVMBuildLoad(builder, ptr, T)
      }
      IntConst(v) => LLVMConstInt(self.i32_t, *v as u64, 0),
      BoolConst(v) => LLVMConstInt(self.i8_t, if *v { 1 } else { 0 }, 0),
      StringConst(v) => self.define_str(v),
      ArrayConst(_) => unimplemented!(),
      Null => LLVMConstNull(self.str_t), // will be casted to other pointer type when using the value(assign, param, ...)
      Call(call) => if call.is_arr_len {
        let owner = call.owner.as_mut().unwrap();
        self.expr(owner);
        let arr = LLVMBuildBitCast(builder, owner.llvm_val, ptr_of(self.i32_t), T);
        let len = LLVMBuildGEP(builder, arr, [LLVMConstNeg(LLVMSizeOf(self.i32_t))].as_mut_ptr(), 1, T);
        LLVMBuildLoad(builder, len, T)
      } else {
        let method = call.method.get();
        let class = method.class.get();
        if method.static_ {
          // here method.llvm_val is meaningful, just call it
          let mut arg = call.arg.iter_mut().map(|e| { (self.expr(e), e.llvm_val).1 })
            .collect::<Vec<_>>();
          LLVMBuildCall(builder, method.llvm_val, arg.as_mut_ptr(), arg.len() as u32, T)
        } else {
          let owner = call.owner.as_mut().unwrap();
          self.expr(owner);
          let mut arg = vec![owner.llvm_val];
          arg.extend(call.arg.iter_mut().map(|e| { (self.expr(e), e.llvm_val).1 }));
          let v_tbl = LLVMBuildLoad(builder, LLVMBuildStructGEP(builder, owner.llvm_val, 0, T), T);
          let v_fn = LLVMBuildLoad(builder, LLVMBuildStructGEP(builder, v_tbl, method.offset as u32 + 2, T), T);
          LLVMBuildCall(builder, v_fn, arg.as_mut_ptr(), arg.len() as u32, T)
        }
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
          Eq | Ne => if binary.l.type_ == STRING {
            let cmp = LLVMBuildCall(builder, self.strcmp, [l, r].as_mut_ptr(), 2, T);
            if binary.op == Eq { LLVMBuildNot(builder, cmp, T) } else { cmp }
          } else {
            LLVMBuildICmp(builder, if binary.op == Eq { LLVMIntPredicate::LLVMIntEQ } else { LLVMIntPredicate::LLVMIntNE },
                          l, r, T)
          }
          Repeat => {
            unimplemented!();
          }
          _ => unimplemented!(),
        }
      }
      This => LLVMGetParam(self.cur_fn, 0),
      ReadInt => {
        let tmp = LLVMBuildAlloca(builder, self.i32_t, T);
        LLVMBuildCall(builder, self.scanf, [self.define_str("%d"), tmp].as_mut_ptr(), 2, T);
        LLVMBuildLoad(builder, tmp, T)
      }
      ReadLine => {
        unimplemented!()
      }
      NewClass { name } => {
        let obj_t = expr.type_.get_class().llvm_t;
        let obj = LLVMBuildMalloc(builder, obj_t, T);
        LLVMBuildCall(builder, self.memset, [self.to_i8_ptr(obj), self.i32_0, LLVMSizeOf(obj_t)].as_mut_ptr(), 3, T);
        let v_tbl = LLVMBuildStructGEP(builder, obj, 0, T);
        LLVMBuildStore(builder, expr.type_.get_class().llvm_v_tbl, v_tbl);
        obj
      }
      NewArray { elem_t, len } => {
        self.expr(len);
        let elem_t = self.type_of(elem_t);
        let tot_len = LLVMBuildAdd(builder, LLVMBuildMul(builder, LLVMBuildIntCast(builder, len.llvm_val, self.i64_t, T), LLVMSizeOf(elem_t), T), LLVMSizeOf(self.i32_t), T);
        let arr_base = LLVMBuildCall(builder, self.malloc, [tot_len].as_mut_ptr(), 1, T);
        LLVMBuildCall(builder, self.memset, [arr_base, self.i32_0, tot_len].as_mut_ptr(), 3, T);
        LLVMBuildStore(builder, len.llvm_val, LLVMBuildBitCast(builder, arr_base, ptr_of(self.i32_t), T));
        let arr = LLVMBuildGEP(builder, arr_base, [LLVMSizeOf(self.i32_t)].as_mut_ptr(), 1, T);
        LLVMBuildBitCast(builder, arr, ptr_of(elem_t), T)
      }
      TypeTest { expr: src, name: _ } => {
        self.expr(src);
        unimplemented!()
      }
      TypeCast { name: _, expr: src } => {
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