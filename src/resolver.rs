use crate::lexer::*;
use crate::parser::*;

// In the future add a variant for custom types
#[derive(Copy, Clone, PartialEq)]
pub enum ExprType
{
    Int, Float, Bool, Unit
}

pub enum HalfResExpr
{
    Literal(Literal),
    // In the future, build another struct "ResFnName" which resolves overloads
    Call(FnName, Vec<ResExpr>),
    Empty
}

pub struct ResExpr
{
    pub expr: HalfResExpr,
    pub typ: ExprType
}

pub enum TypeError
{
    ArgCountMismatch(FnName),
    ArgTypeMismatch(FnName)
}

pub struct Resolver
{
    pub ast: ResExpr,
    pub errors: Vec<TypeError>
}

impl Resolver
{
    pub fn new() -> Self
    {
        Self {ast: ResExpr{expr: HalfResExpr::Empty, typ: ExprType::Unit}, errors: Vec::new()}
    }

    pub fn check(&mut self, expr: &Expr)
    {
        match self.annotate(expr)
        {
            Ok(expr) => self.ast = expr,
            Err(err) => self.errors.push(err)
        }
    }

    fn annotate(&mut self, expr: &Expr) -> Result<ResExpr, TypeError>
    {
        assert!(!matches!(expr, Expr::Empty));

        match expr
        {
            Expr::Literal(x) =>
            {
                let typ = match x
                {
                    Literal::Int(_) => ExprType::Int,
                    Literal::Float(_) => ExprType::Float,
                    Literal::Bool(_) => ExprType::Bool,
                };

                Ok(ResExpr {expr: HalfResExpr::Literal(*x), typ})
            },
            Expr::Call(f, args) =>
            {
                let res_args: Vec<_> = args.iter().filter_map(|e|
                    match self.annotate(e) 
                    {
                        Ok(re) => Some(re), 
                        Err(err) => 
                        {
                            self.errors.push(err); 
                            None
                        }
                    }
                ).collect();

                self.resolve_fn(*f, res_args)
            },
            Expr::Empty => panic!()
        }
    }

    // rewrite for overload resolution in the future
    fn resolve_fn(&mut self, f: FnName, args: Vec<ResExpr>) -> Result<ResExpr, TypeError>
    {
        match f
        {
            FnName::Negate => 
            {
                if args.len() != 1 {return Err(TypeError::ArgCountMismatch(f));}
                match args[0].typ
                {
                    ExprType::Int | ExprType::Float =>
                    {
                        let typ = args[0].typ;
                        Ok(ResExpr {expr: HalfResExpr::Call(f, args), typ})
                    },
                    _ => Err(TypeError::ArgTypeMismatch(f))
                }
            },
            FnName::Not => 
            {
                if args.len() != 1 {return Err(TypeError::ArgCountMismatch(f));}
                if args[0].typ != ExprType::Bool {return Err(TypeError::ArgTypeMismatch(f));}
                Ok(ResExpr {expr: HalfResExpr::Call(f, args), typ: ExprType::Bool})
            },
            FnName::Add | FnName::Sub | FnName::Mul | FnName::Div => 
            {
                if args.len() != 2 {return Err(TypeError::ArgCountMismatch(f));}
                match (args[0].typ, args[1].typ)
                {
                    (ExprType::Int, ExprType::Int) => Ok(ResExpr {expr: HalfResExpr::Call(f, args), typ: ExprType::Int}),
                    (ExprType::Float, ExprType::Float) => Ok(ResExpr {expr: HalfResExpr::Call(f, args), typ: ExprType::Float}),
                    _ => Err(TypeError::ArgTypeMismatch(f))
                }
            },
            FnName::Greater | FnName::Lesser | FnName::GreaterEq | FnName::LesserEq | FnName::EqualTo | FnName::NotEqualTo => 
            {
                if args.len() != 2 {return Err(TypeError::ArgCountMismatch(f));}
                match (args[0].typ, args[1].typ)
                {
                    (ExprType::Int, ExprType::Int) | (ExprType::Float, ExprType::Float) =>
                        Ok(ResExpr {expr: HalfResExpr::Call(f, args), typ: ExprType::Bool}),
                    _ => Err(TypeError::ArgTypeMismatch(f))
                }
            },
            FnName::And | FnName::Or => 
            {
                if args.len() != 2 {return Err(TypeError::ArgCountMismatch(f));}
                match (args[0].typ, args[1].typ)
                {
                    (ExprType::Bool, ExprType::Bool) => Ok(ResExpr {expr: HalfResExpr::Call(f, args), typ: ExprType::Bool}),
                    _ => Err(TypeError::ArgTypeMismatch(f))
                }
            },
            FnName::DiscardLeft => 
            {
                if args.len() != 2 {return Err(TypeError::ArgCountMismatch(f));}
                let typ = args[1].typ;
                Ok(ResExpr {expr: HalfResExpr::Call(f, args), typ})
            },
            FnName::Ternary => 
            {
                if args.len() != 3 {return Err(TypeError::ArgCountMismatch(f));}
                if args[0].typ != ExprType::Bool {return Err(TypeError::ArgTypeMismatch(f));}
                if args[1].typ != args[2].typ {return Err(TypeError::ArgTypeMismatch(f));}
                
                let typ = args[1].typ;
                Ok(ResExpr {expr: HalfResExpr::Call(f, args), typ})
            },
        }
    }
}