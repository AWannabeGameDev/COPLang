use std::range::Range;

use crate::lexer::*;

// In the future add a variant for custom functions
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Operation<'s>
{
    Negate, Not, Print,
    Add, Sub, Mul, Div,
    EqualTo, NotEqualTo,
    Greater, Lesser, GreaterEq, LesserEq,
    And, Or,
    Assign,
    Ternary,
    Func(&'s [u8])
}

#[derive(PartialEq, Debug)]
pub enum ExprEnum<'s>
{
    Literal(Literal),
    Identifier(&'s [u8]),
    Op(Operation<'s>, Vec<Expr<'s>>)
}

#[derive(PartialEq, Debug)]
pub struct Expr<'s>
{
    pub span: Range<usize>,
    pub data: ExprEnum<'s>
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AtomType
{
    Int, Float, Bool
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum VarType
{
    Atom(AtomType),
    Unit
}

#[derive(PartialEq, Debug)]
pub struct StmtBlock<'s>(pub Vec<Stmt<'s>>);

#[derive(PartialEq, Debug)]
pub struct IfElseBlock<'s>
{
    pub cond: Expr<'s>,
    pub block: StmtBlock<'s>,
    pub els: ElseBlock<'s>
}

#[derive(PartialEq, Debug)]
pub enum ElseBlock<'s>
{
    None,
    Else(StmtBlock<'s>),
    ElseIf(Box<IfElseBlock<'s>>)
}

#[derive(PartialEq, Debug)]
pub enum StmtEnum<'s>
{
    Decl(VarType, &'s [u8], Expr<'s>),
    FnDecl(&'s [u8], Vec<(&'s [u8], VarType)>, VarType, StmtBlock<'s>),
    Expr(Expr<'s>),
    Block(StmtBlock<'s>),
    Cond(IfElseBlock<'s>),
    Iter(Expr<'s>, StmtBlock<'s>),
    Break, Continue,
    Error
}

#[derive(PartialEq, Debug)]
pub struct Stmt<'s>
{
    pub span: Range<usize>,
    pub data: StmtEnum<'s>
}