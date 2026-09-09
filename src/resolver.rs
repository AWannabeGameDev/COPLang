// The resolver does several things together because of their interdependent nature.
// Its primary purpose is verifying that the value type and value category of each expression is correct.
// However, since verifying these for Op expressions requires function overload resolution, and for identifiers 
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
    Op(Operation, Vec<ResExpr>)
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

pub struct ResIfElse
{
    pub cond: ResExpr,
    pub block: ResBlock,
    pub els: ResElse
}

pub enum ResElse
{
    None,
    Else(ResBlock),
    ElseIf(Box<ResIfElse>)
}

pub enum ResStmt
{
    Decl(ResExpr),
    Expr(ResExpr),
    Block(ResBlock),
    Cond(ResIfElse)
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
    env_chain: Vec<Environment<'s>>,
    pub errors: Vec<ResError<'s>>
}

impl<'s, 'p> Resolver<'s>
{
    pub fn new() -> Self
    {
        Self {env_chain: Vec::new(), errors: Vec::new()}
    }

    pub fn resolve_block(&mut self, ast: &'p StmtBlock<'s>) -> ResBlock
    {
        self.env_chain.push(Environment {vars: HashMap::new(), next_idx: self.env_chain.last().map(|env| env.next_idx).unwrap_or(0)});
        let mut res_block = ResBlock {stmts: Vec::new(), base: self.env_chain.last().unwrap().next_idx};

        for stmt in ast.0.iter()
        {
            if stmt.data == StmtEnum::Error {continue}

            match self.resolve_stmt(stmt)
            {
                Ok(res_stmt) => res_block.stmts.push(res_stmt),
                Err(err) => self.errors.push(err)
            }
        }

        self.env_chain.pop();
        res_block
    }

    fn resolve_cond(&mut self, if_stmt: &IfElseBlock<'s>) -> Result<ResIfElse, ResError<'s>>
    {
        let cond = self.resolve_rvalue(&if_stmt.cond)?;
        if cond.typ != VarType::Atom(AtomType::Bool) {return Err(ResError::TypeMismatch(if_stmt.cond.span, cond.typ))}
        let block = self.resolve_block(&if_stmt.block);
        let els = self.resolve_else(&if_stmt.els)?;
        Ok(ResIfElse {cond, block, els})
    }

    fn resolve_else(&mut self, else_stmt: &ElseBlock<'s>) -> Result<ResElse, ResError<'s>>
    {
        match else_stmt
        {
            ElseBlock::None => Ok(ResElse::None),
            ElseBlock::Else(block) => Ok(ResElse::Else(self.resolve_block(block))),
            ElseBlock::ElseIf(box_if) => Ok(ResElse::ElseIf(Box::new(self.resolve_cond(box_if.as_ref())?)))
        }
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
            StmtEnum::Expr(expr) => Ok(ResStmt::Expr(self.resolve_rvalue(&expr)?)),
            StmtEnum::Block(block) => Ok(ResStmt::Block(self.resolve_block(block))),
            StmtEnum::Cond(if_stmt) => Ok(ResStmt::Cond(self.resolve_cond(if_stmt)?)),
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
            ExprEnum::Op(f, args) => match f
            {
                Operation::Negate =>
                {
                    if args.len() != 1 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let res_expr = self.resolve_rvalue(&args[0])?;

                    match res_expr.typ
                    {
                        VarType::Atom(AtomType::Int) | VarType::Atom(AtomType::Float) => (),
                        _ => return Err(ResError::TypeMismatch(args[0].span, res_expr.typ))
                    }

                    typ = res_expr.typ;
                    data = ResExprEnum::Op(*f, vec![res_expr]);
                },
                Operation::Not => 
                {
                    if args.len() != 1 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let res_expr = self.resolve_rvalue(&args[0])?;

                    match res_expr.typ
                    {
                        VarType::Atom(AtomType::Bool) => (),
                        _ => return Err(ResError::TypeMismatch(args[0].span, res_expr.typ))
                    }

                    typ = VarType::Atom(AtomType::Bool);
                    data = ResExprEnum::Op(*f, vec![res_expr]);
                },
                Operation::Print => 
                {
                    if args.len() != 1 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let res_expr = self.resolve_rvalue(&args[0])?;
                    
                    typ = VarType::Unit;
                    data = ResExprEnum::Op(*f, vec![res_expr]);
                },
                Operation::Add | Operation::Sub | Operation::Mul | Operation::Div => 
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
                    data = ResExprEnum::Op(*f, vec![expr1, expr2]);
                },
                Operation::EqualTo | Operation::NotEqualTo => 
                {
                    if args.len() != 2 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let expr1 = self.resolve_rvalue(&args[0])?;
                    let expr2 = self.resolve_rvalue(&args[1])?;

                    if expr1.typ != expr2.typ {return Err(ResError::TypeMismatch(args[1].span, expr2.typ));}

                    typ = VarType::Atom(AtomType::Bool);
                    data = ResExprEnum::Op(*f, vec![expr1, expr2]);
                },
                Operation::Greater | Operation::Lesser | Operation::GreaterEq | Operation::LesserEq => 
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
                    data = ResExprEnum::Op(*f, vec![expr1, expr2]);
                },
                Operation::And | Operation::Or => 
                {
                    if args.len() != 2 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let expr1 = self.resolve_rvalue(&args[0])?;
                    let expr2 = self.resolve_rvalue(&args[1])?;

                    if expr1.typ != VarType::Atom(AtomType::Bool) {return Err(ResError::TypeMismatch(args[0].span, expr1.typ));}
                    if expr2.typ != VarType::Atom(AtomType::Bool) {return Err(ResError::TypeMismatch(args[1].span, expr2.typ));}

                    typ = VarType::Atom(AtomType::Bool);
                    data = ResExprEnum::Op(*f, vec![expr1, expr2]);
                },
                Operation::Assign => 
                {
                    if args.len() != 2 {return Err(ResError::ArgCountMismatch(expr.span));}
                    
                    let lhs_expr = self.resolve_lvalue(&args[0])?;
                    let rhs_expr = self.resolve_rvalue(&args[1])?;

                    if lhs_expr.typ != rhs_expr.typ {return Err(ResError::TypeMismatch(args[1].span, rhs_expr.typ));}

                    typ = lhs_expr.typ;
                    data = ResExprEnum::Op(*f, vec![lhs_expr, rhs_expr]);
                },
                Operation::Ternary => 
                {
                    if args.len() != 3 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let cond_expr = self.resolve_rvalue(&args[0])?;
                    
                    if cond_expr.typ != VarType::Atom(AtomType::Bool) {return Err(ResError::TypeMismatch(args[0].span, cond_expr.typ));}

                    let true_expr = self.resolve_rvalue(&args[1])?;
                    let false_expr = self.resolve_rvalue(&args[2])?;

                    if true_expr.typ != false_expr.typ {return Err(ResError::TypeMismatch(args[2].span, false_expr.typ));}

                    typ = true_expr.typ;
                    data = ResExprEnum::Op(*f, vec![cond_expr, true_expr, false_expr]);
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
            ExprEnum::Op(f, args) => match f
            {
                Operation::Assign => 
                {
                    if args.len() != 2 {return Err(ResError::ArgCountMismatch(expr.span));}
                    
                    let lhs_expr = self.resolve_lvalue(&args[0])?;
                    let rhs_expr = self.resolve_rvalue(&args[1])?;

                    if lhs_expr.typ != rhs_expr.typ {return Err(ResError::TypeMismatch(args[1].span, rhs_expr.typ));}

                    typ = lhs_expr.typ;
                    data = ResExprEnum::Op(*f, vec![lhs_expr, rhs_expr]);
                },
                Operation::Ternary => 
                {
                    if args.len() != 3 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let cond_expr = self.resolve_rvalue(&args[0])?;
                    
                    if cond_expr.typ != VarType::Atom(AtomType::Bool) {return Err(ResError::TypeMismatch(args[0].span, cond_expr.typ));}

                    let true_expr = self.resolve_lvalue(&args[1])?;
                    let false_expr = self.resolve_lvalue(&args[2])?;

                    if true_expr.typ != false_expr.typ {return Err(ResError::TypeMismatch(args[2].span, false_expr.typ));}

                    typ = true_expr.typ;
                    data = ResExprEnum::Op(*f, vec![cond_expr, true_expr, false_expr]);
                },
                _ => return Err(ResError::ExpectedLvalue(expr.span)),
            },
            ExprEnum::Literal(_) => return Err(ResError::ExpectedLvalue(expr.span)),
        }
        
        Ok(ResExpr {data, typ, cat: ValCat::Lvalue})
    }
}