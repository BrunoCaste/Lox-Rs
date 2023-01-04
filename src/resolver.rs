use std::collections::HashMap;

use crate::{
    expr::{Expr, Variable},
    prog::Prog,
    stmt::Stmt,
};

#[derive(Clone, Copy, PartialEq)]
enum FunctionType {
    None,
    Function,
}

pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    curr_function: FunctionType,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            curr_function: FunctionType::None,
        }
    }

    fn declare(&mut self, var: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(var) {
                println!("Already a variable with this name in this scope");
                todo!("resolver must return results");
            }
            scope.insert(var.to_string(), false);
        }
    }

    fn define(&mut self, var: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(var.to_string(), true);
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn resolve(&mut self, ast: &mut Prog) {
        for s in &mut ast.stmts {
            self.resolve_stmt(s);
        }
    }

    fn resolve_stmt(&mut self, s: &mut Stmt) {
        match s {
            Stmt::Block(body) => {
                self.begin_scope();
                for s in body {
                    self.resolve_stmt(s);
                }
                self.end_scope();
            }
            Stmt::Expr(expr) | Stmt::Print(expr) => self.resolve_expr(expr),
            Stmt::Decl(var, init) => {
                self.declare(var);
                if let Some(e) = init {
                    self.resolve_expr(e);
                }
                self.define(var);
            }
            Stmt::If(cond, then_b, else_b) => {
                self.resolve_expr(cond);
                self.resolve_stmt(then_b);
                if let Some(else_b) = else_b {
                    self.resolve_stmt(else_b);
                }
            }
            Stmt::While(cond, body) => {
                self.resolve_expr(cond);
                self.resolve_stmt(body);
            }
            Stmt::Func(name, params, body) => {
                self.define(name);
                let enclosing_function = self.curr_function;
                self.curr_function = FunctionType::Function;
                self.begin_scope();
                for p in params {
                    self.define(p);
                }
                self.resolve_stmt(body);
                self.end_scope();
                self.curr_function = enclosing_function;
            }
            Stmt::Return(ret) => {
                if self.curr_function == FunctionType::None {
                    println!("Can't return from top-level code");
                    todo!("resolver must return results");
                }
                if let Some(expr) = ret {
                    self.resolve_expr(expr)
                }
            }
        }
    }

    fn resolve_expr(&mut self, e: &mut Expr) {
        match e {
            Expr::Asgn(var, expr) => {
                self.resolve_expr(expr);
                self.resolve_local(var);
            }
            Expr::Call(callee, args) => {
                self.resolve_expr(callee);
                for a in args {
                    self.resolve_expr(a)
                }
            }
            Expr::And(lhs, rhs)
            | Expr::Or(lhs, rhs)
            | Expr::Eq(lhs, rhs)
            | Expr::Ne(lhs, rhs)
            | Expr::Gt(lhs, rhs)
            | Expr::Ge(lhs, rhs)
            | Expr::Lt(lhs, rhs)
            | Expr::Le(lhs, rhs)
            | Expr::Add(lhs, rhs)
            | Expr::Sub(lhs, rhs)
            | Expr::Mul(lhs, rhs)
            | Expr::Div(lhs, rhs) => {
                self.resolve_expr(lhs);
                self.resolve_expr(rhs)
            }
            Expr::Not(arg) | Expr::Opp(arg) => self.resolve_expr(arg),
            Expr::Lit(_) => {}
            Expr::Var(var) => {
                if self
                    .scopes
                    .last()
                    .and_then(|sc| sc.get(var.name.as_ref()))
                    .is_some_and(|&val| !val)
                {
                    println!("Can't read local variable in its own initializer");
                    todo!("resolver must return results");
                }
                self.resolve_local(var);
            }
        }
    }

    fn resolve_local(&mut self, var: &mut Variable) {
        if let Some((i, _)) = self
            .scopes
            .iter()
            .rev()
            .enumerate()
            .find(|(_, scope)| scope.contains_key(&*var.name))
        {
            var.depth = i as isize;
        };
    }
}
