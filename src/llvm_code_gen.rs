use super::ast::*;
use super::types::*;
use super::symbol::*;
use super::util::*;
use super::config::*;
use super::tac_code_gen::resolve_field_order;

use llvm_sys::*;
use llvm_sys::prelude::*;
use llvm_sys::core::*;

use std::ffi::CString;
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
  i1_t: LLVMTypeRef,
  i32_t: LLVMTypeRef,
  i8_t: LLVMTypeRef,
  void_t: LLVMTypeRef,
  str_t: LLVMTypeRef,
  i64_t: LLVMTypeRef,
  i32_0: LLVMValueRef,
  // c std library functions
  malloc: LLVMValueRef,
  printf: LLVMValueRef,
  scanf: LLVMValueRef,
  strcmp: LLVMValueRef,
  memset: LLVMValueRef,
  memcpy: LLVMValueRef,
  exit: LLVMValueRef,
  string_pool: HashMap<String, LLVMValueRef>,
  break_stack: Vec<LLVMBasicBlockRef>,
  cur_method: *const MethodDef,
}

impl LLVMCodeGen {
  pub fn gen(mut program: Program) {
    unsafe {
      let context = LLVMContextCreate();
      let module = LLVMModuleCreateWithNameInContext(cstr!("Decaf Program"), context);
      let builder = LLVMCreateBuilderInContext(context);
      let i1_t = LLVMInt1TypeInContext(context);
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
      let exit = LLVMAddFunction(module, cstr!("exit"), LLVMFunctionType(void_t, [i32_t].as_mut_ptr(), 1, 0));
      let mut code_gen = LLVMCodeGen { context, module, builder, i1_t, i32_t, i8_t, void_t, str_t, i64_t, i32_0, malloc, printf, scanf, strcmp, memset, memcpy, exit, string_pool: HashMap::new(), break_stack: Vec::new(), cur_method: ptr::null_mut() };
      code_gen.program(&mut program);
      LLVMDisposeBuilder(builder);
      LLVMDumpModule(module);
      LLVMContextDispose(context);
    }
  }
}

unsafe fn ptr_of(type_: LLVMTypeRef) -> LLVMTypeRef {
  LLVMPointerType(type_, 0)
}

impl LLVMCodeGen {
  unsafe fn array_length(&self, arr: LLVMValueRef) -> LLVMValueRef {
    let arr = LLVMBuildBitCast(self.builder, arr, ptr_of(self.i32_t), T);
    let len = LLVMBuildGEP(self.builder, arr, [LLVMConstInt(self.i32_t, -1i64 as u64, 1)].as_mut_ptr(), 1, T);
    LLVMBuildLoad(self.builder, len, T)
  }

  unsafe fn instance_of(&self, object: LLVMValueRef, target_v_tbl: LLVMValueRef) -> LLVMValueRef {
    // ret = 0
    // while (cur)
    //   if cur == target
    //     ret = 1
    //     break
    //   cur = cur->parent
    let builder = self.builder;
    let target_v_tbl = LLVMBuildPtrToInt(builder, target_v_tbl, self.i64_t, T);
    let v_tbl = LLVMBuildAlloca(builder, self.i64_t, T);
    let ret = LLVMBuildAlloca(builder, self.i1_t, T);
    let (before_cond, before_body, after_body) = (self.new_bb(), self.new_bb(), self.new_bb());
    let (on_eq, after_if) = (self.new_bb(), self.new_bb());
    LLVMBuildStore(builder, LLVMConstInt(self.i1_t, 0, 0), ret);
    LLVMBuildStore(builder, LLVMBuildPtrToInt(builder, LLVMBuildLoad(builder, LLVMBuildStructGEP(builder, object, 0, T), T), self.i64_t, T), v_tbl);
    LLVMBuildBr(builder, before_cond);
    self.label(before_cond);
    let v_tbl_load = LLVMBuildLoad(builder, v_tbl, T);
    LLVMBuildCondBr(builder, LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntEQ, v_tbl_load, LLVMConstInt(self.i64_t, 0, 0), T),
                    after_body, before_body);
    self.label(before_body);
    LLVMBuildCondBr(builder, LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntEQ, v_tbl_load, target_v_tbl, T),
                    on_eq, after_if);
    self.label(on_eq);
    LLVMBuildStore(builder, LLVMConstInt(self.i1_t, 1, 0), ret);
    LLVMBuildBr(builder, after_body);
    self.label(after_if);
    LLVMBuildStore(builder, LLVMBuildLoad(builder, LLVMBuildGEP(builder, LLVMBuildIntToPtr(builder, v_tbl_load, ptr_of(self.i64_t), T),
                                                                [self.i32_0].as_mut_ptr(), 1, T), T), v_tbl);
    LLVMBuildBr(builder, before_cond);
    self.label(after_body);
    LLVMBuildLoad(builder, ret, T)
  }

  unsafe fn alloc_array(&self, len: LLVMValueRef, elem_t: LLVMTypeRef) -> LLVMValueRef {
    let builder = self.builder;
    let tot_len = LLVMBuildAdd(builder, LLVMBuildMul(builder, LLVMBuildIntCast(builder, len, self.i64_t, T), LLVMSizeOf(elem_t), T), LLVMSizeOf(self.i32_t), T);
    let arr_base = LLVMBuildCall(builder, self.malloc, [tot_len].as_mut_ptr(), 1, T);
    LLVMBuildStore(builder, len, LLVMBuildBitCast(builder, arr_base, ptr_of(self.i32_t), T));
    let arr = LLVMBuildGEP(builder, arr_base, [LLVMSizeOf(self.i32_t)].as_mut_ptr(), 1, T);
    LLVMBuildBitCast(builder, arr, ptr_of(elem_t), T)
  }

  unsafe fn cur_bb_unterminated(&self) -> bool {
    LLVMGetBasicBlockTerminator(LLVMGetLastBasicBlock(self.cur_method.get().llvm_val)).is_null()
  }

  // the order of how LLVMAppendBasicBlockInContext is called does matter
  // it's not like just make a label and label it
  unsafe fn new_bb(&self) -> LLVMBasicBlockRef {
    LLVMAppendBasicBlockInContext(self.context, self.cur_method.get().llvm_val, T)
  }

  // this adjusts the order of bb
  // so that the logically last-added bb will be at last, and can use it like label
  unsafe fn label(&self, bb: LLVMBasicBlockRef) {
    let last = LLVMGetLastBasicBlock(self.cur_method.get().llvm_val);
    if bb != last {
      LLVMMoveBasicBlockAfter(bb, LLVMGetLastBasicBlock(self.cur_method.get().llvm_val));
    }
    LLVMPositionBuilderAtEnd(self.builder, bb);
  }

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
      SemanticType::Bool => self.i1_t,
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
    // determine class field
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
    // determine v table
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
    self.cur_method = method;
    self.label(bb);
    for (index, param) in method.param.iter_mut().enumerate() {
      param.llvm_val = LLVMBuildAlloca(builder, self.type_of(&param.type_), T);
      LLVMBuildStore(builder, LLVMGetParam(method.llvm_val, index as u32), param.llvm_val);
    }
    self.block(&mut method.body);
    // user code forget to return, just add it for him
    if self.cur_bb_unterminated() {
      match &method.ret_t.sem {
        SemanticType::Int => LLVMBuildRet(builder, self.i32_0),
        SemanticType::Bool => LLVMBuildRet(builder, LLVMConstInt(self.i1_t, 0, 0)),
        SemanticType::Void => LLVMBuildRetVoid(builder),
        SemanticType::String | SemanticType::Object(_) | SemanticType::Array(_) => LLVMBuildRet(builder, LLVMConstNull(self.type_of(&method.ret_t.sem))),
        _ => unreachable!(),
      };
    }
  }

  unsafe fn stmt(&mut self, stmt: &mut Stmt) {
    let builder = self.builder;
    match stmt {
      Stmt::Simple(simple) => self.simple(simple),
      Stmt::If(if_) => {
        self.expr(&mut if_.cond);
        let (on_true, on_false, after) = (self.new_bb(), self.new_bb(), self.new_bb());
        LLVMBuildCondBr(builder, if_.cond.llvm_val, on_true, on_false);
        self.label(on_true);
        self.block(&mut if_.on_true);
        if self.cur_bb_unterminated() {
          LLVMBuildBr(builder, after);
        }
        self.label(on_false);
        if let Some(on_false_block) = &mut if_.on_false { self.block(on_false_block); }
        if self.cur_bb_unterminated() {
          LLVMBuildBr(builder, after);
        }
        self.label(after);
      }
      Stmt::While(while_) => {
        let (before_cond, before_body, after_body) = (self.new_bb(), self.new_bb(), self.new_bb());
        LLVMBuildBr(builder, before_cond);
        self.label(before_cond);
        self.expr(&mut while_.cond);
        LLVMBuildCondBr(builder, while_.cond.llvm_val, before_body, after_body);
        self.label(before_body);
        self.break_stack.push(after_body);
        self.block(&mut while_.body);
        self.break_stack.pop();
        if self.cur_bb_unterminated() {
          LLVMBuildBr(builder, before_cond);
        }
        self.label(after_body);
      }
      Stmt::For(for_) => {
        let (before_cond, before_body, after_body) = (self.new_bb(), self.new_bb(), self.new_bb());
        self.simple(&mut for_.init);
        LLVMBuildBr(builder, before_cond);
        self.label(before_cond);
        self.expr(&mut for_.cond);
        LLVMBuildCondBr(builder, for_.cond.llvm_val, before_body, after_body);
        self.label(before_body);
        self.break_stack.push(after_body);
        self.block(&mut for_.body);
        self.break_stack.pop();
        if self.cur_bb_unterminated() {
          self.simple(&mut for_.update);
          LLVMBuildBr(builder, before_cond);
        }
        self.label(after_body);
      }
      Stmt::Return(return_) => if let Some(expr) = &mut return_.expr {
        self.expr(expr);
        LLVMBuildRet(builder, LLVMBuildBitCast(builder, expr.llvm_val, LLVMGetReturnType(self.cur_method.get().llvm_t), T));
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
      Stmt::Break(_) => { LLVMBuildBr(builder, *self.break_stack.last().unwrap()); }
      Stmt::SCopy(s_copy) => {
        self.expr(&mut s_copy.src);
        let obj_t = s_copy.src.type_.get_class().llvm_t;
        let obj = LLVMBuildMalloc(builder, obj_t, T);
        LLVMBuildCall(builder, self.memcpy, [self.to_i8_ptr(obj), self.to_i8_ptr(s_copy.src.llvm_val), LLVMSizeOf(obj_t)].as_mut_ptr(), 3, T);
        LLVMBuildStore(builder, obj, s_copy.dst_sym.get().llvm_val);
      }
      Stmt::Foreach(foreach) => {
        self.expr(&mut foreach.arr);
        let i = LLVMBuildAlloca(builder, self.i32_t, T);
        LLVMBuildStore(builder, self.i32_0, i); // REMEMBER TO INITIALIZE
        let one = LLVMConstInt(self.i32_t, 1, 0);
        let len = self.array_length(foreach.arr.llvm_val);
        let (before_i, before_cond, before_body, after_body) = (self.new_bb(), self.new_bb(), self.new_bb(), self.new_bb());
        LLVMBuildBr(builder, before_i);
        self.label(before_i);
        let load_i = LLVMBuildLoad(builder, i, T);
        LLVMBuildCondBr(builder, LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSLT, load_i, len, T), before_cond, after_body);
        self.label(before_cond);
        foreach.def.llvm_val = LLVMBuildGEP(builder, foreach.arr.llvm_val, [load_i].as_mut_ptr(), 1, T);
        if let Some(cond) = &mut foreach.cond {
          self.expr(cond);
          LLVMBuildCondBr(builder, cond.llvm_val, before_body, after_body);
        } else {
          LLVMBuildBr(builder, before_body);
        }
        self.label(before_body);
        self.break_stack.push(after_body);
        self.block(&mut foreach.body);
        self.break_stack.pop();
        if self.cur_bb_unterminated() {
          LLVMBuildStore(builder, LLVMBuildAdd(builder, load_i, one, T), i);
          LLVMBuildBr(builder, before_i);
        }
        self.label(after_body);
      }
      Stmt::Guarded(guarded) => for (e, b) in &mut guarded.guarded {
        let (on_true, on_false) = (self.new_bb(), self.new_bb());
        self.expr(e);
        LLVMBuildCondBr(builder, e.llvm_val, on_true, on_false);
        self.label(on_true);
        self.block(b);
        if self.cur_bb_unterminated() {
          LLVMBuildBr(builder, on_false);
        }
        self.label(on_false);
      }
      Stmt::Block(block) => self.block(block),
    }
  }

  // block belongs to bb
  unsafe fn block(&mut self, block: &mut Block) {
    for stmt in &mut block.stmt {
      // this basic block is terminated by return/break
      // doesn't need(and llvm doesn't permit) emit more code
      if !self.cur_bb_unterminated() {
        break;
      }
      self.stmt(stmt);
    }
  }

  unsafe fn simple(&mut self, simple: &mut Simple) {
    let builder = self.builder;
    match simple {
      Simple::Assign(assign) => {
        // to tell them not call LLVMBuildLoad, keep the pointer
        if let ExprData::Id(id) = &mut assign.dst.data { id.for_assign = true; }
        if let ExprData::Indexed(indexed) = &mut assign.dst.data { indexed.for_assign = true; }
        self.expr(&mut assign.dst);
        self.expr(&mut assign.src);
        // this is the advantage of pointer, I don't need to care about with form of assign it is, just store value to pointer
        LLVMBuildStore(builder, LLVMBuildBitCast(builder, assign.src.llvm_val, self.type_of(&assign.dst.type_), T), assign.dst.llvm_val);
      }
      Simple::VarDef(var_def) => {
        var_def.llvm_val = LLVMBuildAlloca(builder, self.type_of(&var_def.type_), T);
        LLVMBuildStore(builder, if let Some(src) = &mut var_def.src {
          self.expr(src);
          src.llvm_val
        } else {
          match &var_def.type_.sem {
            SemanticType::Int => self.i32_0,
            SemanticType::Bool => LLVMConstInt(self.i1_t, 0, 0),
            SemanticType::String | SemanticType::Object(_) | SemanticType::Array(_) => LLVMConstNull(self.type_of(&var_def.type_)),
            _ => unreachable!(),
          }
        }, var_def.llvm_val);
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
        let ptr = match var_def.scope.get().kind {
          ScopeKind::Local(_) | ScopeKind::Parameter(_) => var_def.llvm_val,
          ScopeKind::Class(_) => {
            let owner = id.owner.as_mut().unwrap();
            self.expr(owner);
            LLVMBuildStructGEP(builder, owner.llvm_val, var_def.offset as u32 + 1, T)
          }
          _ => unreachable!(),
        };
        if id.for_assign { ptr } else { LLVMBuildLoad(builder, ptr, T) }
      }
      Indexed(indexed) => {
        self.expr(&mut indexed.arr);
        self.expr(&mut indexed.idx);
        let (on_err, after) = (self.new_bb(), self.new_bb());
        // require unsigned(index) < length, just for convenience(this may cause error, but who cares?)
        LLVMBuildCondBr(builder, LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntULT, indexed.idx.llvm_val, self.array_length(indexed.arr.llvm_val), T), after, on_err);
        self.label(on_err);
        LLVMBuildCall(builder, self.printf, [self.define_str(INDEX_OUT_OF_BOUND)].as_mut_ptr(), 1, T);
        LLVMBuildCall(builder, self.exit, [self.i32_0].as_mut_ptr(), 1, T);
        LLVMBuildBr(builder, after);
        self.label(after);
        let ptr = LLVMBuildGEP(builder, indexed.arr.llvm_val, [indexed.idx.llvm_val].as_mut_ptr(), 1, T);
        if indexed.for_assign { ptr } else { LLVMBuildLoad(builder, ptr, T) }
      }
      IntConst(v) => LLVMConstInt(self.i32_t, *v as u64, 0),
      BoolConst(v) => LLVMConstInt(self.i1_t, if *v { 1 } else { 0 }, 0),
      StringConst(v) => self.define_str(v),
      ArrayConst(_) => unimplemented!(),
      Null => LLVMConstNull(self.str_t), // will be casted to other pointer type when using the value(assign, param, ...)
      Call(call) => if call.is_arr_len {
        let owner = call.owner.as_mut().unwrap();
        self.expr(owner);
        self.array_length(owner.llvm_val)
      } else {
        let method = call.method.get();
        if method.static_ {
          let mut arg = call.arg.iter_mut().zip(method.param.iter()).map(|(a, p)| {
            self.expr(a);
            LLVMBuildBitCast(builder, a.llvm_val, self.type_of(&p.type_), T)
          }).collect::<Vec<_>>();
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
        if binary.op != And && binary.op != Or { // they will handle the short-circuit eval
          self.expr(&mut binary.r);
        }
        let (l, r) = (binary.l.llvm_val, binary.r.llvm_val);
        match binary.op {
          Add => LLVMBuildAdd(builder, l, r, T),
          Sub => LLVMBuildSub(builder, l, r, T),
          Mul => LLVMBuildMul(builder, l, r, T),
          Div | Mod => {
            let (on_err, after) = (self.new_bb(), self.new_bb());
            LLVMBuildCondBr(builder, LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntEQ, r, self.i32_0, T), on_err, after);
            self.label(on_err);
            LLVMBuildCall(builder, self.printf, [self.define_str(DIV_0)].as_mut_ptr(), 1, T);
            LLVMBuildCall(builder, self.exit, [self.i32_0].as_mut_ptr(), 1, T);
            LLVMBuildBr(builder, after);
            self.label(after);
            if binary.op == Div {
              LLVMBuildSDiv(builder, l, r, T)
            } else {
              LLVMBuildSRem(builder, l, r, T)
            }
          }
          Lt => LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSLT, l, r, T),
          Le => LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSLE, l, r, T),
          Gt => LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSGT, l, r, T),
          Ge => LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSGE, l, r, T),
          And | Or => {
            let res = LLVMBuildAlloca(builder, self.i1_t, T);
            let (eval_r, on_true, on_false, after) = (self.new_bb(), self.new_bb(), self.new_bb(), self.new_bb());
            if binary.op == And {
              LLVMBuildCondBr(builder, l, eval_r, on_false);
            } else {
              LLVMBuildCondBr(builder, l, on_true, eval_r);
            }
            self.label(eval_r);
            self.expr(&mut binary.r);
            LLVMBuildCondBr(builder, binary.r.llvm_val, on_true, on_false);
            self.label(on_true);
            LLVMBuildStore(builder, LLVMConstInt(self.i1_t, 1, 0), res);
            LLVMBuildBr(builder, after);
            self.label(on_false);
            LLVMBuildStore(builder, LLVMConstInt(self.i1_t, 0, 0), res);
            LLVMBuildBr(builder, after);
            self.label(after);
            LLVMBuildLoad(builder, res, T)
          }
          Eq | Ne => if binary.l.type_ == STRING {
            let tmp = LLVMBuildCall(builder, self.strcmp, [l, r].as_mut_ptr(), 2, T);
            LLVMBuildICmp(builder, if binary.op == Eq { LLVMIntPredicate::LLVMIntEQ } else { LLVMIntPredicate::LLVMIntNE },
                          tmp, self.i32_0, T)
          } else {
            // use cast to allow obj == null / parent == child
            LLVMBuildICmp(builder, if binary.op == Eq { LLVMIntPredicate::LLVMIntEQ } else { LLVMIntPredicate::LLVMIntNE },
                          l, LLVMBuildBitCast(builder, r, self.type_of(&binary.l.type_), T), T)
          }
          Repeat => {
            let len = r;
            let elem_t = self.type_of(&binary.l.type_);
            let (on_err, after) = (self.new_bb(), self.new_bb());
            LLVMBuildCondBr(builder, LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSLT, len, self.i32_0, T), on_err, after);
            self.label(on_err);
            LLVMBuildCall(builder, self.printf, [self.define_str(REPEAT_NEG)].as_mut_ptr(), 1, T);
            LLVMBuildCall(builder, self.exit, [self.i32_0].as_mut_ptr(), 1, T);
            LLVMBuildBr(builder, after);
            self.label(after);
            let arr = self.alloc_array(len, elem_t);
            let i = LLVMBuildAlloca(builder, self.i32_t, T);
            LLVMBuildStore(builder, self.i32_0, i);
            let (before_cond, before_body, after_body) = (self.new_bb(), self.new_bb(), self.new_bb());
            LLVMBuildBr(builder, before_cond);
            self.label(before_cond);
            let i_load = LLVMBuildLoad(builder, i, T);
            LLVMBuildCondBr(builder, LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSLT, i_load, len, T), before_body, after_body);
            self.label(before_body);
            LLVMBuildStore(builder, match &binary.l.type_ {
              SemanticType::Int | SemanticType::Bool | SemanticType::String => l,
              SemanticType::Object(class) => {
                let obj_t = class.get().llvm_t;
                let obj = LLVMBuildMalloc(builder, obj_t, T);
                LLVMBuildCall(builder, self.memcpy, [self.to_i8_ptr(obj), self.to_i8_ptr(l), LLVMSizeOf(obj_t)].as_mut_ptr(), 3, T);
                obj
              }
              _ => unreachable!(),
            }, LLVMBuildGEP(builder, arr, [i_load].as_mut_ptr(), 1, T));
            LLVMBuildStore(builder, LLVMBuildAdd(builder, i_load, LLVMConstInt(self.i32_t, 1, 0), T), i);
            LLVMBuildBr(builder, before_cond);
            self.label(after_body);
            arr
          }
          _ => unimplemented!(),
        }
      }
      This => LLVMGetParam(self.cur_method.get().llvm_val, 0),
      ReadInt => {
        let tmp = LLVMBuildAlloca(builder, self.i32_t, T);
        LLVMBuildCall(builder, self.scanf, [self.define_str("%d"), tmp].as_mut_ptr(), 2, T);
        LLVMBuildLoad(builder, tmp, T)
      }
      ReadLine => {
        unimplemented!()
      }
      NewClass { name: _ } => {
        let obj_t = expr.type_.get_class().llvm_t;
        let obj = LLVMBuildMalloc(builder, obj_t, T);
        LLVMBuildCall(builder, self.memset, [self.to_i8_ptr(obj), self.i32_0, LLVMSizeOf(obj_t)].as_mut_ptr(), 3, T);
        let v_tbl = LLVMBuildStructGEP(builder, obj, 0, T);
        LLVMBuildStore(builder, expr.type_.get_class().llvm_v_tbl, v_tbl);
        obj
      }
      NewArray { elem_t, len } => {
        self.expr(len);
        let len = len.llvm_val;
        let elem_t = self.type_of(elem_t);
        let (on_err, after) = (self.new_bb(), self.new_bb());
        let cmp = LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSLT, len, self.i32_0, T);
        LLVMBuildCondBr(builder, cmp, on_err, after);
        self.label(on_err);
        LLVMBuildCall(builder, self.printf, [self.define_str(NEW_ARR_NEG)].as_mut_ptr(), 1, T);
        LLVMBuildCall(builder, self.exit, [self.i32_0].as_mut_ptr(), 1, T);
        LLVMBuildBr(builder, after);
        self.label(after);
        let arr = self.alloc_array(len, elem_t);
        LLVMBuildCall(builder, self.memset, [LLVMBuildBitCast(builder, arr, self.str_t, T), self.i32_0, LLVMBuildMul(builder, LLVMBuildIntCast(builder, len, self.i64_t, T), LLVMSizeOf(elem_t), T)].as_mut_ptr(), 3, T);
        arr
      }
      TypeTest { expr: src, name: _, target_class } => {
        self.expr(src);
        self.instance_of(src.llvm_val, target_class.get().llvm_v_tbl)
      }
      TypeCast { name, expr: src } => {
        self.expr(src);
        let target_t = expr.type_.get_class();
        let check = self.instance_of(src.llvm_val, target_t.llvm_v_tbl);
        let (on_false, after) = (self.new_bb(), self.new_bb());
        LLVMBuildCondBr(builder, check, after, on_false);
        self.label(on_false);
        let v_tbl = LLVMBuildLoad(builder, LLVMBuildStructGEP(builder, src.llvm_val, 0, T), T);
        let obj_name = LLVMBuildLoad(builder, LLVMBuildStructGEP(builder, v_tbl, 1, T), T);
        LLVMBuildCall(builder, self.printf, [self.define_str(BAD_CAST), obj_name, self.define_str(name)].as_mut_ptr(), 3, T);
        LLVMBuildCall(builder, self.exit, [self.i32_0].as_mut_ptr(), 1, T);
        LLVMBuildBr(builder, after);
        self.label(after);
        LLVMBuildBitCast(builder, src.llvm_val, ptr_of(target_t.llvm_t), T)
      }
      Range(_) => unimplemented!(),
      Default(default) => {
        self.expr(&mut default.arr);
        self.expr(&mut default.idx);
        let res = LLVMBuildAlloca(builder, self.type_of(&expr.type_), T);
        let (use_idx, use_dft, after) = (self.new_bb(), self.new_bb(), self.new_bb());
        let len = self.array_length(default.arr.llvm_val);
        LLVMBuildCondBr(builder, LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntUGE, default.idx.llvm_val, len, T), use_dft, use_idx);
        self.label(use_dft);
        self.expr(&mut default.dft);
        LLVMBuildStore(builder, default.dft.llvm_val, res);
        LLVMBuildBr(builder, after);
        self.label(use_idx);
        let ptr = LLVMBuildGEP(builder, default.arr.llvm_val, [default.idx.llvm_val].as_mut_ptr(), 1, T);
        LLVMBuildStore(builder, LLVMBuildLoad(builder, ptr, T), res);
        LLVMBuildBr(builder, after);
        self.label(after);
        LLVMBuildLoad(builder, res, T)
      }
      Comprehension(_) => unimplemented!(),
    };
  }
}