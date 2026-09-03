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

pub struct ResAST(Vec<ResStmt>);

pub enum ResError<'a>
{
    IdentifierNotFound(&'a [u8]),
    ArgCountMismatch(FnName),
    ArgTypeMismatch(FnName, i64),
    ExpectedLvalue(Expr<'a>)
}

struct Environment<'a>
{
    vars: HashMap<&'a [u8], (EnttType, i64)>,
    reset_idx: i64,
    next_idx: i64
}

pub struct Resolver<'a>
{
    pub ast: ResAST,
    pub errors: Vec<ResError<'a>>,
    env_chain: Vec<Environment<'a>>
}

impl<'a> Resolver<'a>
{
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
                        Some((typ, idx)) => return Ok((ResExpr::StackBinding(*idx), typ.clone())),
                        None => ()
                    }
                }

                Err(ResError::IdentifierNotFound(x))
            },
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
                            _ => return Err(ResError::ArgTypeMismatch(f, 0))
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
                            _ => return Err(ResError::ArgTypeMismatch(f, 0))
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

                        if typ1 != typ2 {return Err(ResError::ArgTypeMismatch(f, 1));}

                        match typ1
                        {
                            EnttType::Compt(ComptType::Int) | EnttType::Compt(ComptType::Float) => (),
                            _ => return Err(ResError::ArgTypeMismatch(f, 0))
                        }

                        Ok((ResExpr::Call(f, vec![expr1, expr2]), typ1))
                    },
                    FnName::EqualTo | FnName::NotEqualTo => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(f));}
                        let (expr2, typ2) = self.resolve_rvalue(args.pop().unwrap())?;
                        let (expr1, typ1) = self.resolve_rvalue(args.pop().unwrap())?;

                        if typ1 != typ2 {return Err(ResError::ArgTypeMismatch(f, 1));}

                        Ok((ResExpr::Call(f, vec![expr1, expr2]), EnttType::Compt(ComptType::Bool)))
                    },
                    FnName::Greater | FnName::Lesser | FnName::GreaterEq | FnName::LesserEq => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(f));}
                        let (expr2, typ2) = self.resolve_rvalue(args.pop().unwrap())?;
                        let (expr1, typ1) = self.resolve_rvalue(args.pop().unwrap())?;

                        if typ1 != typ2 {return Err(ResError::ArgTypeMismatch(f, 1));}

                        match typ1
                        {
                            EnttType::Compt(ComptType::Int) | EnttType::Compt(ComptType::Float) => (),
                            _ => return Err(ResError::ArgTypeMismatch(f, 0))
                        }

                        Ok((ResExpr::Call(f, vec![expr1, expr2]), EnttType::Compt(ComptType::Bool)))
                    },
                    FnName::And | FnName::Or => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(f));}
                        let (expr2, typ2) = self.resolve_rvalue(args.pop().unwrap())?;
                        let (expr1, typ1) = self.resolve_rvalue(args.pop().unwrap())?;

                        if typ1 != EnttType::Compt(ComptType::Bool) {return Err(ResError::ArgTypeMismatch(f, 0));}
                        if typ2 != EnttType::Compt(ComptType::Bool) {return Err(ResError::ArgTypeMismatch(f, 1));}

                        Ok((ResExpr::Call(f, vec![expr1, expr2]), EnttType::Compt(ComptType::Bool)))
                    },
                    FnName::Assign => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(f));}
                        
                        let (rhs_expr, rhs_typ) = self.resolve_rvalue(args.pop().unwrap())?;
                        let (lhs_expr, lhs_typ) = self.resolve_lvalue(args.pop().unwrap())?;

                        if lhs_typ != rhs_typ {return Err(ResError::ArgTypeMismatch(f, 1));}

                        Ok((ResExpr::Call(f, vec![lhs_expr, rhs_expr]), lhs_typ))
                    },
                    FnName::Ternary => 
                    {
                        if args.len() != 3 {return Err(ResError::ArgCountMismatch(f));}
                        let (cond_expr, cond_typ) = self.resolve_rvalue(args.pop().unwrap())?;
                        
                        if cond_typ != EnttType::Compt(ComptType::Bool) {return Err(ResError::ArgTypeMismatch(f, 0));}

                        let (false_expr, false_typ) = self.resolve_rvalue(args.pop().unwrap())?;
                        let (true_expr, true_typ) = self.resolve_rvalue(args.pop().unwrap())?;

                        if true_typ != false_typ {return Err(ResError::ArgTypeMismatch(f, 2));}

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
                        Some((typ, idx)) => return Ok((ResExpr::StackBinding(*idx), typ.clone())),
                        None => ()
                    }
                }

                Err(ResError::IdentifierNotFound(x))
            },
            Expr::Call(f, mut args) =>
            {
                match f
                {
                    FnName::Assign => 
                    {
                        if args.len() != 2 {return Err(ResError::ArgCountMismatch(f));}
                        
                        let (rhs_expr, rhs_typ) = self.resolve_rvalue(args.pop().unwrap())?;
                        let (lhs_expr, lhs_typ) = self.resolve_lvalue(args.pop().unwrap())?;

                        if lhs_typ != rhs_typ {return Err(ResError::ArgTypeMismatch(f, 1));}

                        Ok((ResExpr::Call(f, vec![lhs_expr, rhs_expr]), lhs_typ))
                    },
                    FnName::Ternary => 
                    {
                        if args.len() != 3 {return Err(ResError::ArgCountMismatch(f));}
                        let (cond_expr, cond_typ) = self.resolve_rvalue(args.pop().unwrap())?;
                        
                        if cond_typ != EnttType::Compt(ComptType::Bool) {return Err(ResError::ArgTypeMismatch(f, 0));}

                        // For a ternary to be a valid L-value, BOTH branches must be valid L-values
                        let (false_expr, false_typ) = self.resolve_lvalue(args.pop().unwrap())?;
                        let (true_expr, true_typ) = self.resolve_lvalue(args.pop().unwrap())?;

                        if true_typ != false_typ {return Err(ResError::ArgTypeMismatch(f, 2));}

                        Ok((ResExpr::Call(f, vec![cond_expr, true_expr, false_expr]), true_typ))
                    },
                    // Any other function call strictly produces an R-value
                    _ => Err(ResError::ExpectedLvalue(Expr::Call(f, args))),
                }
            },
            // Literals have no memory address
            Expr::Literal(_) => Err(ResError::ExpectedLvalue(expr)),
        }
    }
}