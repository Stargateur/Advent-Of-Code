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

fn parse_inputs<T>(input: T) -> Result<Vec<u32>, Box<dyn Error>>
where
    T: BufRead,
{
    input
        .lines()
        .map(|line| {
            let line = line?;
            let n = line.parse()?;
            Ok(n)
        })
        .collect()
}

fn answer_one(inputs: &[u32]) -> u32 {
    inputs
        .iter()
        .copied()
        .filter_map(|i| {
            let r = i / 3;
            if r > 2 {
                Some(r - 2)
            } else {
                None
            }
        })
        .sum()
}

fn answer_two(inputs: &[u32]) -> u32 {
    inputs
        .iter()
        .copied()
        .map(|i| {
            let calc = |i: &u32| {
                let r = i / 3;
                if r > 2 {
                    Some(r - 2)
                } else {
                    None
                }
            };
            std::iter::successors(calc(&i), calc).sum::<u32>()
        })
        .sum::<u32>()
}

fn main() -> Result<(), Box<dyn Error>> {
    let inputs = bench("parse_inputs", || parse_inputs(std::io::stdin().lock()))?;
    let one = bench("answer_one", || answer_one(&inputs));
    let two = bench("answer_two", || answer_two(&inputs));

    println!("Answer One: {}", one);
    println!("Answer Two: {}", two);

    Ok(())
}
