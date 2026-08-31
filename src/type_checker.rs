use crate::parser::*;

pub enum ExprType
{
    Int, Float, Bool
}

fn unary_reqd_type()

pub enum TypeError
{

}

pub struct TypeChecker
{
    pub errors: Vec<TypeError>
}

impl TypeChecker
{
    pub fn new() -> Self
    {
        Self {errors: Vec::new()}
    }

    pub fn check(&mut self, expr: &Expr)
    {

    }
}