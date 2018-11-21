use super::ast::*;
use super::types::*;
use super::errors::*;
use super::symbol::*;
use std::ptr;

macro_rules! issue {
    ($rec:expr, $loc: expr, $err: expr) => {
        $rec.errors.push(Error::new($loc, $err));
    };
}

pub struct TypeChecker {
    errors: Vec<Error>,
    scopes: ScopeStack,
    loop_counter: i32,
    current_method: *const MethodDef,
    current_class: *const ClassDef,
    current_id_used_for_ref: bool,
}

impl TypeChecker {
    pub fn new() -> TypeChecker {
        TypeChecker {
            errors: Vec::new(),
            scopes: ScopeStack {
                global_scope: ptr::null_mut(),
                scopes: Vec::new(),
            },
            loop_counter: 0,
            current_method: ptr::null(),
            current_class: ptr::null(),
            current_id_used_for_ref: false,
        }
    }

    pub fn check(mut self, mut program: Program) -> Result<Program, Vec<Error>> {
        self.visit_program(&mut program);
        if self.errors.is_empty() {
            Ok(program)
        } else {
            self.errors.sort_by_key(|x| x.loc);
            Err(self.errors)
        }
    }

    fn check_bool(&mut self, expr: &mut Expr) {
        self.visit_expr(expr);
        let t = expr.get_type();
        if !require_type(t, &BOOL) {
            issue!(self, expr.get_loc(), TestNotBool);
        }
    }

    fn check_call(&mut self, call_: &mut Call, method: Option<Symbol>) {

    }
}

fn require_type(type_: &SemanticType, target: &SemanticType) -> bool {
    type_ == &ERROR || type_ == target
}

impl Visitor for TypeChecker {
    fn visit_program(&mut self, program: &mut Program) {
        self.scopes.open(&mut program.scope);
        for class_def in &mut program.classes { self.visit_class_def(class_def); }
        self.scopes.close();
    }

    fn visit_class_def(&mut self, class_def: &mut ClassDef) {
        self.current_class = class_def;
        self.scopes.open(&mut class_def.scope);
        for field_def in &mut class_def.fields { self.visit_field_def(field_def) }
        self.scopes.close();
    }

    fn visit_method_def(&mut self, method_def_: &mut MethodDef) {
        self.current_method = method_def_;
        self.scopes.open(&mut method_def_.scope);
        self.visit_block(&mut method_def_.body);
        self.scopes.close();
    }

    fn visit_block(&mut self, block: &mut Block) {
        self.scopes.open(&mut block.scope);
        for statement in &mut block.statements { self.visit_statement(statement); }
        self.scopes.close();
    }

    fn visit_while(&mut self, while_: &mut While) {
        self.check_bool(&mut while_.cond);
        self.loop_counter += 1;
        self.visit_statement(&mut while_.body);
        self.loop_counter -= 1;
    }

    fn visit_break(&mut self, break_: &mut Break) {
        if self.loop_counter == 0 { issue!(self, break_.loc, BreakOutOfLoop); }
    }

    fn visit_assign(&mut self, assign: &mut Assign) {
        self.visit_expr(&mut assign.dst);
        self.visit_expr(&mut assign.src);
        let dst_type = assign.dst.get_type();
        let src_type = assign.src.get_type();
        if dst_type != &ERROR && (dst_type.is_method() || !src_type.extends(dst_type)) {
            issue!(self, assign.loc, IncompatibleBinary{left_type:dst_type.to_string(), opt:"=", right_type:src_type.to_string() })
        }
    }

    fn visit_unary(&mut self, unary: &mut Unary) {
        self.visit_expr(&mut unary.opr);
        let opr = unary.opr.get_type();
        match unary.opt {
            Operator::Neg => {
                if require_type(opr, &INT) {
                    unary.type_ = INT;
                } else {
                    issue!(self, unary.loc, IncompatibleUnary { opt: "-", type_: opr.to_string() });
                    unary.type_ = ERROR;
                }
            }
            Operator::Not => {
                if !require_type(opr, &BOOL) {
                    issue!(self, unary.loc, IncompatibleUnary { opt: "!", type_: opr.to_string() });
                    unary.type_ = ERROR;
                }
                // no matter error or not, set type to bool
                unary.type_ = BOOL;
            }
            _ => unreachable!(),
        }
    }

    fn visit_binary(&mut self, binary: &mut Binary) {
        self.visit_expr(&mut binary.left);
        self.visit_expr(&mut binary.right);
        let (left, right) = (&*binary.left, &*binary.right);
        let (left_t, right_t) = (left.get_type(), right.get_type());
        if left_t == &ERROR || right_t == &ERROR {
            match binary.opt {
                Operator::Add | Operator::Sub | Operator::Mul | Operator::Div => binary.type_ = left_t.clone(),
                Operator::Mod => binary.type_ = INT,
                Operator::Repeat | Operator::Concat => unimplemented!(),
                _ => binary.type_ = BOOL,
            }
            return;
        }
        // TODO move repeat & concat out from binary operator(both in java & rust version)
        if !match binary.opt {
            Operator::Add | Operator::Sub | Operator::Mul | Operator::Div | Operator::Mod => {
                binary.type_ = left_t.clone();
                left_t == &INT && right_t == &INT
            }
            Operator::Lt | Operator::Le | Operator::Gt | Operator::Ge => {
                binary.type_ = BOOL;
                left_t == &INT && right_t == &INT
            }
            Operator::Eq | Operator::Ne => {
                binary.type_ = BOOL;
                left_t == right_t
            }
            Operator::And | Operator::Or => {
                binary.type_ = BOOL;
                left_t == &BOOL && right_t == &BOOL
            }
            Operator::Repeat | Operator::Concat => unimplemented!(),
            _ => unreachable!(),
        } {
            issue!(self, binary.loc, IncompatibleBinary {
                left_type: left_t.to_string(),
                opt: binary.opt.to_str(),
                right_type: right_t.to_string(),
            });
        }
    }

    fn visit_call(&mut self, call_: &mut Call) {
//        let rec_t = match &call_.receiver {
//            Some(expr) => expr.get_type(),
//            None => self.current_class,
//        };
        match &mut call_.rec {
            Some(receiver) => {
                self.current_id_used_for_ref = true;
                self.visit_expr(receiver);
                let rec_t = receiver.get_type();
                if rec_t == &ERROR {
                    call_.type_ = ERROR;
                    return;
                }
                // check array length call
                // quite a dirty implementation
                if call_.name == "length" {
                    if rec_t.is_array() {
                        if !call_.args.is_empty() {
                            issue!(self, call_.loc, LengthWithArgument { count: call_.arguments.len() as i32 });
                        }
                        call_.type_ = INT;
                    } else if !rec_t.is_object() {
                        issue!(self, call_.loc, BadLength);
                        call_.type_ = ERROR;
                    }
                }
                if !rec_t.is_object() {
                    issue!(self, call_.loc, BadFieldAccess{name: call_.name, owner_type: rec_t.to_string() });
                    call_.type_ = ERROR;
                    return;
                }
//                self.check_call(call_,  rec_t. )
            }
            None => {}
        }
        /*
        if (callExpr.receiver == null) {
            ClassScope cs = (ClassScope) table.lookForScope(Kind.CLASS);
            checkCallExpr(callExpr, cs.lookupVisible(callExpr.method));
            return;
        }

        if (!callExpr.receiver.type.isClassType()) {
            issueError(new NotClassFieldError(callExpr.getLocation(),
                    callExpr.method, callExpr.receiver.type.toString()));
            callExpr.type = BaseType.ERROR;
            return;
        }

        ClassScope cs = ((ClassType) callExpr.receiver.type)
                .getClassScope();
        checkCallExpr(callExpr, cs.lookupVisible(callExpr.method));
        */
    }

    fn visit_identifier(&mut self, id: &mut Identifier) {
        // not found(no owner) or sole ClassName => UndeclaredVar
        // refer to field in static function => RefInStatic
        // <not object>.a (Main.a, 1.a, func.a) => BadFieldAssess
        // access a field that doesn't belong to self & parent => PrivateFieldAccess
        // given owner but not found object.a => NoSuchField

        // actually a ClassName in the looking-up process is bound to occur an error
        // wither UndeclaredVar or BadFieldAssess

        unsafe {
            let owner_ptr = &mut id.owner as *mut _; // work with borrow check
            match &mut id.owner {
                Some(owner) => {
                    self.current_id_used_for_ref = true;
                    self.visit_expr(owner);
                    let owner_type = owner.get_type();
                    match owner_type {
                        SemanticType::Object(_, class) => {
                            let class = &**class;
                            // lookup through inheritance chain
                            match class.lookup(id.name) {
                                Some(symbol) => {
                                    match symbol {
                                        Symbol::Var(var, _) => {
                                            id.type_ = (*var).type_.sem.clone();
                                            if !(*self.current_class).extends(class) {
                                                issue!(self, id.loc, PrivateFieldAccess { name: id.name, owner_type: owner_type.to_string() });
                                            }
                                        }
                                        _ => id.type_ = symbol.get_type(),
                                    }
                                }
                                None => {
                                    issue!(self, id.loc, NoSuchField { name: id.name, owner_type: owner_type.to_string() });
                                    id.type_ = ERROR;
                                }
                            }
                        }
                        SemanticType::Error => id.type_ = ERROR,
                        _ => {
                            issue!(self, id.loc, BadFieldAccess{name: id.name, owner_type: owner_type.to_string() });
                            id.type_ = ERROR;
                        }
                    }
                }
                None => {
                    match self.scopes.lookup_before(id.name, id.loc) {
                        Some(symbol) => {
                            match symbol {
                                Symbol::Class(class) => {
                                    if !self.current_id_used_for_ref {
                                        issue!(self, id.loc, UndeclaredVar { name: id.name });
                                        id.type_ = ERROR;
                                    } else { id.type_ = SemanticType::Object((*class).name, class); }
                                }
                                Symbol::Method(method) => id.type_ = SemanticType::Method(method),
                                Symbol::Var(var, scope) => {
                                    id.type_ = (*var).type_.sem.clone();
                                    if (*scope).is_class() && (*self.current_method).static_ {
                                        issue!(self, id.loc, RefInStatic {
                                            field: id.name,
                                            method: (*self.current_method).name
                                        });
                                    } else {
                                        // add a virtual `this`, it doesn't need visit
                                        *owner_ptr = Some(Box::new(Expr::This(This {
                                            loc: id.loc,
                                            type_: SemanticType::Object((*self.current_class).name, self.current_class),
                                        })));
                                    }
                                }
                            }
                        }
                        None => {
                            issue!(self, id.loc, UndeclaredVar { name: id.name });
                            id.type_ = ERROR;
                        }
                    }
                    self.current_id_used_for_ref = false;
                }
            }
        }
    }
}