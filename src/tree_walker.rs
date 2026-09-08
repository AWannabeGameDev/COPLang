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
                ResStmt::Decl(expr) => {self.eval(expr);},
                ResStmt::Expr(expr) => 
                {
                    self.eval(expr);
                    if expr.typ != VarType::Unit {unsafe {drop(Box::from_raw(self.stack.pop().unwrap()))}}
                },
                ResStmt::Block(nest_block) => self.execute_block(nest_block),
            }
        }

        while self.stack.len() > block.base {unsafe {drop(Box::from_raw(self.stack.pop().unwrap()));}}
    }

    // evaluates the expression and puts the result on the stack
    fn eval(&mut self, expr: &ResExpr)
    {
        let ptr = match &expr.data
        {
            ResExprEnum::Literal(lit) => match lit
            {
                Literal::Int(x) => Box::into_raw(x.to_ne_bytes().to_vec().into_boxed_slice()),
                Literal::Float(x) => Box::into_raw(x.to_ne_bytes().to_vec().into_boxed_slice()),
                Literal::Bool(x) => Box::into_raw([*x as u8].to_vec().into_boxed_slice()),
            },
            ResExprEnum::StackBinding(idx) => match &expr.cat
            {
                ValCat::Lvalue => self.stack[*idx],
                ValCat::Rvalue => unsafe {Box::into_raw((&*self.stack[*idx]).to_vec().into_boxed_slice())}
            },
            ResExprEnum::Op(f, args) => match f
            {
                Operation::Negate =>
                {
                    self.eval(&args[0]);
                    let ret = self.stack.pop().unwrap();
                    match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(ret as *mut i64) = -*(ret as *mut i64);},
                        VarType::Atom(AtomType::Float) => unsafe {*(ret as *mut f64) = -*(ret as *mut f64);},
                        _ => unreachable!()
                    }

                    ret
                },
                Operation::Not => 
                {
                    self.eval(&args[0]);
                    let ret = self.stack.pop().unwrap();

                    unsafe {*(ret as *mut u8) = (*(ret as *mut u8) == 0) as u8;}
                    ret
                },
                Operation::Print => 
                {
                    self.eval(&args[0]);
                    let arg = self.stack.pop().unwrap();
                    match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {println!("{}", *(arg as *mut i64))},
                        VarType::Atom(AtomType::Float) => unsafe {println!("{}", *(arg as *mut f64))},
                        VarType::Atom(AtomType::Bool) => unsafe {println!("{}", *(arg as *mut u8) != 0)},
                        _ => unreachable!()
                    };

                    unsafe {drop(Box::from_raw(arg));}
                    Box::into_raw(Box::new([]))
                },
                Operation::Add => 
                {
                    self.eval(&args[0]);
                    self.eval(&args[1]);
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) += *(rhs as *mut i64);},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) += *(rhs as *mut f64);},
                        _ => unreachable!()
                    }

                    unsafe {drop(Box::from_raw(rhs));}
                    lhs
                },
                Operation::Sub => 
                {
                    self.eval(&args[0]);
                    self.eval(&args[1]);
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) -= *(rhs as *mut i64);},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) -= *(rhs as *mut f64);},
                        _ => unreachable!()
                    }

                    unsafe {drop(Box::from_raw(rhs));}
                    lhs
                },
                Operation::Mul => 
                {
                    self.eval(&args[0]);
                    self.eval(&args[1]);
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) *= *(rhs as *mut i64);},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) *= *(rhs as *mut f64);},
                        _ => unreachable!()
                    }

                    unsafe {drop(Box::from_raw(rhs));}
                    lhs
                },
                Operation::Div => 
                {
                    self.eval(&args[0]);
                    self.eval(&args[1]);
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) /= *(rhs as *mut i64);},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) /= *(rhs as *mut f64);},
                        _ => unreachable!()
                    }

                    unsafe {drop(Box::from_raw(rhs));}
                    lhs
                },
                Operation::EqualTo => 
                {
                    self.eval(&args[0]);
                    self.eval(&args[1]);
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
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
                Operation::NotEqualTo => 
                {
                    self.eval(&args[0]);
                    self.eval(&args[1]);
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
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
                Operation::Greater => 
                {
                    self.eval(&args[0]);
                    self.eval(&args[1]);
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let res = match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) > *(rhs as *mut i64)},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) > *(rhs as *mut f64)},
                        _ => unreachable!()
                    };

                    unsafe {drop(Box::from_raw(lhs)); drop(Box::from_raw(rhs));}
                    Box::into_raw(Box::new([res as u8]))
                },
                Operation::Lesser => 
                {
                    self.eval(&args[0]);
                    self.eval(&args[1]);
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let res = match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) < *(rhs as *mut i64)},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) < *(rhs as *mut f64)},
                        _ => unreachable!()
                    };

                    unsafe {drop(Box::from_raw(lhs)); drop(Box::from_raw(rhs));}
                    Box::into_raw(Box::new([res as u8]))
                },
                Operation::GreaterEq => 
                {
                    self.eval(&args[0]);
                    self.eval(&args[1]);
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let res = match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) >= *(rhs as *mut i64)},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) >= *(rhs as *mut f64)},
                        _ => unreachable!()
                    };

                    unsafe {drop(Box::from_raw(lhs)); drop(Box::from_raw(rhs));}
                    Box::into_raw(Box::new([res as u8]))
                },
                Operation::LesserEq => 
                {
                    self.eval(&args[0]);
                    self.eval(&args[1]);
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let res = match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(lhs as *mut i64) <= *(rhs as *mut i64)},
                        VarType::Atom(AtomType::Float) => unsafe {*(lhs as *mut f64) <= *(rhs as *mut f64)},
                        _ => unreachable!()
                    };

                    unsafe {drop(Box::from_raw(lhs)); drop(Box::from_raw(rhs));}
                    Box::into_raw(Box::new([res as u8]))
                },
                Operation::And => 
                {
                    self.eval(&args[0]);
                    self.eval(&args[1]);
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();

                    unsafe {*(lhs as *mut u8) = (*(lhs as *mut u8) != 0 && *(rhs as *mut u8) != 0) as u8;}
                    unsafe {drop(Box::from_raw(rhs));}
                    lhs
                },
                Operation::Or => 
                {
                    self.eval(&args[0]);
                    self.eval(&args[1]);
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();

                    unsafe {*(lhs as *mut u8) = (*(lhs as *mut u8) != 0 || *(rhs as *mut u8) != 0) as u8;}
                    unsafe {drop(Box::from_raw(rhs));}
                    lhs
                },
                Operation::Assign => 
                {
                    self.eval(&args[0]);
                    self.eval(&args[1]);
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();

                    unsafe {std::ptr::copy_nonoverlapping((*rhs).as_ptr(), (*lhs).as_mut_ptr(), (&*rhs).len());}
                    match expr.cat
                    {
                        ValCat::Lvalue => {unsafe {drop(Box::from_raw(rhs));} lhs},
                        ValCat::Rvalue => rhs
                    }
                },
                Operation::Ternary => 
                {
                    self.eval(&args[0]);
                    let cond = self.stack.pop().unwrap();
                    let is_true = unsafe {*(cond as *mut u8) != 0};
                    unsafe {drop(Box::from_raw(cond));}

                    if is_true {
                        self.eval(&args[1]);
                        self.stack.pop().unwrap()
                    } else {
                        self.eval(&args[2]);
                        self.stack.pop().unwrap()
                    }
                },
            }
        };

        self.stack.push(ptr);
    }
}