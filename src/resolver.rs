// The resolver does several things together because of their interdependent nature.
// Its primary purpose is verifying that the value type and value category of each expression is correct.
// However, since verifying these for call expressions requires function overload resolution, and for identifiers 
// requires variable binding, it does both of those too. There's no point doing these a second time in a later stage.

// Lifetime 's is for references to the source code.
// Lifetime 'p is for references to the previous AST

use std::collections::HashMap;
use std::range::Range;

use crate::lexer::*;
use crate::ast::*;

pub enum ResExprEnum
{
    Literal(Literal),
    StackBinding(usize),
    Call(FnName, Vec<ResExpr>)
}

#[derive(Copy, Clone, PartialEq)]
pub enum ValCat {Lvalue, Rvalue}

pub struct ResExpr
{
    pub data: ResExprEnum,
    pub typ: VarType,
    pub cat: ValCat
}

pub struct ResBlock
{
    pub stmts: Vec<ResStmt>,
    pub base: usize
}

pub enum ResStmt
{
    Decl(ResExpr),
    Expr(ResExpr),
    Block(ResBlock)
}

pub enum ResError<'s>
{
    IdentifierNotFound(Range<usize>, &'s [u8]),
    ArgCountMismatch(Range<usize>),
    TypeMismatch(Range<usize>, VarType),
    ExpectedLvalue(Range<usize>),
    Redecl(Range<usize>)
}

struct Environment<'s>
{
    vars: HashMap<&'s [u8], (VarType, usize)>,
    next_idx: usize
}

pub struct Resolver<'s>
{
    env_chain: Vec<Environment<'s>>
}

impl<'s, 'p> Resolver<'s>
{
    pub fn new() -> Self
    {
        Self {env_chain: Vec::new()}
    }

    pub fn resolve_block(&mut self, ast: &'p StmtBlock<'s>) -> (ResBlock, Vec<ResError<'s>>)
    {
        self.env_chain.push(Environment {vars: HashMap::new(), next_idx: self.env_chain.last().map(|env| env.next_idx).unwrap_or(0)});
        let mut res_block = ResBlock {stmts: Vec::new(), base: self.env_chain.last().unwrap().next_idx};
        let mut errors = Vec::<ResError<'s>>::new();

        for stmt in ast.0.iter()
        {
            if stmt.data == StmtEnum::Error {continue}

            match self.resolve_stmt(stmt)
            {
                Ok(res_stmt) => res_block.stmts.push(res_stmt),
                Err(err) => errors.push(err)
            }
        }

        self.env_chain.pop();
        (res_block, errors)
    }

    fn resolve_stmt(&mut self, stmt: &'p Stmt<'s>) -> Result<ResStmt, ResError<'s>>
    {
        match &stmt.data
        {
            StmtEnum::Decl(typ, iden, expr) => 
            {
                let res_expr = self.resolve_rvalue(expr)?;
                if res_expr.typ != *typ {return Err(ResError::TypeMismatch(expr.span, res_expr.typ));}

                let env = self.env_chain.last_mut().unwrap();
                match env.vars.get(iden)
                {
                    Some(_) => Err(ResError::Redecl(stmt.span)),
                    None =>
                    {
                        env.vars.insert(iden, (*typ, env.next_idx));
                        env.next_idx += 1;
                        Ok(ResStmt::Decl(res_expr))
                    }
                }
            },
            StmtEnum::Expr(expr) => 
            {
                let res_expr = self.resolve_rvalue(&expr)?;
                Ok(ResStmt::Expr(res_expr))
            },
            StmtEnum::Block(block) => Ok(ResStmt::Block(self.resolve_block(block).0)),
            StmtEnum::Error => unreachable!()
        }
    }

    fn resolve_rvalue(&self, expr: &'p Expr<'s>) -> Result<ResExpr, ResError<'s>>
    {
        let data: ResExprEnum;
        let typ: VarType;
        match &expr.data
        {
            ExprEnum::Literal(x) => match x
            {
                Literal::Int(_) => {data = ResExprEnum::Literal(*x); typ = VarType::Atom(AtomType::Int)},
                Literal::Float(_) => {data = ResExprEnum::Literal(*x); typ = VarType::Atom(AtomType::Float)},
                Literal::Bool(_) => {data = ResExprEnum::Literal(*x); typ = VarType::Atom(AtomType::Bool)}
            },
            ExprEnum::Identifier(x) =>
            {
                for env in self.env_chain.iter().rev()
                {
                    if let Some((var_typ, idx)) = env.vars.get(x) 
                    {
                        return Ok(ResExpr {data: ResExprEnum::StackBinding(*idx), typ: *var_typ, cat: ValCat::Rvalue});
                    }
                }
                return Err(ResError::IdentifierNotFound(expr.span, x));
            },
            ExprEnum::Call(f, args) => match f
            {
                FnName::Negate =>
                {
                    if args.len() != 1 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let res_expr = self.resolve_rvalue(&args[0])?;

                    match res_expr.typ
                    {
                        VarType::Atom(AtomType::Int) | VarType::Atom(AtomType::Float) => (),
                        _ => return Err(ResError::TypeMismatch(args[0].span, res_expr.typ))
                    }

                    typ = res_expr.typ;
                    data = ResExprEnum::Call(*f, vec![res_expr]);
                },
                FnName::Not => 
                {
                    if args.len() != 1 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let res_expr = self.resolve_rvalue(&args[0])?;

                    match res_expr.typ
                    {
                        VarType::Atom(AtomType::Bool) => (),
                        _ => return Err(ResError::TypeMismatch(args[0].span, res_expr.typ))
                    }

                    typ = VarType::Atom(AtomType::Bool);
                    data = ResExprEnum::Call(*f, vec![res_expr]);
                },
                FnName::Print => 
                {
                    if args.len() != 1 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let res_expr = self.resolve_rvalue(&args[0])?;
                    
                    typ = VarType::Unit;
                    data = ResExprEnum::Call(*f, vec![res_expr]);
                },
                FnName::Add | FnName::Sub | FnName::Mul | FnName::Div => 
                {
                    if args.len() != 2 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let expr1 = self.resolve_rvalue(&args[0])?;
                    let expr2 = self.resolve_rvalue(&args[1])?;

                    if expr1.typ != expr2.typ {return Err(ResError::TypeMismatch(args[1].span, expr2.typ));}

                    match expr1.typ
                    {
                        VarType::Atom(AtomType::Int) | VarType::Atom(AtomType::Float) => (),
                        _ => return Err(ResError::TypeMismatch(args[0].span, expr1.typ))
                    }

                    typ = expr1.typ;
                    data = ResExprEnum::Call(*f, vec![expr1, expr2]);
                },
                FnName::EqualTo | FnName::NotEqualTo => 
                {
                    if args.len() != 2 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let expr1 = self.resolve_rvalue(&args[0])?;
                    let expr2 = self.resolve_rvalue(&args[1])?;

                    if expr1.typ != expr2.typ {return Err(ResError::TypeMismatch(args[1].span, expr2.typ));}

                    typ = VarType::Atom(AtomType::Bool);
                    data = ResExprEnum::Call(*f, vec![expr1, expr2]);
                },
                FnName::Greater | FnName::Lesser | FnName::GreaterEq | FnName::LesserEq => 
                {
                    if args.len() != 2 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let expr1 = self.resolve_rvalue(&args[0])?;
                    let expr2 = self.resolve_rvalue(&args[1])?;

                    if expr1.typ != expr2.typ {return Err(ResError::TypeMismatch(args[1].span, expr2.typ));}

                    match expr1.typ
                    {
                        VarType::Atom(AtomType::Int) | VarType::Atom(AtomType::Float) => (),
                        _ => return Err(ResError::TypeMismatch(args[0].span, expr1.typ))
                    }

                    typ = VarType::Atom(AtomType::Bool);
                    data = ResExprEnum::Call(*f, vec![expr1, expr2]);
                },
                FnName::And | FnName::Or => 
                {
                    if args.len() != 2 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let expr1 = self.resolve_rvalue(&args[0])?;
                    let expr2 = self.resolve_rvalue(&args[1])?;

                    if expr1.typ != VarType::Atom(AtomType::Bool) {return Err(ResError::TypeMismatch(args[0].span, expr1.typ));}
                    if expr2.typ != VarType::Atom(AtomType::Bool) {return Err(ResError::TypeMismatch(args[1].span, expr2.typ));}

                    typ = VarType::Atom(AtomType::Bool);
                    data = ResExprEnum::Call(*f, vec![expr1, expr2]);
                },
                FnName::Assign => 
                {
                    if args.len() != 2 {return Err(ResError::ArgCountMismatch(expr.span));}
                    
                    let lhs_expr = self.resolve_lvalue(&args[0])?;
                    let rhs_expr = self.resolve_rvalue(&args[1])?;

                    if lhs_expr.typ != rhs_expr.typ {return Err(ResError::TypeMismatch(args[1].span, rhs_expr.typ));}

                    typ = lhs_expr.typ;
                    data = ResExprEnum::Call(*f, vec![lhs_expr, rhs_expr]);
                },
                FnName::Ternary => 
                {
                    if args.len() != 3 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let cond_expr = self.resolve_rvalue(&args[0])?;
                    
                    if cond_expr.typ != VarType::Atom(AtomType::Bool) {return Err(ResError::TypeMismatch(args[0].span, cond_expr.typ));}

                    let true_expr = self.resolve_rvalue(&args[1])?;
                    let false_expr = self.resolve_rvalue(&args[2])?;

                    if true_expr.typ != false_expr.typ {return Err(ResError::TypeMismatch(args[2].span, false_expr.typ));}

                    typ = true_expr.typ;
                    data = ResExprEnum::Call(*f, vec![cond_expr, true_expr, false_expr]);
                },
            }
        }

        Ok(ResExpr {data, typ, cat: ValCat::Rvalue})
    }

    fn resolve_lvalue(&self, expr: &'p Expr<'s>) -> Result<ResExpr, ResError<'s>>
    {
        let data: ResExprEnum;
        let typ: VarType;
        match &expr.data
        {
            ExprEnum::Identifier(x) =>
            {
                for env in self.env_chain.iter().rev()
                {
                    if let Some((var_typ, idx)) = env.vars.get(x) 
                    {
                        return Ok(ResExpr {data: ResExprEnum::StackBinding(*idx), typ: *var_typ, cat: ValCat::Lvalue});
                    }
                }

                return Err(ResError::IdentifierNotFound(expr.span, x));
            },
            ExprEnum::Call(f, args) => match f
            {
                FnName::Assign => 
                {
                    if args.len() != 2 {return Err(ResError::ArgCountMismatch(expr.span));}
                    
                    let lhs_expr = self.resolve_lvalue(&args[0])?;
                    let rhs_expr = self.resolve_rvalue(&args[1])?;

                    if lhs_expr.typ != rhs_expr.typ {return Err(ResError::TypeMismatch(args[1].span, rhs_expr.typ));}

                    typ = lhs_expr.typ;
                    data = ResExprEnum::Call(*f, vec![lhs_expr, rhs_expr]);
                },
                FnName::Ternary => 
                {
                    if args.len() != 3 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let cond_expr = self.resolve_rvalue(&args[0])?;
                    
                    if cond_expr.typ != VarType::Atom(AtomType::Bool) {return Err(ResError::TypeMismatch(args[0].span, cond_expr.typ));}

                    let true_expr = self.resolve_lvalue(&args[1])?;
                    let false_expr = self.resolve_lvalue(&args[2])?;

                    if true_expr.typ != false_expr.typ {return Err(ResError::TypeMismatch(args[2].span, false_expr.typ));}

                    typ = true_expr.typ;
                    data = ResExprEnum::Call(*f, vec![cond_expr, true_expr, false_expr]);
                },
                _ => return Err(ResError::ExpectedLvalue(expr.span)),
            },
            ExprEnum::Literal(_) => return Err(ResError::ExpectedLvalue(expr.span)),
        }
        
        Ok(ResExpr {data, typ, cat: ValCat::Lvalue})
    }
}