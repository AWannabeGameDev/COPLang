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

pub enum ResOp
{
    Negate, Not, Print,
    Add, Sub, Mul, Div,
    EqualTo, NotEqualTo,
    Greater, Lesser, GreaterEq, LesserEq,
    And, Or,
    Assign,
    Ternary,
    Func(usize)
}

fn res_op(op: &Operation) -> ResOp
{
    match op
    {
        Operation::Negate => ResOp::Negate,
        Operation::Not => ResOp::Not,
        Operation::Print => ResOp::Print,
        Operation::Add => ResOp::Add,
        Operation::Sub => ResOp::Sub,
        Operation::Mul => ResOp::Mul,
        Operation::Div => ResOp::Div,
        Operation::EqualTo => ResOp::EqualTo,
        Operation::NotEqualTo => ResOp::NotEqualTo,
        Operation::Greater => ResOp::Greater,
        Operation::Lesser => ResOp::Lesser,
        Operation::GreaterEq => ResOp::GreaterEq,
        Operation::LesserEq => ResOp::LesserEq,
        Operation::And => ResOp::And,
        Operation::Or => ResOp::Or,
        Operation::Assign => ResOp::Assign,
        Operation::Ternary => ResOp::Ternary,
        Operation::Func(_) => unreachable!()
    }
}

pub enum ResExprEnum
{
    Literal(Literal),
    StackBinding(isize),
    Op(ResOp, Vec<ResExpr>)
}

#[derive(Copy, Clone, PartialEq)]
pub enum ValCat {Lvalue, Rvalue}

pub struct ResExpr
{
    pub data: ResExprEnum,
    pub typ: VarType,
    pub cat: ValCat
}

pub struct ResBlock(pub Vec<ResStmt>);

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
    Cond(ResIfElse),
    Iter(ResExpr, ResBlock),
    Break, Continue,
    FnDecl(usize)
}

pub enum ResError<'s>
{
    IdentifierNotFound(Range<usize>, &'s [u8]),
    ArgCountMismatch(Range<usize>),
    TypeMismatch(Range<usize>, VarType),
    ExpectedLvalue(Range<usize>),
    Redecl(Range<usize>),
    OnlyInLoop(Range<usize>)
}

struct Environment<'s>
{
    vars: HashMap<&'s [u8], (VarType, usize)>,
    funcs: HashMap<&'s [u8], (Vec<VarType>, VarType, usize)>,
    base: usize,
    next_var_idx: usize,
}

pub struct Resolver<'s>
{
    loop_depth: usize,
    func_depth: usize,
    out_scope_limit: usize,
    env_chain: Vec<Environment<'s>>,
    pub res_funcs: Vec<ResBlock>,
    pub errors: Vec<ResError<'s>>
}

impl<'s, 'p> Resolver<'s>
{
    pub fn new() -> Self
    {
        Self {loop_depth: 0, func_depth: 0, out_scope_limit: 0, env_chain: Vec::new(), res_funcs: Vec::new(), errors: Vec::new()}
    }

    fn get_var_base_idx(&self, id: &[u8]) -> Option<(VarType, isize)>
    {
        for env in self.env_chain.iter().rev().take(self.out_scope_limit)
        {
            let Some((var_typ, idx)) = env.vars.get(id) else {continue};
            return Some((*var_typ, *idx as isize - self.env_chain.last().unwrap().base as isize));
        }
        None
    }

    fn get_fn_idx(&self, id: &[u8]) -> Option<&(Vec<VarType>, VarType, usize)>
    {
        for env in self.env_chain.iter().rev()
        {
            let Some(ret) = env.funcs.get(id) else {continue};
            return Some(ret)
        }
        None
    }

    pub fn resolve(&mut self, block: &StmtBlock<'s>) -> ResBlock
    {
        self.resolve_block(block, HashMap::new())
    }

    fn resolve_block(&mut self, ast: &'p StmtBlock<'s>, vars: HashMap<&'s [u8], (VarType, usize)>) -> ResBlock
    {
        let base = self.env_chain.last().map(|env| env.next_var_idx).unwrap_or(0);
        let next_var_idx = base + vars.len();
        self.env_chain.push(Environment {vars, funcs: HashMap::new(), next_var_idx, base});
        self.out_scope_limit += 1;
        let mut res_block = ResBlock(Vec::new());

        for stmt in ast.0.iter()
        {
            if stmt.data == StmtEnum::Error {continue}

            match self.resolve_stmt(stmt)
            {
                Ok(res_stmt) => res_block.0.push(res_stmt),
                Err(err) => self.errors.push(err)
            }
        }

        self.env_chain.pop();
        res_block
    }

    fn resolve_cond(&mut self, if_stmt: &'p IfElseBlock<'s>) -> Result<ResIfElse, ResError<'s>>
    {
        let cond = self.resolve_rvalue(&if_stmt.cond)?;
        if cond.typ != VarType::Atom(AtomType::Bool) {return Err(ResError::TypeMismatch(if_stmt.cond.span, cond.typ))}
        let block = self.resolve_block(&if_stmt.block, HashMap::new());
        let els = self.resolve_else(&if_stmt.els)?;
        Ok(ResIfElse {cond, block, els})
    }

    fn resolve_else(&mut self, else_stmt: &'p ElseBlock<'s>) -> Result<ResElse, ResError<'s>>
    {
        match else_stmt
        {
            ElseBlock::None => Ok(ResElse::None),
            ElseBlock::Else(block) => Ok(ResElse::Else(self.resolve_block(block, HashMap::new()))),
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
                        env.vars.insert(iden, (*typ, env.next_var_idx));
                        env.next_var_idx += 1;
                        Ok(ResStmt::Decl(res_expr))
                    }
                }
            },
            StmtEnum::FnDecl(id, params, out_typ, block) =>
            {
                if *out_typ != VarType::Unit {todo!()}

                let env = self.env_chain.last_mut().unwrap();
                match env.funcs.get(id)
                {
                    Some(_) => Err(ResError::Redecl(stmt.span)),
                    None =>
                    {
                        let mut vars = HashMap::<&'s [u8], (VarType, usize)>::new();
                        let mut param_typs = Vec::<VarType>::new();
                        for (idx, param) in params.iter().enumerate()
                        {
                            vars.insert(param.0, (param.1, env.next_var_idx + idx));
                            param_typs.push(param.1);
                        }

                        env.funcs.insert(id, (param_typs, *out_typ, self.res_funcs.len()));

                        self.func_depth += 1;
                        let old_limit = self.out_scope_limit;
                        self.out_scope_limit = 0;
                        let res_block = self.resolve_block(block, vars);
                        self.out_scope_limit = old_limit;
                        self.func_depth -= 1;

                        self.res_funcs.push(res_block);

                        Ok(ResStmt::FnDecl(self.res_funcs.len() - 1))
                    }
                }
            },
            StmtEnum::Expr(expr) => Ok(ResStmt::Expr(self.resolve_rvalue(&expr)?)),
            StmtEnum::Block(block) => Ok(ResStmt::Block(self.resolve_block(block, HashMap::new()))),
            StmtEnum::Cond(if_stmt) => Ok(ResStmt::Cond(self.resolve_cond(if_stmt)?)),
            StmtEnum::Iter(cond, block) =>
            {
                let res_cond = self.resolve_rvalue(cond)?;
                if res_cond.typ != VarType::Atom(AtomType::Bool) {return Err(ResError::TypeMismatch(cond.span, res_cond.typ))}
                
                self.loop_depth += 1;
                let res_block = self.resolve_block(block, HashMap::new());
                self.loop_depth -= 1;

                Ok(ResStmt::Iter(res_cond, res_block))
            },
            StmtEnum::Break => if self.loop_depth > 0 {Ok(ResStmt::Break)} else {Err(ResError::OnlyInLoop(stmt.span))},
            StmtEnum::Continue => if self.loop_depth > 0 {Ok(ResStmt::Continue)} else {Err(ResError::OnlyInLoop(stmt.span))},
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
            ExprEnum::Identifier(id) => match self.get_var_base_idx(id)
            {
                Some((typ, off)) => return Ok(ResExpr {data: ResExprEnum::StackBinding(off), typ, cat: ValCat::Rvalue}),
                None => return Err(ResError::IdentifierNotFound(expr.span, id))
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
                    data = ResExprEnum::Op(res_op(f), vec![res_expr]);
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
                    data = ResExprEnum::Op(res_op(f), vec![res_expr]);
                },
                Operation::Print => 
                {
                    if args.len() != 1 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let res_expr = self.resolve_rvalue(&args[0])?;
                    
                    typ = VarType::Unit;
                    data = ResExprEnum::Op(res_op(f), vec![res_expr]);
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
                    data = ResExprEnum::Op(res_op(f), vec![expr1, expr2]);
                },
                Operation::EqualTo | Operation::NotEqualTo => 
                {
                    if args.len() != 2 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let expr1 = self.resolve_rvalue(&args[0])?;
                    let expr2 = self.resolve_rvalue(&args[1])?;

                    if expr1.typ != expr2.typ {return Err(ResError::TypeMismatch(args[1].span, expr2.typ));}

                    typ = VarType::Atom(AtomType::Bool);
                    data = ResExprEnum::Op(res_op(f), vec![expr1, expr2]);
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
                    data = ResExprEnum::Op(res_op(f), vec![expr1, expr2]);
                },
                Operation::And | Operation::Or => 
                {
                    if args.len() != 2 {return Err(ResError::ArgCountMismatch(expr.span));}
                    let expr1 = self.resolve_rvalue(&args[0])?;
                    let expr2 = self.resolve_rvalue(&args[1])?;

                    if expr1.typ != VarType::Atom(AtomType::Bool) {return Err(ResError::TypeMismatch(args[0].span, expr1.typ));}
                    if expr2.typ != VarType::Atom(AtomType::Bool) {return Err(ResError::TypeMismatch(args[1].span, expr2.typ));}

                    typ = VarType::Atom(AtomType::Bool);
                    data = ResExprEnum::Op(res_op(f), vec![expr1, expr2]);
                },
                Operation::Assign => 
                {
                    if args.len() != 2 {return Err(ResError::ArgCountMismatch(expr.span));}
                    
                    let lhs_expr = self.resolve_lvalue(&args[0])?;
                    let rhs_expr = self.resolve_rvalue(&args[1])?;

                    if lhs_expr.typ != rhs_expr.typ {return Err(ResError::TypeMismatch(args[1].span, rhs_expr.typ));}

                    typ = lhs_expr.typ;
                    data = ResExprEnum::Op(res_op(f), vec![lhs_expr, rhs_expr]);
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
                    data = ResExprEnum::Op(res_op(f), vec![cond_expr, true_expr, false_expr]);
                },
                Operation::Func(id) =>
                {
                    let Some((param_typs, out_typ, res_idx)) = self.get_fn_idx(id) 
                        else {return Err(ResError::IdentifierNotFound(expr.span, id))};
                    if args.len() != param_typs.len() {return Err(ResError::ArgCountMismatch(expr.span))}

                    let mut res_args = Vec::<ResExpr>::new();
                    for (idx, arg) in args.iter().enumerate()
                    {
                        let res_arg = self.resolve_rvalue(arg)?;
                        if res_arg.typ != param_typs[idx] {return Err(ResError::TypeMismatch(arg.span, res_arg.typ))}
                        res_args.push(res_arg);
                    }

                    typ = *out_typ;
                    data = ResExprEnum::Op(ResOp::Func(*res_idx), res_args);
                }
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
            ExprEnum::Identifier(id) => match self.get_var_base_idx(id)
            {
                Some((typ, off)) => return Ok(ResExpr {data: ResExprEnum::StackBinding(off), typ, cat: ValCat::Lvalue}),
                None => return Err(ResError::IdentifierNotFound(expr.span, id))
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
                    data = ResExprEnum::Op(res_op(f), vec![lhs_expr, rhs_expr]);
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
                    data = ResExprEnum::Op(res_op(f), vec![cond_expr, true_expr, false_expr]);
                },
                _ => return Err(ResError::ExpectedLvalue(expr.span)),
            },
            ExprEnum::Literal(_) => return Err(ResError::ExpectedLvalue(expr.span)),
        }
        
        Ok(ResExpr {data, typ, cat: ValCat::Lvalue})
    }
}