// The resolver does several things together because of their interdependent nature.
// Its primary purpose is verifying that the value type and value category of each expression is correct.
// However, since verifying these for call expressions requires function overload resolution, and for identifiers 
// requires variable binding, it does both of those too. There's no point doing these a second time in a later stage.

use std::collections::HashMap;

use crate::lexer::*;
use crate::ast::*;

pub enum ResExpr
{
    Literal(Literal),
    StackBinding(i64),
    Call(FnName, Vec<ResExpr>)
}

pub enum ResStmt
{
    Decl(ResExpr),
    Expr(ResExpr)
}

pub struct ResAST(pub Vec<ResStmt>);

pub enum ResError<'a>
{
    IdentifierNotFound(&'a [u8]),
    ArgCountMismatch(FnName),
    TypeMismatch(ResExpr),
    ExpectedLvalue(Expr<'a>),
    Redecl(&'a [u8])
}

struct Environment<'a>
{
    vars: HashMap<&'a [u8], (EnttType, i64)>,
    reset_idx: i64,
    next_idx: i64
}

pub struct Resolver<'a>
{
    env_chain: Vec<Environment<'a>>
}

impl<'a> Resolver<'a>
{
    pub fn new() -> Self
    {
        Self {env_chain: vec![Environment {vars:HashMap::new(), reset_idx: -1, next_idx: 0}]}
    }

    pub fn resolve(&mut self, ast: AST<'a>) -> (ResAST, Vec<ResError<'a>>)
    {
        let mut res_ast = ResAST(Vec::new());
        let mut errors = Vec::<ResError<'a>>::new();

        for stmt in ast.0
        {
            match self.resolve_stmt(stmt)
            {
                Ok(res_stmt) => res_ast.0.push(res_stmt),
                Err(err) => errors.push(err)
            }
        }

        (res_ast, errors)
    }

    fn resolve_stmt(&mut self, stmt: Stmt<'a>) -> Result<ResStmt, ResError<'a>>
    {
        match stmt
        {
            Stmt::Decl(typ, iden, expr) => 
            {
                let (res_expr, expr_type) = self.resolve_rvalue(expr)?;
                if expr_type != typ {return Err(ResError::TypeMismatch(res_expr));}

                let env = self.env_chain.last_mut().unwrap();
                match env.vars.get(iden)
                {
                    Some(_) => Err(ResError::Redecl(iden)),
                    None =>
                    {
                        env.vars.insert(iden, (typ, env.next_idx));
                        env.next_idx += 1;
                        Ok(ResStmt::Decl(res_expr))
                    }
                }
            },
            Stmt::Expr(expr) => 
            {
                let (res_expr, _) = self.resolve_rvalue(expr)?;
                Ok(ResStmt::Expr(res_expr))
            },
        }
    }

    fn resolve_rvalue(&self, expr: Expr<'a>) -> Result<(ResExpr, EnttType), ResError<'a>>
    {
        match expr
        {
            Expr::Literal(x) =>
            {
                match x
                {
                    Literal::Int(_) => Ok((ResExpr::Literal(x), EnttType::Compt(ComptType::Int))),
                    Literal::Float(_) => Ok((ResExpr::Literal(x), EnttType::Compt(ComptType::Float))),
                    Literal::Bool(_) => Ok((ResExpr::Literal(x), EnttType::Compt(ComptType::Bool))),
                }
            },
            Expr::Identifier(x) =>
            {
                for env in self.env_chain.iter().rev()
                {
                    match env.vars.get(x)
                    {
                        // TODO: ENTTTYPE REPO
                        Some((typ, idx)) => return Ok((ResExpr::StackBinding(*idx), typ.clone())),
                        None => ()
                    }
                }

                Err(ResError::IdentifierNotFound(x))
            },
            // This arm is mostly AI generated since it was busywork
            Expr::Call(f, mut args) =>
            {
                match f
                {
                    FnName::Negate =>
                    {
                        if args.len() != 1 {return Err(ResError::ArgCountMismatch(f));}
                        let (expr, typ) = self.resolve_rvalue(args.pop().unwrap())?;

                        match typ
                        {
                            EnttType::Compt(ComptType::Int) | EnttType::Compt(ComptType::Float) => (),
                            _ => return Err(ResError::TypeMismatch(expr))
                        }

                        Ok((ResExpr::Call(f, vec![expr]), typ))
                    },
                    FnName::Not => 
                    {
                        if args.len() != 1 {return Err(ResError::ArgCountMismatch(f));}
                        let (expr, typ) = self.resolve_rvalue(args.pop().unwrap())?;

                        match typ
                        {
                            EnttType::Compt(ComptType::Bool) => (),
                            _ => return Err(ResError::TypeMismatch(expr))
                        }

                        Ok((ResExpr::Call(f, vec![expr]), EnttType::Compt(ComptType::Bool)))
                    },
                    FnName::Print => 
                    {
                        if args.len() != 1 {return Err(ResError::ArgCountMismatch(f));}
                        let (expr, _typ) = self.resolve_rvalue(args.pop().unwrap())?;
                        
                        Ok((ResExpr::Call(f, vec![expr]), EnttType::Unit))
                    },
                    FnName::Add | FnName::Sub | FnName::Mul | FnName::Div => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(f));}
                        let (expr2, typ2) = self.resolve_rvalue(args.pop().unwrap())?;
                        let (expr1, typ1) = self.resolve_rvalue(args.pop().unwrap())?;

                        if typ1 != typ2 {return Err(ResError::TypeMismatch(expr2));}

                        match typ1
                        {
                            EnttType::Compt(ComptType::Int) | EnttType::Compt(ComptType::Float) => (),
                            _ => return Err(ResError::TypeMismatch(expr1))
                        }

                        Ok((ResExpr::Call(f, vec![expr1, expr2]), typ1))
                    },
                    FnName::EqualTo | FnName::NotEqualTo => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(f));}
                        let (expr2, typ2) = self.resolve_rvalue(args.pop().unwrap())?;
                        let (expr1, typ1) = self.resolve_rvalue(args.pop().unwrap())?;

                        if typ1 != typ2 {return Err(ResError::TypeMismatch(expr2));}

                        Ok((ResExpr::Call(f, vec![expr1, expr2]), EnttType::Compt(ComptType::Bool)))
                    },
                    FnName::Greater | FnName::Lesser | FnName::GreaterEq | FnName::LesserEq => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(f));}
                        let (expr2, typ2) = self.resolve_rvalue(args.pop().unwrap())?;
                        let (expr1, typ1) = self.resolve_rvalue(args.pop().unwrap())?;

                        if typ1 != typ2 {return Err(ResError::TypeMismatch(expr2));}

                        match typ1
                        {
                            EnttType::Compt(ComptType::Int) | EnttType::Compt(ComptType::Float) => (),
                            _ => return Err(ResError::TypeMismatch(expr1))
                        }

                        Ok((ResExpr::Call(f, vec![expr1, expr2]), EnttType::Compt(ComptType::Bool)))
                    },
                    FnName::And | FnName::Or => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(f));}
                        let (expr2, typ2) = self.resolve_rvalue(args.pop().unwrap())?;
                        let (expr1, typ1) = self.resolve_rvalue(args.pop().unwrap())?;

                        if typ1 != EnttType::Compt(ComptType::Bool) {return Err(ResError::TypeMismatch(expr1));}
                        if typ2 != EnttType::Compt(ComptType::Bool) {return Err(ResError::TypeMismatch(expr2));}

                        Ok((ResExpr::Call(f, vec![expr1, expr2]), EnttType::Compt(ComptType::Bool)))
                    },
                    FnName::Assign => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(f));}
                        
                        let (rhs_expr, rhs_typ) = self.resolve_rvalue(args.pop().unwrap())?;
                        let (lhs_expr, lhs_typ) = self.resolve_lvalue(args.pop().unwrap())?;

                        if lhs_typ != rhs_typ {return Err(ResError::TypeMismatch(rhs_expr));}

                        Ok((ResExpr::Call(f, vec![lhs_expr, rhs_expr]), lhs_typ))
                    },
                    FnName::Ternary => 
                    {
                        if args.len() != 3 {return Err(ResError::ArgCountMismatch(f));}
                        let (cond_expr, cond_typ) = self.resolve_rvalue(args.pop().unwrap())?;
                        
                        if cond_typ != EnttType::Compt(ComptType::Bool) {return Err(ResError::TypeMismatch(cond_expr));}

                        let (false_expr, false_typ) = self.resolve_rvalue(args.pop().unwrap())?;
                        let (true_expr, true_typ) = self.resolve_rvalue(args.pop().unwrap())?;

                        if true_typ != false_typ {return Err(ResError::TypeMismatch(false_expr));}

                        Ok((ResExpr::Call(f, vec![cond_expr, true_expr, false_expr]), true_typ))
                    },
                }
            }
        }
    }

    fn resolve_lvalue(&self, expr: Expr<'a>) -> Result<(ResExpr, EnttType), ResError<'a>>
    {
        match expr
        {
            Expr::Identifier(x) =>
            {
                for env in self.env_chain.iter().rev()
                {
                    match env.vars.get(x)
                    {
                        // TODO: ENTTTYPE REPO
                        Some((typ, idx)) => return Ok((ResExpr::StackBinding(*idx), typ.clone())),
                        None => ()
                    }
                }

                Err(ResError::IdentifierNotFound(x))
            },
            // This arm is AI generated since it was busywork
            Expr::Call(f, mut args) =>
            {
                match f
                {
                    FnName::Assign => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(f));}
                        
                        let (rhs_expr, rhs_typ) = self.resolve_rvalue(args.pop().unwrap())?;
                        let (lhs_expr, lhs_typ) = self.resolve_lvalue(args.pop().unwrap())?;

                        if lhs_typ != rhs_typ {return Err(ResError::TypeMismatch(rhs_expr));}

                        Ok((ResExpr::Call(f, vec![lhs_expr, rhs_expr]), lhs_typ))
                    },
                    FnName::Ternary => 
                    {
                        if args.len() != 3 {return Err(ResError::ArgCountMismatch(f));}
                        let (cond_expr, cond_typ) = self.resolve_rvalue(args.pop().unwrap())?;
                        
                        if cond_typ != EnttType::Compt(ComptType::Bool) {return Err(ResError::TypeMismatch(cond_expr));}

                        let (false_expr, false_typ) = self.resolve_lvalue(args.pop().unwrap())?;
                        let (true_expr, true_typ) = self.resolve_lvalue(args.pop().unwrap())?;

                        if true_typ != false_typ {return Err(ResError::TypeMismatch(false_expr));}

                        Ok((ResExpr::Call(f, vec![cond_expr, true_expr, false_expr]), true_typ))
                    },
                    _ => Err(ResError::ExpectedLvalue(Expr::Call(f, args))),
                }
            },
            Expr::Literal(_) => Err(ResError::ExpectedLvalue(expr)),
        }
    }
}