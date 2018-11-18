use super::ast::*;

struct BuildSymbol {}

impl Visitor for BuildSymbol {
    fn visit_program(&mut self, program: &mut Program) {
//        program.globalScope = new GlobalScope();
//        table.open(program.globalScope);
//        for (Tree.ClassDef cd : program.classes) {
//            Class c = new Class(cd.name, cd.parent, cd.isSealed, cd.getLocation());
//            Class earlier = table.lookupClass(cd.name);
//            if (earlier != null) {
//                issueError(new DeclConflictError(cd.getLocation(), cd.name,
//                                                 earlier.getLocation()));
//            } else {
//                table.declare(c);
//            }
//            cd.symbol = c;
//        }
//
//        for (Tree.ClassDef cd : program.classes) {
//            Class c = cd.symbol;
//            if (cd.parent != null && c.getParent() == null) {
//                issueError(new ClassNotFoundError(cd.getLocation(), cd.parent));
//                c.dettachParent();
//            }
//            Class parent = c.getParent();
//            if (calcOrder(c) <= calcOrder(parent)) {
//                issueError(new BadInheritanceError(cd.getLocation()));
//                c.dettachParent();
//            }
//            if (parent != null && parent.isSealed) {
//                issueError(new BadSealedInherError(cd.getLocation()));
//            }
//        }
//
//        for (Tree.ClassDef cd : program.classes) {
//            cd.symbol.createType();
//        }
//
//        for (Tree.ClassDef cd : program.classes) {
//            cd.accept(this);
//            if (Driver.getDriver().getOption().getMainClassName().equals(
//                cd.name)) {
//                program.main = cd.symbol;
//            }
//        }
//
//        for (Tree.ClassDef cd : program.classes) {
//            checkOverride(cd.symbol);
//        }
//
//        if (!isMainClass(program.main)) {
//            issueError(new NoMainClassError(Driver.getDriver().getOption()
//                .getMainClassName()));
//        }
//        table.close();
    }

    fn visit_class_def(&mut self, class_def: &mut ClassDef) {
        unimplemented!()
    }

    fn visit_method_def(&mut self, method_def: &mut MethodDef) {
        unimplemented!()
    }

    fn visit_simple(&mut self, simple: &mut Simple) {
        unimplemented!()
    }

    fn visit_var_def(&mut self, var_def: &mut VarDef) {
        unimplemented!()
    }

    fn visit_skip(&mut self, skip: &mut Skip) {
        unimplemented!()
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

    fn visit_lvalue(&mut self, lvalue: &mut LValue) {
        unimplemented!()
    }

    fn visit_const(&mut self, const_: &mut Const) {
        unimplemented!()
    }

    fn visit_unary(&mut self, unary: &mut Unary) {
        unimplemented!()
    }

    fn visit_binary(&mut self, binary: &mut Binary) {
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