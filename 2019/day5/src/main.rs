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

struct VM {
    mem: Vec<i32>,
    input: Vec<i32>,
    output: Vec<i32>,

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

impl VM {
    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        while self.tick()? {}
        Ok(())
    }

    fn tick(&mut self) -> Result<bool, Box<dyn Error>> {
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
                    Ok(true)
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
                    Ok(true)
                }
            }
            3 => {
                let (r, mode) = r.div(10);
                let value = self.input.pop().ok_or_else(|| Box::new(VMError::Empty))?;
                let index = self.get_mem();
                let _a = self.put(index, value)?;
                if r != 0 || mode != 0 {
                    Err(Box::new(VMError::Input))
                } else {
                    Ok(true)
                }
            }
            4 => {
                let (r, mode) = r.div(10);
                let value = self.get_mem();
                let a = self.get(mode, value)?;
                self.output.push(a);
                if r != 0 {
                    Err(Box::new(VMError::Output))
                } else {
                    Ok(true)
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
                    Ok(true)
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
                    Ok(true)
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
                    Ok(true)
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
                    Ok(true)
                }
            }
            99 => Ok(false),
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
    let mut vm = VM {
        mem: instructions.clone(),
        input: vec![1],
        output: vec![],
        p: 0,
    };
    bench("Answer One", || vm.run())?;
    let mut iter = vm.output.iter().rev();
    let one = iter.next().unwrap();
    if !iter.all(|&i| i == 0) {
        println!("Error");
    }

    let mut vm = VM {
        mem: instructions.clone(),
        input: vec![5],
        output: vec![],
        p: 0,
    };
    bench("Answer Two", || vm.run())?;
    let mut iter = vm.output.iter().rev();
    let two = iter.next().unwrap();
    if let Some(_) = iter.next() {
        println!("Error");
    }
    println!("Answer One: {:?}", one);
    println!("Answer Two: {:?}", two);

    Ok(())
}
