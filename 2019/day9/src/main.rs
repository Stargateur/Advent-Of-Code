use std::error::Error;
use std::fmt::Debug;
use std::io::BufRead;
use std::time::Instant;

fn bench<T, F>(name: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let res = (f)();
    let elapsed = start.elapsed();
    println!("{}: {:?}", name, elapsed);
    res
}

use nom::{
    character::complete::{char, digit0},
    combinator::all_consuming,
    combinator::map_res,
    combinator::opt,
    combinator::recognize,
    multi::separated_list,
    IResult,
};

use std::str::FromStr;

fn parse_i64(input: &str) -> IResult<&str, i64> {
    let (input, neg) = opt(char('-'))(input)?;
    let (input, n) = map_res(recognize(digit0), i64::from_str)(input)?;
    Ok((input, if let Some(_) = neg { -n } else { n }))
}

trait DivPlus: Sized {
    fn div(self, other: Self) -> (Self, Self);
}

impl DivPlus for i64 {
    fn div(self, other: Self) -> (Self, Self) {
        (self / other, self % other)
    }
}

use std::collections::VecDeque;

struct VM {
    mem: Vec<i64>,
    inputs: VecDeque<i64>,
    p: usize,
    r: i64,
}

#[derive(Debug)]
enum VMError {
    Output,
    Mode,
    Opcode(i64),
    Empty,
}

impl std::fmt::Display for VMError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "VMError")
    }
}

impl std::error::Error for VMError {}

use std::convert::TryFrom;

enum State {
    Continue,
    Output(i64),
    Halt,
}

impl State {
    fn output(self) -> Result<i64, VMError> {
        if let State::Output(o) = self {
            Ok(o)
        } else {
            Err(VMError::Output)
        }
    }
}

impl VM {
    fn run(&mut self) -> Result<State, Box<dyn Error>> {
        loop {
            match self.tick()? {
                State::Continue => continue,
                state => break Ok(state),
            }
        }
    }

    fn tick(&mut self) -> Result<State, Box<dyn Error>> {
        let (r, op) = self.get_mem().div(100);
        match op {
            1 => {
                let (r, mode) = r.div(10);
                let value = self.get_mem();
                let a = self.get(mode, value)?;
                let (r, mode) = r.div(10);
                let value = self.get_mem();
                let b = self.get(mode, value)?;
                let (r, mode) = r.div(10);
                let index = self.get_mem();
                let _c = self.put(mode, index, a + b)?;
                if r != 0 {
                    Err(Box::new(VMError::Opcode(r)))
                } else {
                    Ok(State::Continue)
                }
            }
            2 => {
                let (r, mode) = r.div(10);
                let value = self.get_mem();
                let a = self.get(mode, value)?;
                let (r, mode) = r.div(10);
                let value = self.get_mem();
                let b = self.get(mode, value)?;
                let (r, mode) = r.div(10);
                let index = self.get_mem();
                let _c = self.put(mode, index, a * b)?;
                if r != 0 {
                    Err(Box::new(VMError::Opcode(r)))
                } else {
                    Ok(State::Continue)
                }
            }
            3 => {
                let (r, mode) = r.div(10);
                let value = self
                    .inputs
                    .pop_front()
                    .ok_or_else(|| Box::new(VMError::Empty))?;
                let index = self.get_mem();
                let _a = self.put(mode, index, value)?;
                if r != 0 {
                    Err(Box::new(VMError::Opcode(r)))
                } else {
                    Ok(State::Continue)
                }
            }
            4 => {
                let (r, mode) = r.div(10);
                let value = self.get_mem();
                let a = self.get(mode, value)?;
                if r != 0 {
                    Err(Box::new(VMError::Opcode(r)))
                } else {
                    Ok(State::Output(a))
                }
            }
            5 => {
                let (r, mode) = r.div(10);
                let value = self.get_mem();
                let a = self.get(mode, value)?;
                let (r, mode) = r.div(10);
                let value = self.get_mem();
                let b = self.get(mode, value)?;
                if a != 0 {
                    self.p = usize::try_from(b)?;
                }
                if r != 0 {
                    Err(Box::new(VMError::Opcode(r)))
                } else {
                    Ok(State::Continue)
                }
            }
            6 => {
                let (r, mode) = r.div(10);
                let value = self.get_mem();
                let a = self.get(mode, value)?;
                let (r, mode) = r.div(10);
                let value = self.get_mem();
                let b = self.get(mode, value)?;
                if a == 0 {
                    self.p = usize::try_from(b)?;
                }
                if r != 0 {
                    Err(Box::new(VMError::Opcode(r)))
                } else {
                    Ok(State::Continue)
                }
            }
            7 => {
                let (r, mode) = r.div(10);
                let value = self.get_mem();
                let a = self.get(mode, value)?;
                let (r, mode) = r.div(10);
                let value = self.get_mem();
                let b = self.get(mode, value)?;
                let (r, mode) = r.div(10);
                let index = self.get_mem();
                let _c = self.put(mode, index, if a < b { 1 } else { 0 })?;
                if r != 0 {
                    Err(Box::new(VMError::Opcode(r)))
                } else {
                    Ok(State::Continue)
                }
            }
            8 => {
                let (r, mode) = r.div(10);
                let value = self.get_mem();
                let a = self.get(mode, value)?;
                let (r, mode) = r.div(10);
                let value = self.get_mem();
                let b = self.get(mode, value)?;
                let (r, mode) = r.div(10);
                let index = self.get_mem();
                let _c = self.put(mode, index, if a == b { 1 } else { 0 })?;
                if r != 0 {
                    Err(Box::new(VMError::Opcode(r)))
                } else {
                    Ok(State::Continue)
                }
            }
            9 => {
                let (r, mode) = r.div(10);
                let value = self.get_mem();
                self.r = self.r + self.get(mode, value)?;
                if r != 0 {
                    Err(Box::new(VMError::Opcode(r)))
                } else {
                    Ok(State::Continue)
                }
            }
            99 => Ok(State::Halt),
            opcode => Err(Box::new(VMError::Opcode(opcode))),
        }
    }

    fn get_mem(&mut self) -> i64 {
        let p = self.p;
        self.p += 1;
        self.mem[p]
    }

    fn get(&self, mode: i64, value: i64) -> Result<i64, Box<dyn Error>> {
        match mode {
            0 => Ok(self.mem.get(usize::try_from(value)?).copied().unwrap_or(0)),
            1 => Ok(value),
            2 => Ok(self
                .mem
                .get(usize::try_from(self.r + value)?)
                .copied()
                .unwrap_or(0)),
            _ => Err(Box::new(VMError::Mode)),
        }
    }

    fn put(&mut self, mode: i64, index: i64, value: i64) -> Result<i64, Box<dyn Error>> {
        let index = match mode {
            0 => index,
            2 => self.r + index,
            _ => return Err(Box::new(VMError::Mode)),
        };
        let index = usize::try_from(index)?;
        let len = self.mem.len();
        if len < index + 1 {
            self.mem.resize(index + 1, 0);
        }
        Ok(std::mem::replace(&mut self.mem[index], value))
    }
}

fn instructions(input: &str) -> IResult<&str, Vec<i64>> {
    all_consuming(separated_list(char(','), parse_i64))(input)
}

fn main() -> Result<(), Box<dyn Error>> {
    let instructions = bench("parse_inputs", || {
        std::io::stdin()
            .lock()
            .lines()
            .flatten()
            .map(|line| instructions(&line).unwrap().1)
            .next()
            .unwrap()
    });
    let one = bench("calc_one", || -> Result<i64, Box<dyn Error>> {
        let mut vm = VM {
            inputs: (1..2).collect(),
            p: 0,
            mem: instructions.clone(),
            r: 0,
        };
        let mut output = 0;
        loop {
            match vm.run()? {
                State::Halt => break Ok(output),
                State::Output(o) => {
                    if output != 0 {
                        break Err(Box::new(VMError::Output));
                    } else {
                        output = o;
                    }
                }
                _ => unreachable!(),
            }
        }
    })?;
    let two = bench("calc_two", || -> Result<i64, Box<dyn Error>> {
        Ok(VM {
            inputs: (2..3).collect(),
            p: 0,
            mem: instructions.clone(),
            r: 0,
        }
        .run()?
        .output()?)
    })?;
    println!("Answer One: {:?}", one);
    println!("Answer Two: {:?}", two);

    Ok(())
}
