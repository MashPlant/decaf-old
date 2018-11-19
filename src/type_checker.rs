use super::ast::*;
use super::errors::*;
use super::symbol::*;

pub struct TypeChecker {
    errors: Vec<Error>,
    scopes: ScopeStack,
    loops: Vec<*mut Statement>,
    current_method: *mut MethodDef,
}

impl Visitor {
    fn check_binary(&mut self, left: &mut Expr, right: &mut Expr, op: Operator) -> Type {
        self.visit_expr(left);
        self.visit_expr(right);
//        if left.get
    }
}


impl Visitor for TypeChecker {
    fn visit_binary(&mut self, _binary: &mut Binary) {
        unimplemented!()
    }
}