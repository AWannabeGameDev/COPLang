use crate::lexer::*;

// In the future add a variant for custom functions
#[derive(Copy, Clone, Debug)]
pub enum FnName
{
    Negate, Not, Print,
    Add, Sub, Mul, Div,
    EqualTo, NotEqualTo,
    Greater, Lesser, GreaterEq, LesserEq,
    And, Or,
    Assign,
    Ternary
}

#[derive(Debug)]
pub enum Expr<'a>
{
    Literal(Literal),
    Identifier(&'a [u8]),
    Call(FnName, Vec<Expr<'a>>)
}

#[derive(Copy, Clone, Debug)]
pub enum ComptType
{
    Int, Float, Bool
}

#[derive(Debug)]
pub struct EnttType(pub Vec<ComptType>);

#[derive(Debug)]
pub enum Stmt<'a>
{
    Decl(EnttType, &'a [u8], Expr<'a>),
    Expr(Expr<'a>)
}

#[derive(Debug)]
pub struct AST<'a>(pub Vec<Stmt<'a>>);