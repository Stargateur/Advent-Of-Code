use std::error::Error;
use std::io::BufRead;
use std::time::Instant;

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

use nom::{
    branch::alt,
    character::complete::{char, digit0},
    combinator::all_consuming,
    combinator::map_res,
    combinator::recognize,
    multi::separated_list,
    IResult,
};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Sens {
    Right,
    Left,
    Up,
    Down,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Dir {
    sens: Sens,
    value: u32,
}

use std::str::FromStr;

fn index(input: &str) -> IResult<&str, u32> {
    map_res(recognize(digit0), u32::from_str)(input)
}

fn right(input: &str) -> IResult<&str, Sens> {
    let (input, _) = char('R')(input)?;
    Ok((input, Sens::Right {}))
}

fn left(input: &str) -> IResult<&str, Sens> {
    let (input, _) = char('L')(input)?;
    Ok((input, Sens::Left {}))
}

fn up(input: &str) -> IResult<&str, Sens> {
    let (input, _) = char('U')(input)?;
    Ok((input, Sens::Up {}))
}

fn down(input: &str) -> IResult<&str, Sens> {
    let (input, _) = char('D')(input)?;
    Ok((input, Sens::Down {}))
}

fn direction(input: &str) -> IResult<&str, Dir> {
    let (input, sens) = alt((right, left, up, down))(input)?;
    let (input, value) = index(input)?;
    Ok((input, Dir { sens, value }))
}

fn directions(input: &str) -> IResult<&str, Vec<Dir>> {
    all_consuming(separated_list(char(','), direction))(input)
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Point {
    x: i32,
    y: i32,
    step: u32,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum AxeSegment {
    X { x: i32, ya: i32, yb: i32, xstep: u32 },
    Y { y: i32, xa: i32, xb: i32, ystep: u32 },
}

impl Point {
    fn new(x: i32, y: i32, step: u32) -> Self {
        Self { x, y, step }
    }
    fn right(self, n: u32) -> Self {
        Self {
            x: self.x + n as i32,
            y: self.y,
            step: self.step + n,
        }
    }
    fn left(self, n: u32) -> Self {
        Self {
            x: self.x - n as i32,
            y: self.y,
            step: self.step + n,
        }
    }
    fn up(self, n: u32) -> Self {
        Self {
            x: self.x,
            y: self.y + n as i32,
            step: self.step + n,
        }
    }
    fn down(self, n: u32) -> Self {
        Self {
            x: self.x,
            y: self.y - n as i32,
            step: self.step + n,
        }
    }

    fn point(self, direction: Dir) -> Self {
        (match direction.sens {
            Sens::Right => Self::right,
            Sens::Left => Self::left,
            Sens::Up => Self::up,
            Sens::Down => Self::down,
        })(self, direction.value)
    }

    fn manhattan(self) -> i32 {
        self.x.abs() + self.y.abs()
    }
}

fn calc_points(directions: &[Dir]) -> Vec<Point> {
    let mut directions = directions.iter();
    std::iter::successors(Some(Point::new(0, 0, 0)), |point| {
        if let Some(direction) = directions.next() {
            Some(point.point(*direction))
        } else {
            None
        }
    })
    .collect()
}

impl AxeSegment {
    /*
    fn new(a: Point, b: Point) -> Self {
        Self { a, b }
    }

    fn pente(self) -> f32 {
        (self.b.y - self.a.y) as f32 / (self.b.x - self.a.x) as f32
    }*/
}

use itertools::Itertools;

fn calc_segments(points: &[Point]) -> Vec<AxeSegment> {
    points
        .iter()
        .copied()
        .tuple_windows()
        .map(|(a, b)| {
            if a.x == b.x {
                AxeSegment::X {
                    x: a.x,
                    ya: a.y,
                    yb: b.y,
                    xstep: a.step,
                }
            } else if a.y == b.y {
                AxeSegment::Y {
                    y: a.y,
                    xa: a.x,
                    xb: b.x,
                    ystep: a.step,
                }
            } else {
                panic!("That not possible");
            }
        })
        .collect()
}
/*
fn calc_pentes(points: &[Segment]) -> Vec<f32> {
    points.iter().copied().map(Segment::pente).collect()
}*/

use std::fmt::Debug;

use std::cmp::{max, min};

fn calc_intersection<'a, A, B>(segments: A) -> Vec<Point>
where
    A: IntoIterator<Item = B> + 'a,
    <A as IntoIterator>::IntoIter: Clone,
    B: IntoIterator<Item = &'a AxeSegment> + Clone + Debug,
{
    segments
        .into_iter()
        .tuple_combinations()
        .flat_map(|(a, b)| {
            a.into_iter().flat_map(move |a| {
                b.clone().into_iter().filter_map(move |b| match (a, b) {
                    (AxeSegment::X { x, ya, yb, xstep }, AxeSegment::Y { y, xa, xb, ystep })
                    | (AxeSegment::Y { y, xa, xb, ystep }, AxeSegment::X { x, ya, yb, xstep }) => {
                        if min(xa, xb) < x && x < max(xa, xb) && min(ya, yb) < y && y < max(ya, yb)
                        {
                            let i = if xa < xb {
                                x - xa
                            } else {
                                xa - x
                            };
                            let j = if ya < yb {
                                y - ya
                            }
                            else {
                                ya - y
                            };
                            Some(Point::new(*x, *y, xstep + ystep + i.abs() as u32 + j.abs() as u32))
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
            })
        })
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let wires: Vec<_> = bench("parse_inputs", || {
        std::io::stdin()
            .lock()
            .lines()
            .flatten()
            .map(|line| directions(&line).unwrap().1)
            .collect()
    });
    let points: Vec<_> = bench("calc_points", || {
        wires.iter().map(|wire| calc_points(&wire)).collect()
    });
    let segments: Vec<_> = bench("calc_segments", || {
        points.iter().map(|point| calc_segments(&point)).collect()
    });
    let intersections: Vec<_> = bench("calc_intersection", || calc_intersection(&segments));

    let one = bench("Answer One", || intersections.iter().copied().map(Point::manhattan).min());
    let two = bench("Answer One", || intersections.iter().copied().map(|point| point.step).min());
    //println!("{:?}", intersections);
    println!("Answer One: {:?}", one);
    println!("Answer Two: {:?}", two);

    Ok(())
}
