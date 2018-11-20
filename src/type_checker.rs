use super::ast::*;
use super::types::*;
use super::errors::*;
use super::symbol::*;

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
}

impl TypeChecker {
    fn check_bool(&mut self, expr: &mut Expr) {
        self.visit_expr(expr);
        let t = expr.get_type();
        if !require_type(t, &BOOL) {
            issue!(self, expr.get_loc(), TestNotBool);
        }
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
//        self.visit_lvalue(&mut assign.dst);
//        self.visit_expr(&mut assign.src);
//        if assign.dst != &ERROR && () {
//
//        }


//        if (!assign.left.type.equal(BaseType.ERROR)
//            && (assign.left.type.isFuncType() || !assign.expr.type
//        .compatible(assign.left.type))) {
//        issueError(new IncompatBinOpError(assign.getLocation(),
//        assign.left.type.toString(), "=", assign.expr.type
//        .toString()));
//        }
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
        if {
            !match binary.opt {
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
            }
        } {
            issue!(self, binary.loc, IncompatibleBinary {
                left_type: left_t.to_string(),
                opt: binary.opt.to_str(),
                right_type: right_t.to_string(),
            });
        }
    }

    fn visit_identifier(&mut self, identifier: &mut Identifier) {
        unsafe {
            match &mut identifier.owner {
                Some(owner) => {
//                    if let Expr::Identifier(owner) = owner {
//
//                    }
                }
                None => {
                    match self.scopes.lookup_before(identifier.name, identifier.loc) {
                        Some(symbol) => {
                            match symbol {
                                Symbol::Class(class) => {
                                    identifier.type_ = SemanticType::Class((*class).name, class);
                                    if identifier.use_for_ref {
                                        identifier.is_class = true
                                    } else {
                                        // e.g. x = ClassName
                                        issue!(self, identifier.loc, UndeclaredVar { name: identifier.name });
                                        identifier.type_ = ERROR;
                                    }
                                }
                                Symbol::Method(method) => identifier.type_ = SemanticType::Method(method),
                                Symbol::Var(var) => {
                                    identifier.type_ = var.type_.sem.clone();
                                    if (*self.current_method).static_ {
                                        issue!(self, identifier.loc, RefInStatic {
                                            field: identifier.name,
                                            method: (*self.current_method).name
                                        });
                                    } else {
                                        // add a virtual `this`, it doesn't need visit
                                        identifier.owner = Some(Box::new(Expr::This(This {
                                            loc: identifier.loc,
                                            type_: SemanticType::Class((*self.current_class).name, self.current_class),
                                        })));
                                    }
                                }
                            }
                        }
                        None => {
                            issue!(self, identifier.loc, UndeclaredVar { name: identifier.name });
                            identifier.type_ = ERROR;
                        }
                    }
                }
            }
        }
        /*
            ident.owner.usedForRef = true;
            ident.owner.accept(this);
            if (!ident.owner.type.equal(BaseType.ERROR)) {
                if (ident.owner.isClass || !ident.owner.type.isClassType()) {
                    issueError(new NotClassFieldError(ident.getLocation(),
                            ident.name, ident.owner.type.toString()));
                    ident.type = BaseType.ERROR;
                } else {
                    ClassScope cs = ((ClassType) ident.owner.type)
                            .getClassScope();
                    Symbol v = cs.lookupVisible(ident.name);
                    if (v == null) {
                        issueError(new FieldNotFoundError(ident.getLocation(),
                                ident.name, ident.owner.type.toString()));
                        ident.type = BaseType.ERROR;
                    } else if (v.isVariable()) {
                        ClassType thisType = ((ClassScope) table
                                .lookForScope(Scope.Kind.CLASS)).getOwner()
                                .getType();
                        ident.type = v.getType();
                        if (!thisType.compatible(ident.owner.type)) {
                            issueError(new FieldNotAccessError(ident
                                    .getLocation(), ident.name,
                                    ident.owner.type.toString()));
                        } else {
                            ident.symbol = (Variable) v;
                            ident.lvKind = Tree.LValue.Kind.MEMBER_VAR;
                        }
                    } else {
                        ident.type = v.getType();
                    }
                }
            } else {
                ident.type = BaseType.ERROR;
            }
        }
        */
    }
}