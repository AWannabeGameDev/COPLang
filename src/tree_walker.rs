use crate::lexer::*;
use crate::parser::*;
use crate::resolver::*;

pub enum ExprResult
{
    Literal(Literal),
    Unit
}

pub enum RuntimeError
{
    DivByZero(FnName)
}

pub fn interpret(ast: &ResExpr) -> Result<ExprResult, RuntimeError>
{
    match &ast.expr
    {
        HalfResExpr::Literal(x) => Ok(ExprResult::Literal(*x)),
        HalfResExpr::Call(f, args) => 
        {
            match f
            {
                FnName::Negate => 
                {
                    match interpret(&args[0])?
                    {
                        ExprResult::Literal(Literal::Int(v)) => Ok(ExprResult::Literal(Literal::Int(-v))),
                        ExprResult::Literal(Literal::Float(v)) => Ok(ExprResult::Literal(Literal::Float(-v))),
                        _ => unreachable!(),
                    }
                },
                FnName::Not => 
                {
                    match interpret(&args[0])?
                    {
                        ExprResult::Literal(Literal::Bool(v)) => Ok(ExprResult::Literal(Literal::Bool(!v))),
                        _ => unreachable!(),
                    }
                },
                FnName::Add => 
                {
                    match (interpret(&args[0])?, interpret(&args[1])?)
                    {
                        (ExprResult::Literal(Literal::Int(a)), ExprResult::Literal(Literal::Int(b))) => Ok(ExprResult::Literal(Literal::Int(a + b))),
                        (ExprResult::Literal(Literal::Float(a)), ExprResult::Literal(Literal::Float(b))) => Ok(ExprResult::Literal(Literal::Float(a + b))),
                        _ => unreachable!(),
                    }
                },
                FnName::Sub => 
                {
                    match (interpret(&args[0])?, interpret(&args[1])?)
                    {
                        (ExprResult::Literal(Literal::Int(a)), ExprResult::Literal(Literal::Int(b))) => Ok(ExprResult::Literal(Literal::Int(a - b))),
                        (ExprResult::Literal(Literal::Float(a)), ExprResult::Literal(Literal::Float(b))) => Ok(ExprResult::Literal(Literal::Float(a - b))),
                        _ => unreachable!(),
                    }
                },
                FnName::Mul => 
                {
                    match (interpret(&args[0])?, interpret(&args[1])?)
                    {
                        (ExprResult::Literal(Literal::Int(a)), ExprResult::Literal(Literal::Int(b))) => Ok(ExprResult::Literal(Literal::Int(a * b))),
                        (ExprResult::Literal(Literal::Float(a)), ExprResult::Literal(Literal::Float(b))) => Ok(ExprResult::Literal(Literal::Float(a * b))),
                        _ => unreachable!(),
                    }
                },
                FnName::Div => 
                {
                    match (interpret(&args[0])?, interpret(&args[1])?)
                    {
                        (ExprResult::Literal(Literal::Int(a)), ExprResult::Literal(Literal::Int(b))) => 
                        {
                            if b == 0 {return Err(RuntimeError::DivByZero(*f));}
                            Ok(ExprResult::Literal(Literal::Int(a / b)))
                        },
                        (ExprResult::Literal(Literal::Float(a)), ExprResult::Literal(Literal::Float(b))) => 
                        {
                            if b == 0.0 {return Err(RuntimeError::DivByZero(*f));}
                            Ok(ExprResult::Literal(Literal::Float(a / b)))
                        },
                        _ => unreachable!(),
                    }
                },
                FnName::EqualTo => 
                {
                    match (interpret(&args[0])?, interpret(&args[1])?)
                    {
                        (ExprResult::Literal(Literal::Int(a)), ExprResult::Literal(Literal::Int(b))) => Ok(ExprResult::Literal(Literal::Bool(a == b))),
                        (ExprResult::Literal(Literal::Float(a)), ExprResult::Literal(Literal::Float(b))) => Ok(ExprResult::Literal(Literal::Bool(a == b))),
                        (ExprResult::Literal(Literal::Bool(a)), ExprResult::Literal(Literal::Bool(b))) => Ok(ExprResult::Literal(Literal::Bool(a == b))),
                        _ => unreachable!(),
                    }
                },
                FnName::NotEqualTo => 
                {
                    match (interpret(&args[0])?, interpret(&args[1])?)
                    {
                        (ExprResult::Literal(Literal::Int(a)), ExprResult::Literal(Literal::Int(b))) => Ok(ExprResult::Literal(Literal::Bool(a != b))),
                        (ExprResult::Literal(Literal::Float(a)), ExprResult::Literal(Literal::Float(b))) => Ok(ExprResult::Literal(Literal::Bool(a != b))),
                        (ExprResult::Literal(Literal::Bool(a)), ExprResult::Literal(Literal::Bool(b))) => Ok(ExprResult::Literal(Literal::Bool(a != b))),
                        _ => unreachable!(),
                    }
                },
                FnName::Greater => 
                {
                    match (interpret(&args[0])?, interpret(&args[1])?)
                    {
                        (ExprResult::Literal(Literal::Int(a)), ExprResult::Literal(Literal::Int(b))) => Ok(ExprResult::Literal(Literal::Bool(a > b))),
                        (ExprResult::Literal(Literal::Float(a)), ExprResult::Literal(Literal::Float(b))) => Ok(ExprResult::Literal(Literal::Bool(a > b))),
                        _ => unreachable!(),
                    }
                },
                FnName::Lesser => 
                {
                    match (interpret(&args[0])?, interpret(&args[1])?)
                    {
                        (ExprResult::Literal(Literal::Int(a)), ExprResult::Literal(Literal::Int(b))) => Ok(ExprResult::Literal(Literal::Bool(a < b))),
                        (ExprResult::Literal(Literal::Float(a)), ExprResult::Literal(Literal::Float(b))) => Ok(ExprResult::Literal(Literal::Bool(a < b))),
                        _ => unreachable!(),
                    }
                },
                FnName::GreaterEq => 
                {
                    match (interpret(&args[0])?, interpret(&args[1])?)
                    {
                        (ExprResult::Literal(Literal::Int(a)), ExprResult::Literal(Literal::Int(b))) => Ok(ExprResult::Literal(Literal::Bool(a >= b))),
                        (ExprResult::Literal(Literal::Float(a)), ExprResult::Literal(Literal::Float(b))) => Ok(ExprResult::Literal(Literal::Bool(a >= b))),
                        _ => unreachable!(),
                    }
                },
                FnName::LesserEq => 
                {
                    match (interpret(&args[0])?, interpret(&args[1])?)
                    {
                        (ExprResult::Literal(Literal::Int(a)), ExprResult::Literal(Literal::Int(b))) => Ok(ExprResult::Literal(Literal::Bool(a <= b))),
                        (ExprResult::Literal(Literal::Float(a)), ExprResult::Literal(Literal::Float(b))) => Ok(ExprResult::Literal(Literal::Bool(a <= b))),
                        _ => unreachable!(),
                    }
                },
                FnName::And => 
                {
                    match interpret(&args[0])?
                    {
                        ExprResult::Literal(Literal::Bool(false)) => Ok(ExprResult::Literal(Literal::Bool(false))),
                        ExprResult::Literal(Literal::Bool(true)) => interpret(&args[1]),
                        _ => unreachable!(),
                    }
                },
                FnName::Or => 
                {
                    match interpret(&args[0])?
                    {
                        ExprResult::Literal(Literal::Bool(true)) => Ok(ExprResult::Literal(Literal::Bool(true))),
                        ExprResult::Literal(Literal::Bool(false)) => interpret(&args[1]),
                        _ => unreachable!(),
                    }
                },
                FnName::DiscardLeft => 
                {
                    interpret(&args[0])?;
                    interpret(&args[1])
                },
                FnName::Ternary => 
                {
                    match interpret(&args[0])?
                    {
                        ExprResult::Literal(Literal::Bool(true)) => interpret(&args[1]),
                        ExprResult::Literal(Literal::Bool(false)) => interpret(&args[2]),
                        _ => unreachable!(),
                    }
                },
            }
        },
        HalfResExpr::Empty => Ok(ExprResult::Unit),
    }
}