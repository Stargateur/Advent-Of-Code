#[macro_use]
extern crate nom;

use nom::types::CompleteStr;
use std::error::Error;
use std::io::BufRead;
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
struct Input {
    instruction: char,
    requiere: char,
}

fn bench<T, F>(name: &str, mut f: F) -> T
where
    F: FnMut() -> T,
{
    let start = Instant::now();
    let res = (f)();
    let elapsed = start.elapsed();
    println!("{}: {:?}", name, elapsed);
    res
}

// Step B must be finished before step X can begin.

named!(step<CompleteStr, Input>,
  do_parse!(
    ws!(tag!("Step")) >>
    requiere: ws!(none_of!("")) >>
    ws!(tag!("must")) >>
    ws!(tag!("be")) >>
    ws!(tag!("finished")) >>
    ws!(tag!("before")) >>
    ws!(tag!("step")) >>
    instruction: ws!(none_of!("")) >>
    ws!(tag!("can")) >>
    ws!(tag!("begin")) >>
    ws!(char!('.'))>>

    (Input { instruction, requiere })
  )
);

fn parse_inputs<T>(input: T) -> Result<Vec<Input>, Box<Error>>
where
    T: BufRead,
{
    let inputs = input
        .lines()
        .map(|line| {
            let line = line?;
            Ok(step(CompleteStr(&line)).map_err(|e| format!("{}", e))?.1)
        })
        .collect::<Result<Vec<Input>, Box<Error>>>()?;
    Ok(inputs)
}

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Eq)]
struct Instruction {
    instruction: char,
    requires: Vec<Rc<RefCell<Instruction>>>,
    done: bool,
    allows: Vec<Rc<RefCell<Instruction>>>,
}

impl Ord for Instruction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.instruction.cmp(&other.instruction)
    }
}

impl PartialOrd for Instruction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        self.instruction == other.instruction
    }
}

fn create_instructions(inputs: &Vec<Input>) -> HashMap<char, Rc<RefCell<Instruction>>> {
    let mut instructions = HashMap::new();
    for input in inputs {
        let require = instructions
            .entry(input.requiere)
            .or_insert_with(|| {
                Rc::new(RefCell::new(Instruction {
                    requires: Vec::new(),
                    done: false,
                    allows: Vec::new(),
                    instruction: input.requiere,
                }))
            })
            .clone();
        let allow = instructions.entry(input.instruction).or_insert_with(|| {
            Rc::new(RefCell::new(Instruction {
                requires: Vec::new(),
                done: false,
                allows: Vec::new(),
                instruction: input.instruction,
            }))
        });
        require.borrow_mut().allows.push(allow.clone());
        allow.borrow_mut().requires.push(require);
    }
    instructions
}

fn answer_one(instructions: &HashMap<char, Rc<RefCell<Instruction>>>) -> String {
    let mut work = create_work(instructions);
    let mut answer = String::new();
    while let Some(instruction) = work.pop() {
        answer.push(instruction.borrow().instruction);
        instruction.borrow_mut().done = true;
        for allow in instruction.borrow().allows.iter() {
            if allow
                .borrow()
                .requires
                .iter()
                .all(|require| require.borrow().done)
            {
                work.push(allow.clone());
            }
        }
    }
    answer
}

use binary_heap_plus::{BinaryHeap, MinComparator};

fn create_work(
    instructions: &HashMap<char, Rc<RefCell<Instruction>>>,
) -> BinaryHeap<Rc<RefCell<Instruction>>, MinComparator> {
    let mut work = BinaryHeap::new_min();

    for instruction in instructions
        .values()
        .filter(|instruction| instruction.borrow().requires.is_empty())
        .cloned()
    {
        work.push(instruction);
    }
    work
}

#[derive(Eq)]
struct Worker {
    instruction: Rc<RefCell<Instruction>>,
    time: i32,
}

impl Ord for Worker {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time)
    }
}

impl PartialOrd for Worker {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Worker {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

fn answer_two(instructions: &HashMap<char, Rc<RefCell<Instruction>>>) -> Result<i32, Box<Error>> {
    let mut work = create_work(instructions);
    let max_worker = 5;
    let times: HashMap<_, _> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().zip(61..).collect();

    let mut workers = BinaryHeap::new_min();
    while workers.len() < max_worker {
        if let Some(instruction) = work.pop() {
            let time = *times
                .get(&instruction.borrow().instruction)
                .ok_or("A instruction doesn't have time")?;
            workers.push(Worker {
                instruction,
                time: time,
            });
        } else {
            break;
        }
    }
    let mut result = 0;
    while let Some(worker) = workers.pop() {
        let instruction = worker.instruction;
        result = worker.time;
        instruction.borrow_mut().done = true;
        for allow in instruction.borrow().allows.iter() {
            if allow
                .borrow()
                .requires
                .iter()
                .all(|require| require.borrow().done)
            {
                work.push(allow.clone());
            }
        }
        while workers.len() < max_worker {
            if let Some(instruction) = work.pop() {
                let time = *times
                    .get(&instruction.borrow().instruction)
                    .ok_or("A instruction doesn't have time")?;
                workers.push(Worker {
                    instruction,
                    time: time + worker.time,
                });
            } else {
                break;
            }
        }
    }
    Ok(result)
}

fn main() -> Result<(), Box<Error>> {
    let inputs = bench("parse_inputs", || parse_inputs(std::io::stdin().lock()))?;
    let instructions = bench("create_instructions", || create_instructions(&inputs));
    let part1 = bench("answer_one", || answer_one(&instructions));
    bench("reset instructions", || {
        for instruction in instructions.values() {
            instruction.borrow_mut().done = false;
        }
    });
    let part2 = bench("answer_two", move || answer_two(&instructions))?;

    println!("Answer One: {}", part1);
    println!("Answer Two: {}", part2);

    Ok(())
}
