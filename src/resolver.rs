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

pub enum ResError<'a, 'b>
{
    IdentifierNotFound(&'a [u8]),
    ArgCountMismatch(FnName),
    TypeMismatch(&'b Expr<'a>),
    ExpectedLvalue(&'b Expr<'a>),
    Redecl(&'a [u8])
}

struct Environment<'a, 'b>
{
    vars: HashMap<&'a [u8], (&'b EnttType, i64)>,
    reset_idx: i64,
    next_idx: i64
}

pub struct Resolver<'a, 'b>
{
    env_chain: Vec<Environment<'a, 'b>>
}

impl<'a, 'b> Resolver<'a, 'b>
{
    pub fn new() -> Self
    {
        Self {env_chain: vec![Environment {vars:HashMap::new(), reset_idx: -1, next_idx: 0}]}
    }

    pub fn resolve(&mut self, ast: &'b AST<'a>) -> (ResAST, Vec<ResError<'a, 'b>>)
    {
        let mut res_ast = ResAST(Vec::new());
        let mut errors = Vec::<ResError<'a, 'b>>::new();

        for stmt in ast.0.iter()
        {
            match self.resolve_stmt(stmt)
            {
                Ok(res_stmt) => res_ast.0.push(res_stmt),
                Err(err) => errors.push(err)
            }
        }

        (res_ast, errors)
    }

    fn resolve_stmt(&mut self, stmt: &'b Stmt<'a>) -> Result<ResStmt, ResError<'a, 'b>>
    {
        match stmt
        {
            Stmt::Decl(typ, iden, expr) => 
            {
                let (res_expr, expr_type) = self.resolve_rvalue(&expr)?;
                if expr_type != typ {return Err(ResError::TypeMismatch(expr));}

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
                let (res_expr, _) = self.resolve_rvalue(&expr)?;
                Ok(ResStmt::Expr(res_expr))
            },
        }
    }

    fn resolve_rvalue(&self, expr: &'b Expr<'a>) -> Result<(ResExpr, &EnttType), ResError<'a, 'b>>
    {
        match expr
        {
            Expr::Literal(x) =>
            {
                match x
                {
                    Literal::Int(_) => Ok((ResExpr::Literal(*x), &EnttType::Compt(ComptType::Int))),
                    Literal::Float(_) => Ok((ResExpr::Literal(*x), &EnttType::Compt(ComptType::Float))),
                    Literal::Bool(_) => Ok((ResExpr::Literal(*x), &EnttType::Compt(ComptType::Bool))),
                }
            },
            Expr::Identifier(x) =>
            {
                for env in self.env_chain.iter().rev()
                {
                    if let Some((typ, idx)) = env.vars.get(x) 
                    {
                        return Ok((ResExpr::StackBinding(*idx), typ));
                    }
                }
                Err(ResError::IdentifierNotFound(x))
            },
            Expr::Call(f, args) =>
            {
                match f
                {
                    FnName::Negate =>
                    {
                        if args.len() != 1 {return Err(ResError::ArgCountMismatch(*f));}
                        let (res_expr, typ) = self.resolve_rvalue(&args[0])?;

                        match typ
                        {
                            EnttType::Compt(ComptType::Int) | EnttType::Compt(ComptType::Float) => (),
                            _ => return Err(ResError::TypeMismatch(&args[0]))
                        }

                        Ok((ResExpr::Call(*f, vec![res_expr]), typ))
                    },
                    FnName::Not => 
                    {
                        if args.len() != 1 {return Err(ResError::ArgCountMismatch(*f));}
                        let (res_expr, typ) = self.resolve_rvalue(&args[0])?;

                        match typ
                        {
                            EnttType::Compt(ComptType::Bool) => (),
                            _ => return Err(ResError::TypeMismatch(&args[0]))
                        }

                        Ok((ResExpr::Call(*f, vec![res_expr]), &EnttType::Compt(ComptType::Bool)))
                    },
                    FnName::Print => 
                    {
                        if args.len() != 1 {return Err(ResError::ArgCountMismatch(*f));}
                        let (res_expr, _typ) = self.resolve_rvalue(&args[0])?;
                        
                        Ok((ResExpr::Call(*f, vec![res_expr]), &EnttType::Unit))
                    },
                    FnName::Add | FnName::Sub | FnName::Mul | FnName::Div => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(*f));}
                        let (expr1, typ1) = self.resolve_rvalue(&args[0])?;
                        let (expr2, typ2) = self.resolve_rvalue(&args[1])?;

                        if typ1 != typ2 {return Err(ResError::TypeMismatch(&args[1]));}

                        match typ1
                        {
                            EnttType::Compt(ComptType::Int) | EnttType::Compt(ComptType::Float) => (),
                            _ => return Err(ResError::TypeMismatch(&args[0]))
                        }

                        Ok((ResExpr::Call(*f, vec![expr1, expr2]), typ1))
                    },
                    FnName::EqualTo | FnName::NotEqualTo => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(*f));}
                        let (expr1, typ1) = self.resolve_rvalue(&args[0])?;
                        let (expr2, typ2) = self.resolve_rvalue(&args[1])?;

                        if typ1 != typ2 {return Err(ResError::TypeMismatch(&args[1]));}

                        Ok((ResExpr::Call(*f, vec![expr1, expr2]), &EnttType::Compt(ComptType::Bool)))
                    },
                    FnName::Greater | FnName::Lesser | FnName::GreaterEq | FnName::LesserEq => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(*f));}
                        let (expr1, typ1) = self.resolve_rvalue(&args[0])?;
                        let (expr2, typ2) = self.resolve_rvalue(&args[1])?;

                        if typ1 != typ2 {return Err(ResError::TypeMismatch(&args[1]));}

                        match typ1
                        {
                            EnttType::Compt(ComptType::Int) | EnttType::Compt(ComptType::Float) => (),
                            _ => return Err(ResError::TypeMismatch(&args[0]))
                        }

                        Ok((ResExpr::Call(*f, vec![expr1, expr2]), &EnttType::Compt(ComptType::Bool)))
                    },
                    FnName::And | FnName::Or => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(*f));}
                        let (expr1, typ1) = self.resolve_rvalue(&args[0])?;
                        let (expr2, typ2) = self.resolve_rvalue(&args[1])?;

                        if *typ1 != EnttType::Compt(ComptType::Bool) {return Err(ResError::TypeMismatch(&args[0]));}
                        if *typ2 != EnttType::Compt(ComptType::Bool) {return Err(ResError::TypeMismatch(&args[1]));}

                        Ok((ResExpr::Call(*f, vec![expr1, expr2]), &EnttType::Compt(ComptType::Bool)))
                    },
                    FnName::Assign => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(*f));}
                        
                        let (lhs_expr, lhs_typ) = self.resolve_lvalue(&args[0])?;
                        let (rhs_expr, rhs_typ) = self.resolve_rvalue(&args[1])?;

                        if lhs_typ != rhs_typ {return Err(ResError::TypeMismatch(&args[1]));}

                        Ok((ResExpr::Call(*f, vec![lhs_expr, rhs_expr]), lhs_typ))
                    },
                    FnName::Ternary => 
                    {
                        if args.len() != 3 {return Err(ResError::ArgCountMismatch(*f));}
                        let (cond_expr, cond_typ) = self.resolve_rvalue(&args[0])?;
                        
                        if *cond_typ != EnttType::Compt(ComptType::Bool) {return Err(ResError::TypeMismatch(&args[0]));}

                        let (true_expr, true_typ) = self.resolve_rvalue(&args[1])?;
                        let (false_expr, false_typ) = self.resolve_rvalue(&args[2])?;

                        if true_typ != false_typ {return Err(ResError::TypeMismatch(&args[2]));}

                        Ok((ResExpr::Call(*f, vec![cond_expr, true_expr, false_expr]), true_typ))
                    },
                }
            }
        }
    }

    fn resolve_lvalue(&self, expr: &'b Expr<'a>) -> Result<(ResExpr, &EnttType), ResError<'a, 'b>>
    {
        match expr
        {
            Expr::Identifier(x) =>
            {
                for env in self.env_chain.iter().rev()
                {
                    if let Some((typ, idx)) = env.vars.get(x) {
                        return Ok((ResExpr::StackBinding(*idx), typ));
                    }
                }
                Err(ResError::IdentifierNotFound(x))
            },
            Expr::Call(f, args) =>
            {
                match f
                {
                    FnName::Assign => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(*f));}
                        
                        let (lhs_expr, lhs_typ) = self.resolve_lvalue(&args[0])?;
                        let (rhs_expr, rhs_typ) = self.resolve_rvalue(&args[1])?;

                        if lhs_typ != rhs_typ {return Err(ResError::TypeMismatch(&args[1]));}

                        Ok((ResExpr::Call(*f, vec![lhs_expr, rhs_expr]), lhs_typ))
                    },
                    FnName::Ternary => 
                    {
                        if args.len() != 3 {return Err(ResError::ArgCountMismatch(*f));}
                        let (cond_expr, cond_typ) = self.resolve_rvalue(&args[0])?;
                        
                        if *cond_typ != EnttType::Compt(ComptType::Bool) {return Err(ResError::TypeMismatch(&args[0]));}

                        let (true_expr, true_typ) = self.resolve_lvalue(&args[1])?;
                        let (false_expr, false_typ) = self.resolve_lvalue(&args[2])?;

                        if true_typ != false_typ {return Err(ResError::TypeMismatch(&args[2]));}

                        Ok((ResExpr::Call(*f, vec![cond_expr, true_expr, false_expr]), true_typ))
                    },
                    _ => Err(ResError::ExpectedLvalue(expr)),
                }
            },
            Expr::Literal(_) => Err(ResError::ExpectedLvalue(expr)),
        }
    }
}