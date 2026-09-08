use crate::lexer::*;
use crate::ast::*;
use crate::resolver::*;

pub struct TreeWalker
{
    stack: Vec<*mut [u8]>
}

impl TreeWalker
{
    pub fn new() -> Self {Self {stack: Vec::new()}}

    pub fn execute_block(&mut self, block: &ResBlock)
    {
        for stmt in block.stmts.iter()
        {
            match stmt
            {
                ResStmt::Decl(expr) => {let eval = self.eval(expr); self.stack.push(eval)},
                ResStmt::Expr(expr) => 
                {
                    let val = self.eval(expr);
                    if expr.typ != VarType::Unit {unsafe {drop(Box::from_raw(val))}}
                },
                ResStmt::Block(nest_block) => self.execute_block(nest_block),
            }
        }

        while self.stack.len() > block.base {unsafe {drop(Box::from_raw(self.stack.pop().unwrap()));}}
    }

    fn eval(&mut self, expr: &ResExpr) -> *mut [u8]
    {
        match &expr.data
        {
            ResExprEnum::Literal(lit) => match lit
            {
                Literal::Int(x) => Box::into_raw(Box::new(x.to_ne_bytes())),
                Literal::Float(x) => Box::into_raw(Box::new(x.to_ne_bytes())),
                Literal::Bool(x) => Box::into_raw(Box::new([*x as u8])),
            },
            ResExprEnum::StackBinding(idx) => match &expr.cat
            {
                ValCat::Lvalue => self.stack[*idx],
                ValCat::Rvalue => unsafe {Box::into_raw((&*self.stack[*idx]).to_vec().into_boxed_slice())}
            },
            ResExprEnum::Call(f, args) => match f
            {
                FnName::Negate =>
                {
                    let ret = self.eval(&args[0]);
                    match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(ret as *mut i64) = -*(ret as *mut i64);},
                        VarType::Atom(AtomType::Float) => unsafe {*(ret as *mut f64) = -*(ret as *mut f64);},
                        _ => unreachable!()
                    }

                    ret
                },
                FnName::Not => 
                {
                    let ret = self.eval(&args[0]);
                    unsafe {*(ret as *mut u8) = (*(ret as *mut u8) == 0) as u8;}

                    ret
                },
                FnName::Print => 
                {
                    let arg = self.eval(&args[0]);
                    match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {println!("{}", *(arg as *mut i64))},
                        VarType::Atom(AtomType::Float) => unsafe {println!("{}", *(arg as *mut f64))},
                        VarType::Atom(AtomType::Bool) => unsafe {println!("{}", *(arg as *mut u8) != 0)},
                        _ => unreachable!()
                    };

                    unsafe {drop(Box::from_raw(arg));}
                    std::ptr::slice_from_raw_parts_mut(std::ptr::null_mut(), 0)
                },
                FnName::Add => 
                {
                    let lhs = self.eval(&args[0]);
                    let rhs = self.eval(&args[1]);
                    match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) += *(rhs as *mut i64);},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) += *(rhs as *mut f64);},
                        _ => unreachable!()
                    }

                    unsafe {drop(Box::from_raw(rhs));}
                    lhs
                },
                FnName::Sub => 
                {
                    let lhs = self.eval(&args[0]);
                    let rhs = self.eval(&args[1]);
                    match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) -= *(rhs as *mut i64);},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) -= *(rhs as *mut f64);},
                        _ => unreachable!()
                    }

                    unsafe {drop(Box::from_raw(rhs));}
                    lhs
                },
                FnName::Mul => 
                {
                    let lhs = self.eval(&args[0]);
                    let rhs = self.eval(&args[1]);
                    match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) *= *(rhs as *mut i64);},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) *= *(rhs as *mut f64);},
                        _ => unreachable!()
                    }

                    unsafe {drop(Box::from_raw(rhs));}
                    lhs
                },
                FnName::Div => 
                {
                    let lhs = self.eval(&args[0]);
                    let rhs = self.eval(&args[1]);
                    match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) /= *(rhs as *mut i64);},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) /= *(rhs as *mut f64);},
                        _ => unreachable!()
                    }

                    unsafe {drop(Box::from_raw(rhs));}
                    lhs
                },
                FnName::EqualTo => 
                {
                    let lhs = self.eval(&args[0]);
                    let rhs = self.eval(&args[1]);
                    let res = match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) == *(rhs as *mut i64)},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) == *(rhs as *mut f64)},
                        VarType::Atom(AtomType::Bool) => unsafe {*(lhs as *mut u8) == *(rhs as *mut u8)},
                        _ => unreachable!()
                    };

                    unsafe {drop(Box::from_raw(lhs)); drop(Box::from_raw(rhs));}
                    Box::into_raw(Box::new([res as u8]))
                },
                FnName::NotEqualTo => 
                {
                    let lhs = self.eval(&args[0]);
                    let rhs = self.eval(&args[1]);
                    let res = match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) != *(rhs as *mut i64)},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) != *(rhs as *mut f64)},
                        VarType::Atom(AtomType::Bool) => unsafe {*(lhs as *mut u8) != *(rhs as *mut u8)},
                        _ => unreachable!()
                    };

                    unsafe {drop(Box::from_raw(lhs)); drop(Box::from_raw(rhs));}
                    Box::into_raw(Box::new([res as u8]))
                },
                FnName::Greater => 
                {
                    let lhs = self.eval(&args[0]);
                    let rhs = self.eval(&args[1]);
                    let res = match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) > *(rhs as *mut i64)},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) > *(rhs as *mut f64)},
                        _ => unreachable!()
                    };

                    unsafe {drop(Box::from_raw(lhs)); drop(Box::from_raw(rhs));}
                    Box::into_raw(Box::new([res as u8]))
                },
                FnName::Lesser => 
                {
                    let lhs = self.eval(&args[0]);
                    let rhs = self.eval(&args[1]);
                    let res = match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) < *(rhs as *mut i64)},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) < *(rhs as *mut f64)},
                        _ => unreachable!()
                    };

                    unsafe {drop(Box::from_raw(lhs)); drop(Box::from_raw(rhs));}
                    Box::into_raw(Box::new([res as u8]))
                },
                FnName::GreaterEq => 
                {
                    let lhs = self.eval(&args[0]);
                    let rhs = self.eval(&args[1]);
                    let res = match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) >= *(rhs as *mut i64)},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) >= *(rhs as *mut f64)},
                        _ => unreachable!()
                    };

                    unsafe {drop(Box::from_raw(lhs)); drop(Box::from_raw(rhs));}
                    Box::into_raw(Box::new([res as u8]))
                },
                FnName::LesserEq => 
                {
                    let lhs = self.eval(&args[0]);
                    let rhs = self.eval(&args[1]);
                    let res = match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) <= *(rhs as *mut i64)},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) <= *(rhs as *mut f64)},
                        _ => unreachable!()
                    };

                    unsafe {drop(Box::from_raw(lhs)); drop(Box::from_raw(rhs));}
                    Box::into_raw(Box::new([res as u8]))
                },
                FnName::And => 
                {
                    let lhs = self.eval(&args[0]);
                    let rhs = self.eval(&args[1]);

                    unsafe {*(lhs as *mut u8) = (*(lhs as *mut u8) != 0 && *(rhs as *mut u8) != 0) as u8;}
                    unsafe {drop(Box::from_raw(rhs));}
                    lhs
                },
                FnName::Or => 
                {
                    let lhs = self.eval(&args[0]);
                    let rhs = self.eval(&args[1]);

                    unsafe {*(lhs as *mut u8) = (*(lhs as *mut u8) != 0 || *(rhs as *mut u8) != 0) as u8;}
                    unsafe {drop(Box::from_raw(rhs));}
                    lhs
                },
                FnName::Assign => 
                {
                    let lhs = self.eval(&args[0]); 
                    let rhs = self.eval(&args[1]);

                    unsafe {std::ptr::copy_nonoverlapping((*rhs).as_ptr(), (*lhs).as_mut_ptr(), (&*rhs).len());}
                    match expr.cat
                    {
                        ValCat::Lvalue => {unsafe {drop(Box::from_raw(rhs));} lhs},
                        ValCat::Rvalue => rhs
                    }
                },
                FnName::Ternary => 
                {
                    let cond = self.eval(&args[0]);
                    let is_true = unsafe {*(cond as *mut u8) != 0};

                    unsafe {drop(Box::from_raw(cond));}
                    if is_true {self.eval(&args[1])} else {self.eval(&args[2])}
                },
            }
        }
    }
}