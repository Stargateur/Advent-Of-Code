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
    combinator::recognize,
    sequence::separated_pair,
    IResult,
};

use std::ops::Range;
use std::str::FromStr;

fn index(input: &str) -> IResult<&str, u32> {
    map_res(recognize(digit0), u32::from_str)(input)
}

fn range(input: &str) -> IResult<&str, Range<u32>> {
    let (input, (a, b)) = all_consuming(separated_pair(index, char('-'), index))(input)?;
    Ok((input, a..b))
}

//use std::convert::TryFrom;

fn decompose(n: u32, prev: u32, i: u32, adjacent: bool, radix: u32) -> bool {
    if n != 0 {
        let r = n % radix;
        if prev < r {
            false
        } else {
            let n = n / radix;
            let (i, adjacent) = if prev == r {
                (i + 1, adjacent)
            } else if i == 2 {
                (1, true)
            } else {
                (1, adjacent)
            };
            decompose(n, r, i, adjacent, radix)
        }
    } else {
        i == 2 || adjacent
    }
}

fn count(range: Range<u32>) -> usize {
    range
        .into_iter()
        .filter(|i| decompose(i / 10, i % 10, 1, false, 10))
        //        .inspect(|i| println!("{}", i))
        .count()
}

fn main() -> Result<(), Box<dyn Error>> {
    let range = bench("parse_inputs", || {
        std::io::stdin()
            .lock()
            .lines()
            .flatten()
            .map(|line| range(&line).unwrap().1)
            .next()
            .unwrap()
    });
    let n = bench("count", || count(range));
    println!("{:?}", n);

    //    let one = bench("Answer One", || intersections.iter().copied().map(Point::manhattan).min());
    //    let two = bench("Answer One", || intersections.iter().copied().map(|point| point.step).min());
    //    println!("Answer One: {:?}", one);
    //    println!("Answer Two: {:?}", two);

    Ok(())
}
