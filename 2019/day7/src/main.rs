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

fn parse_i32(input: &str) -> IResult<&str, i32> {
    let (input, neg) = opt(char('-'))(input)?;
    let (input, n) = map_res(recognize(digit0), i32::from_str)(input)?;
    Ok((input, if let Some(_) = neg { -n } else { n }))
}

trait DivPlus: Sized {
    fn div(self, other: Self) -> (Self, Self);
}

impl DivPlus for i32 {
    fn div(self, other: Self) -> (Self, Self) {
        (self / other, self % other)
    }
}

use std::collections::VecDeque;

struct VM {
    mem: Vec<i32>,
    inputs: VecDeque<i32>,
    p: usize,
}

#[derive(Debug)]
enum VMError {
    Add,
    Mul,
    Input,
    Output,
    Mode,
    Opcode(i32),
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
    Output(i32),
    Halt,
}

impl State {
    fn output(self) -> Result<i32, VMError> {
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
                let _c = self.put(index, a + b)?;
                if r != 0 || mode != 0 {
                    Err(Box::new(VMError::Add))
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
                let _c = self.put(index, a * b)?;
                if r != 0 || mode != 0 {
                    Err(Box::new(VMError::Mul))
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
                let _a = self.put(index, value)?;
                if r != 0 || mode != 0 {
                    Err(Box::new(VMError::Input))
                } else {
                    Ok(State::Continue)
                }
            }
            4 => {
                let (r, mode) = r.div(10);
                let value = self.get_mem();
                let a = self.get(mode, value)?;
                if r != 0 {
                    Err(Box::new(VMError::Output))
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
                    Err(Box::new(VMError::Output))
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
                    Err(Box::new(VMError::Output))
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
                let _c = self.put(index, if a < b { 1 } else { 0 })?;
                if r != 0 || mode != 0 {
                    Err(Box::new(VMError::Output))
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
                let _c = self.put(index, if a == b { 1 } else { 0 })?;
                if r != 0 || mode != 0 {
                    Err(Box::new(VMError::Output))
                } else {
                    Ok(State::Continue)
                }
            }
            99 => Ok(State::Halt),
            opcode => Err(Box::new(VMError::Opcode(opcode))),
        }
    }

    fn get_mem(&mut self) -> i32 {
        let p = self.p;
        self.p += 1;
        self.mem[p]
    }

    fn get(&self, mode: i32, value: i32) -> Result<i32, Box<dyn Error>> {
        match mode {
            0 => Ok(self.mem[usize::try_from(value)?]),
            1 => Ok(value),
            _ => Err(Box::new(VMError::Mode)),
        }
    }

    fn put(&mut self, index: i32, value: i32) -> Result<i32, Box<dyn Error>> {
        Ok(std::mem::replace(
            &mut self.mem[usize::try_from(index)?],
            value,
        ))
    }
}

fn instructions(input: &str) -> IResult<&str, Vec<i32>> {
    all_consuming(separated_list(char(','), parse_i32))(input)
}

use itertools::Itertools;
use std::cmp::max;

fn amplifier(instructions: &[i32], inputs: &[i32]) -> Result<i32, Box<dyn Error>> {
    inputs
        .iter()
        .copied()
        .try_fold(0, |a, b| -> Result<i32, Box<dyn Error>> {
            let mut inputs = VecDeque::new();
            inputs.push_back(b);
            inputs.push_back(a);
            let output = VM {
                mem: instructions.to_vec(),
                inputs,
                p: 0,
            }
            .run()?
            .output()?;
            Ok(output)
        })
}

use std::iter::successors;

fn amplifier_loop(instructions: &[i32], inputs: &[i32]) -> Result<i32, Box<dyn Error>> {
    let mut vms: Vec<_> = inputs
        .iter()
        .copied()
        .map(|i| {
            let mut inputs = VecDeque::new();
            inputs.push_back(i);
            VM {
                mem: instructions.to_vec(),
                inputs,
                p: 0,
            }
        })
        .collect();

    let mut index = (0..vms.len()).cycle();

    successors(Some(Ok(0)), |a| {
        let vm = &mut vms[index.next().unwrap()];
        vm.inputs.push_back(*a.as_ref().unwrap());
        let state = match vm.run() {
            Ok(state) => state,
            Err(e) => return Some(Err(e)),
        };
        match state {
            State::Halt => None,
            State::Output(a) => Some(Ok(a)),
            _ => unreachable!(),
        }
    })
    .last()
    .unwrap()
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
    let one = bench("calc_one", || {
        (0..5)
            .permutations(5)
            .map(|input| amplifier(&instructions, &input))
            .try_fold(0, |acc, x| -> Result<_, Box<dyn Error>> {
                let x = x?;
                Ok(max(x, acc))
            })
    })?;
    let two = bench("calc_two", || {
        (5..10)
            .permutations(5)
            .map(|input| amplifier_loop(&instructions, &input))
            .try_fold(0, |acc, x| -> Result<_, Box<dyn Error>> {
                let x = x?;
                Ok(max(x, acc))
            })
    })?;
    println!("Answer One: {:?}", one);
    println!("Answer Two: {:?}", two);

    Ok(())
}
