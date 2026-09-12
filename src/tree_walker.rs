use crate::lexer::*;
use crate::ast::*;
use crate::resolver::*;

pub enum JumpOut
{
    None,
    Continue,
    Break,
    Return(*mut [u8])
}

pub struct TreeWalker<'p>
{
    res_funcs: &'p Vec<ResBlock>,
    stack: Vec<*mut [u8]>,
    frame_base: usize
}

pub enum RuntimeError
{
    DivZero,
    NoReturn
}

impl<'p> TreeWalker<'p>
{
    pub fn new(res_funcs: &'p Vec<ResBlock>) -> Self {Self {res_funcs, frame_base: 0, stack: Vec::new()}}

    pub fn execute(&mut self, block: &ResBlock) -> Result<(), RuntimeError>
    {
        self.execute_block(block, 0)?;
        Ok(())
    }

    fn execute_block(&mut self, block: &ResBlock, base: usize) -> Result<JumpOut, RuntimeError>
    {
        let mut ret = JumpOut::None;
        let old_base = self.frame_base;
        self.frame_base = base;

        for stmt in block.0.iter()
        {
            match stmt
            {
                ResStmt::Decl(expr) => {self.eval(expr)?;},
                ResStmt::Expr(expr) => 
                {
                    self.eval(expr)?;
                    unsafe {drop(Box::from_raw(self.stack.pop().unwrap()))}
                },
                ResStmt::Block(nest_block) => ret = self.execute_block(nest_block, self.stack.len())?,
                ResStmt::Cond(if_stmt) => ret = self.execute_cond(if_stmt)?,
                ResStmt::Break => ret = JumpOut::Break,
                ResStmt::Continue => ret = JumpOut::Continue,
                ResStmt::Iter(cond, block) => ret = self.execute_while(cond, block)?,
                ResStmt::FnDecl(_) => (),
                ResStmt::Return(expr) =>
                {
                    self.eval(expr)?;
                    ret = JumpOut::Return(self.stack.pop().unwrap());
                }
            }

            match ret
            {
                JumpOut::None => (),
                _ => break
            }
        }

        while self.stack.len() > self.frame_base {unsafe {drop(Box::from_raw(self.stack.pop().unwrap()))}}
        self.frame_base = old_base;
        Ok(ret)
    }

    fn execute_while(&mut self, cond: &ResExpr, block: &ResBlock) -> Result<JumpOut, RuntimeError>
    {
        let mut run = true;
        while run
        {
            self.eval(cond)?;
            let pred = unsafe {*(self.stack.pop().unwrap() as *mut u8)} != 0;

            if !pred {run = false;}
            else
            {
                let jump = self.execute_block(block, self.stack.len())?;
                match jump
                {
                    JumpOut::Break => run = false,
                    jump @ JumpOut::Return(_) => return Ok(jump),
                    _ => ()
                }
            }
        }

        Ok(JumpOut::None)
    }

    fn execute_cond(&mut self, if_stmt: &ResIfElse) -> Result<JumpOut, RuntimeError>
    {
        self.eval(&if_stmt.cond)?;
        let pred = unsafe {*(self.stack.pop().unwrap() as *mut u8)} != 0;

        if pred {self.execute_block(&if_stmt.block, self.stack.len())}
        else {self.execute_else(&if_stmt.els)}
    }

    fn execute_else(&mut self, else_stmt: &ResElse) -> Result<JumpOut, RuntimeError>
    {
        match else_stmt
        {
            ResElse::None => Ok(JumpOut::None),
            ResElse::Else(block) => self.execute_block(block, self.stack.len()),
            ResElse::ElseIf(if_stmt) => self.execute_cond(if_stmt)
        }
    }

    // evaluates the expression and puts the result on the stack
    fn eval(&mut self, expr: &ResExpr) -> Result<(), RuntimeError>
    {
        let ptr = match &expr.data
        {
            ResExprEnum::Unit => Box::into_raw(Box::new([])),
            ResExprEnum::Literal(lit) => match lit
            {
                Literal::Int(x) => Box::into_raw(x.to_ne_bytes().to_vec().into_boxed_slice()),
                Literal::Float(x) => Box::into_raw(x.to_ne_bytes().to_vec().into_boxed_slice()),
                Literal::Bool(x) => Box::into_raw([*x as u8].to_vec().into_boxed_slice()),
            },
            ResExprEnum::StackBinding(idx) => 
            {
                let var_idx = (self.frame_base as isize + *idx) as usize;
                match &expr.cat
                {
                    ValCat::Lvalue => self.stack[var_idx],
                    ValCat::Rvalue => unsafe {Box::into_raw((&*self.stack[var_idx]).to_vec().into_boxed_slice())}
                }
            },
            ResExprEnum::Op(f, args) => match f
            {
                ResOp::Negate =>
                {
                    self.eval(&args[0])?;
                    let ret = self.stack.pop().unwrap();
                    match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe {*(ret as *mut i64) = -*(ret as *mut i64);},
                        VarType::Atom(AtomType::Float) => unsafe {*(ret as *mut f64) = -*(ret as *mut f64);},
                        _ => unreachable!()
                    }

                    ret
                },
                ResOp::Not => 
                {
                    self.eval(&args[0])?;
                    let ret = self.stack.pop().unwrap();

                    unsafe {*(ret as *mut u8) = (*(ret as *mut u8) == 0) as u8;}
                    ret
                },
                ResOp::Print => 
                {
                    self.eval(&args[0])?;
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
                ResOp::Add => 
                {
                    self.eval(&args[0])?;
                    self.eval(&args[1])?;
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
                ResOp::Sub => 
                {
                    self.eval(&args[0])?;
                    self.eval(&args[1])?;
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
                ResOp::Mul => 
                {
                    self.eval(&args[0])?;
                    self.eval(&args[1])?;
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
                ResOp::Div => 
                {
                    self.eval(&args[0])?;
                    self.eval(&args[1])?;
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe 
                        {
                            let rhs_val = *(rhs as *mut i64);
                            if rhs_val == 0 {return Err(RuntimeError::DivZero)}
                            *(lhs as *mut i64) /= rhs_val;
                        },
                        VarType::Atom(AtomType::Float) => unsafe 
                        {
                            let rhs_val = *(rhs as *mut f64);
                            if rhs_val == 0.0 {return Err(RuntimeError::DivZero)}
                            *(lhs as *mut f64) /= *(rhs as *mut f64);
                        },
                        _ => unreachable!()
                    }

                    unsafe {drop(Box::from_raw(rhs));}
                    lhs
                },
                ResOp::Mod =>
                {
                    self.eval(&args[0])?;
                    self.eval(&args[1])?;
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    match args[0].typ
                    {
                        VarType::Atom(AtomType::Int) => unsafe 
                        {
                            let rhs_val = *(rhs as *mut i64);
                            if rhs_val == 0 {return Err(RuntimeError::DivZero)}
                            *(lhs as *mut i64) %= rhs_val;
                        },
                        VarType::Atom(AtomType::Float) => unsafe 
                        {
                            let rhs_val = *(rhs as *mut f64);
                            if rhs_val == 0.0 {return Err(RuntimeError::DivZero)}
                            *(lhs as *mut f64) %= *(rhs as *mut f64);
                        },
                        _ => unreachable!()
                    }

                    unsafe {drop(Box::from_raw(rhs));}
                    lhs
                }
                ResOp::EqualTo => 
                {
                    self.eval(&args[0])?;
                    self.eval(&args[1])?;
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
                ResOp::NotEqualTo => 
                {
                    self.eval(&args[0])?;
                    self.eval(&args[1])?;
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
                ResOp::Greater => 
                {
                    self.eval(&args[0])?;
                    self.eval(&args[1])?;
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
                ResOp::Lesser => 
                {
                    self.eval(&args[0])?;
                    self.eval(&args[1])?;
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
                ResOp::GreaterEq => 
                {
                    self.eval(&args[0])?;
                    self.eval(&args[1])?;
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
                ResOp::LesserEq => 
                {
                    self.eval(&args[0])?;
                    self.eval(&args[1])?;
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
                ResOp::And => 
                {
                    self.eval(&args[0])?;
                    self.eval(&args[1])?;
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();

                    unsafe {*(lhs as *mut u8) = (*(lhs as *mut u8) != 0 && *(rhs as *mut u8) != 0) as u8;}
                    unsafe {drop(Box::from_raw(rhs));}
                    lhs
                },
                ResOp::Or => 
                {
                    self.eval(&args[0])?;
                    self.eval(&args[1])?;
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();

                    unsafe {*(lhs as *mut u8) = (*(lhs as *mut u8) != 0 || *(rhs as *mut u8) != 0) as u8;}
                    unsafe {drop(Box::from_raw(rhs));}
                    lhs
                },
                ResOp::Assign => 
                {
                    self.eval(&args[0])?;
                    self.eval(&args[1])?;
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();

                    unsafe {std::ptr::copy_nonoverlapping((*rhs).as_ptr(), (*lhs).as_mut_ptr(), (&*rhs).len());}
                    match expr.cat
                    {
                        ValCat::Lvalue => {unsafe {drop(Box::from_raw(rhs));} lhs},
                        ValCat::Rvalue => rhs
                    }
                },
                ResOp::Ternary => 
                {
                    self.eval(&args[0])?;
                    let cond = self.stack.pop().unwrap();
                    let is_true = unsafe {*(cond as *mut u8) != 0};
                    unsafe {drop(Box::from_raw(cond));}

                    if is_true 
                    {
                        self.eval(&args[1])?;
                        self.stack.pop().unwrap()
                    } 
                    else 
                    {
                        self.eval(&args[2])?;
                        self.stack.pop().unwrap()
                    }
                },
                ResOp::Func(fn_idx) =>
                {
                    let base = self.stack.len();
                    for arg in args {self.eval(arg)?}
                    
                    match self.execute_block(&self.res_funcs[*fn_idx], base)?
                    {
                        JumpOut::Return(ret) => ret,
                        _ => return Err(RuntimeError::NoReturn)
                    }
                },
            }
        };

        self.stack.push(ptr);
        Ok(())
    }
}