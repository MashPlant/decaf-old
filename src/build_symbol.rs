use super::ast::*;
use super::loc::*;
use super::errors::*;
use super::config::*;
use std::collections::HashMap;
use std::ptr;

struct BuildSymbol {
    errors: Vec<Error>,
    class_scope: *mut HashMap<&'static str, *mut FieldDef>,
    scope_stack: Vec<*mut HashMap<&'static str, *mut VarDef>>,
}

unsafe fn calc_order(class_def: &mut ClassDef) -> i32 {
    if class_def.order == 0 {
        class_def.order = calc_order(&mut *class_def.parent_ref) + 1;
    }
    class_def.order
}

impl BuildSymbol {
    unsafe fn check_override(&mut self, class_def: &ClassDef) {}

    unsafe fn check_main(&mut self, class_def: *const ClassDef) -> bool {
        true
    }
}

impl Visitor for BuildSymbol {
    fn visit_program(&mut self, program: &mut Program) {
        unsafe {
            for class_def in &mut program.classes {
                program.symbols.entry(class_def.name)
                    .and_modify(|earlier| {
                        self.errors.push(Error::new(class_def.loc, ConflictDeclaration {
                            earlier: (**earlier).loc,
                            name: class_def.name,
                        }));
                    })
                    .or_insert(class_def);
            }
            for class_def in &mut program.classes {
                if let Some(parent) = class_def.parent {
                    if let Some(parent_ref) = program.symbols.get(parent) {
                        if calc_order(class_def) <= calc_order(&mut **parent_ref) {
                            self.errors.push(Error::new(class_def.loc, CyclicInheritance));
                        } else if (**parent_ref).sealed {
                            self.errors.push(Error::new(class_def.loc, SealedInheritance))
                        } else {
                            class_def.parent_ref = *parent_ref;
                        }
                    } else {
                        self.errors.push(Error::new(class_def.loc, ClassNotFound { name: parent }));
                    }
                }
            }
            for class_def in &mut program.classes {
                self.visit_class_def(class_def);
                if class_def.name == MAIN_CLASS {
                    program.main = class_def;
                }
            }
            for class_def in &program.classes {
                self.check_override(class_def);
            }
            if !self.check_main(program.main) {
                self.errors.push(Error::new(NO_LOC, NoMainClass));
            }
        }
    }

    fn visit_class_def(&mut self, class_def: &mut ClassDef) {
        self.class_scope = &mut class_def.symbols;
        for field_def in &mut class_def.fields {
            self.visit_field_def(field_def)
        }
        self.class_scope = ptr::null_mut();
    }

    fn visit_field_def(&mut self, field_def: &mut FieldDef) {
        unsafe {
            let field_def_ptr = field_def as *mut FieldDef;
            match field_def {
                FieldDef::MethodDef(method_def) => {
                    self.visit_type(&mut method_def.return_type);
                    (*self.class_scope).entry(method_def.name)
                        .and_modify(|earlier| {
                            self.errors.push(Error::new(method_def.loc, ConflictDeclaration {
                                earlier: (**earlier).get_loc(),
                                name: method_def.name,
                            }));
                        })
                        .or_insert(field_def_ptr);
                    for var_def in &mut method_def.parameters {
                        self.visit_var_def(var_def);
                    }
                    method_def.body.is_method = true;
                    self.visit_block(&mut method_def.body);
                }
                FieldDef::VarDef(var_def) => self.visit_var_def(var_def),
            }
        }
    }

    fn visit_var_def(&mut self, var_def: &mut VarDef) {
        self.visit_type(&mut var_def.type_);
//        varDef.type.accept(this);
//        if (varDef.type.type.equal(BaseType.VOID)) {
//            issueError(new BadVarTypeError(varDef.getLocation(), varDef.name));
//            // for argList
//            varDef.symbol = new Variable(".error", BaseType.ERROR, varDef
//                .getLocation());
//            return;
//        }
//        Variable v = new Variable(varDef.name, varDef.type.type,
//        varDef.getLocation());
//        Symbol sym = table.lookup(varDef.name, true);
//        if (sym != null) {
//            if (table.getCurrentScope().equals(sym.getScope())) {
//                issueError(new DeclConflictError(v.getLocation(), v.getName(),
//                                                 sym.getLocation()));
//            } else if ((sym.getScope().isFormalScope() && table.getCurrentScope().isLocalScope() && ((LocalScope) table.getCurrentScope()).isCombinedtoFormal())) {
//            issueError(new DeclConflictError(v.getLocation(), v.getName(),
//            sym.getLocation()));
//            } else {
//            table.declare(v);
//            }
//        } else {
//            table.declare(v);
//        }
//        varDef.symbol = v;
    }

    fn visit_block(&mut self, block: &mut Block) {
        unimplemented!()
    }

    fn visit_while(&mut self, while_: &mut While) {
        unimplemented!()
    }

    fn visit_for(&mut self, for_: &mut For) {
        unimplemented!()
    }

    fn visit_if(&mut self, if_: &mut If) {
        unimplemented!()
    }

    fn visit_break(&mut self, break_: &mut Break) {
        unimplemented!()
    }

    fn visit_return(&mut self, return_: &mut Return) {
        unimplemented!()
    }

    fn visit_object_copy(&mut self, object_copy: &mut ObjectCopy) {
        unimplemented!()
    }

    fn visit_foreach(&mut self, foreach: &mut Foreach) {
        unimplemented!()
    }

    fn visit_guarded(&mut self, guarded: &mut Guarded) {
        unimplemented!()
    }

    fn visit_new_class(&mut self, new_class: &mut NewClass) {
        unimplemented!()
    }

    fn visit_new_array(&mut self, new_array: &mut NewArray) {
        unimplemented!()
    }

    fn visit_assign(&mut self, assign: &mut Assign) {
        unimplemented!()
    }

    fn visit_const(&mut self, const_: &mut Const) {
        unimplemented!()
    }

    fn visit_unary(&mut self, unary: &mut Unary) {
        unimplemented!()
    }

    fn visit_call(&mut self, call: &mut Call) {
        unimplemented!()
    }

    fn visit_read_int(&mut self, read_int: &mut ReadInt) {
        unimplemented!()
    }

    fn visit_read_line(&mut self, read_line: &mut ReadLine) {
        unimplemented!()
    }

    fn visit_print(&mut self, print: &mut Print) {
        unimplemented!()
    }

    fn visit_this(&mut self, this: &mut This) {
        unimplemented!()
    }

    fn visit_type_cast(&mut self, type_cast: &mut TypeCast) {
        unimplemented!()
    }

    fn visit_type_test(&mut self, type_test: &mut TypeTest) {
        unimplemented!()
    }

    fn visit_indexed(&mut self, indexed: &mut Indexed) {
        unimplemented!()
    }

    fn visit_identifier(&mut self, identifier: &mut Identifier) {
        unimplemented!()
    }

    fn visit_range(&mut self, range: &mut Range) {
        unimplemented!()
    }

    fn visit_default(&mut self, default: &mut Default) {
        unimplemented!()
    }

    fn visit_comprehension(&mut self, comprehension: &mut Comprehension) {
        unimplemented!()
    }

    fn visit_type(&mut self, type_: &mut Type) {
        unimplemented!()
    }
}