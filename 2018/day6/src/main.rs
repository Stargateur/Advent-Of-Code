#[macro_use]
extern crate nom;

use nom::types::CompleteStr;
use std::error::Error;
use std::io::BufRead;
use std::str::FromStr;
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
struct Coord {
    x: usize,
    y: usize,
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

fn parse_coords<T>(input: T) -> Result<Vec<Coord>, Box<Error>>
where
    T: BufRead,
{
    let coords = input
        .lines()
        .map(|line| {
            let line = line?;
            Ok(coord(CompleteStr(&line)).map_err(|e| format!("{}", e))?.1)
        })
        .collect::<Result<Vec<Coord>, Box<Error>>>()?;
    Ok(coords)
}

named!(usize_dec<CompleteStr, usize>,
    map_res!(take_while!(|c: char| c.is_digit(10)), |CompleteStr(s)| FromStr::from_str(&s))
);

named!(coord<CompleteStr, Coord>,
  do_parse!(
    x: ws!(usize_dec) >>
    ws!(char!(',')) >>
    y: ws!(usize_dec) >>
    (Coord { x, y })
  )
);

impl std::ops::Sub for Coord {
    type Output = Coord;
    fn sub(self, other: Self) -> Self::Output {
        (&self).sub(&other)
    }
}

impl std::ops::Sub for &Coord {
    type Output = Coord;
    fn sub(self, other: Self) -> Self::Output {
        Self::Output {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[derive(Debug)]
enum State {
    Visited(usize),
    NotVisited,
}

use std::collections::{HashSet, VecDeque};

type Plan<T> = Vec<Vec<T>>;

fn create_plans(coords: &Vec<Coord>) -> Result<Vec<Plan<State>>, Box<Error>> {
    let mut iter = coords.iter();
    let first = iter.next().ok_or("At least one coordonate")?;
    let (min, max) = iter
        .fold((*first, *first), |(min, max), Coord { x, y }| {
            (
                Coord {
                    x: min.x.min(*x),
                    y: min.y.min(*y),
                },
                Coord {
                    x: max.x.max(*x),
                    y: max.y.max(*y),
                },
            )
        });
    let size = max - min;
    let offset = max - size;

    let plans : Vec<_> = coords
        .iter()
        .map(|coord| coord - &offset)
        .map(|coord| {
            let mut plan: Vec<Vec<_>> = (0..=size.y)
                .map(|_| (0..=size.x).map(|_| State::NotVisited).collect())
                .collect();
            let mut queue = VecDeque::new();

            queue.push_back((coord, 0));
            while let Some((coord, i)) = queue.pop_front() {
                plan[coord.y][coord.x] = match plan[coord.y][coord.x] {
                    State::NotVisited => State::Visited(i),
                    State::Visited(_) => {
                        continue;
                    }
                };

                let i = i + 1;
                if coord.y != 0 {
                    queue.push_back((
                        Coord {
                            x: coord.x,
                            y: coord.y - 1,
                        },
                        i,
                    ));
                }
                if coord.x != 0 {
                    queue.push_back((
                        Coord {
                            x: coord.x - 1,
                            y: coord.y,
                        },
                        i,
                    ));
                }
                if coord.y != size.y {
                    queue.push_back((
                        Coord {
                            x: coord.x,
                            y: coord.y + 1,
                        },
                        i,
                    ));
                }
                if coord.x != size.x {
                    queue.push_back((
                        Coord {
                            x: coord.x + 1,
                            y: coord.y,
                        },
                        i,
                    ));
                }
            }
            plan
        })
        .collect();
        Ok(plans)
}

#[derive(Debug)]
enum Closest {
    Equal(usize),
    Id(usize, usize),
}

fn create_closest(plans: &Vec<Plan<State>>) -> Result<Plan<Closest>, Box<Error>> {
    let mut iter = plans.iter().enumerate();
    let mut closest: Vec<Vec<_>> = iter
        .next()
        .map(|(id, plan)| {
            plan.iter()
                .map(|row| {
                    row.iter()
                        .map(|state| match state {
                            State::NotVisited => panic!("Bug"),
                            State::Visited(n) => Closest::Id(id, *n),
                        })
                        .collect()
                })
                .collect()
        })
        .ok_or("At least one coordonate")?;
    for (id, plan) in iter {
        for (closest, state) in closest
            .iter_mut()
            .zip(plan.iter())
            .flat_map(|(row_closest, row_plan)| row_closest.iter_mut().zip(row_plan.iter()))
        {
            let i = match state {
                State::NotVisited => panic!("Bug"),
                State::Visited(n) => *n,
            };
            *closest = match closest {
                Closest::Id(_, n) | Closest::Equal(n) => {
                    if i < *n {
                        Closest::Id(id, i)
                    } else if i == *n {
                        Closest::Equal(i)
                    } else {
                        continue;
                    }
                }
            };
        }
    }
    Ok(closest)
}

fn main() -> Result<(), Box<Error>> {
    let coords = bench("parse_coords", || parse_coords(std::io::stdin().lock()))?;
    let plans = bench("create_plans", || create_plans(&coords))?;
    let closest = bench("create_closest", || create_closest(&plans))?;

    let infinite_ids: HashSet<_> = closest
        .iter()
        .take(1)
        .flatten()
        .chain(closest.iter().rev().take(1).flatten())
        .chain(closest.iter().flat_map(|row| row.iter().take(1)))
        .chain(closest.iter().flat_map(|row| row.iter().rev().take(1)))
        .filter_map(|state| {
            if let Closest::Id(id, _) = state {
                Some(id)
            } else {
                None
            }
        })
        .collect();

    let mut counts = vec![0; coords.len()];
    for state in closest.iter().flat_map(|row| row.iter()) {
        if let Closest::Id(id, _) = state {
            counts[*id] += 1;
        }
    }

    let mut iter = coords.iter().enumerate().filter_map(|(id, _)| {
        if !infinite_ids.contains(&id) {
            Some(id)
        } else {
            None
        }
    });

    let init = iter.next().ok_or("No answer possible")?;
    let answer = iter.fold(
        init,
        |acc, id| if counts[acc] < counts[id] { id } else { acc },
    );
    println!("Answer: {}", counts[answer]);

    let mut iter = plans.iter();
    let mut totals: Vec<Vec<_>> = iter
        .next()
        .map(|plan| {
            plan.iter()
                .map(|row| {
                    row.iter()
                        .map(|state| match state {
                            State::NotVisited => panic!("Bug"),
                            State::Visited(n) => *n,
                        })
                        .collect()
                })
                .collect()
        })
        .ok_or("At least one coordonate")?;
    for plan in iter {
        for (total, state) in totals
            .iter_mut()
            .zip(plan.iter())
            .flat_map(|(row_totals, row_plan)| row_totals.iter_mut().zip(row_plan.iter()))
        {
            let i = match state {
                State::NotVisited => panic!("Bug"),
                State::Visited(n) => *n,
            };
            *total += i;
        }
    }

    let answer = totals
        .iter()
        .flat_map(|row| row.iter())
        .filter(|total| **total < 10_000)
        .count();
    println!("Answer: {}", answer);

    Ok(())
}
