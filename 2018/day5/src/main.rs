use std::collections::HashSet;
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

fn parse_polymers<T>(input: T) -> Result<Vec<String>, Box<Error>>
where
    T: BufRead,
{
    input
        .lines()
        .map(|line| line.map_err(|e| e.into()))
        .collect::<Result<Vec<String>, Box<Error>>>()
}

fn strip_polymer(polymer: impl Iterator<Item = char>) -> Result<String, Box<Error>> {
    let mut react = String::with_capacity(polymer.size_hint().1.unwrap_or(0));
    for unit in polymer {
        if !unit.is_ascii_alphabetic() {
            return Err("Unit must be a character in lowercase or uppercase".into());
        }
        if react
            .chars()
            .rev()
            .next()
            .map(
                |last| match (last.is_ascii_lowercase(), unit.is_ascii_lowercase()) {
                    (false, true) => last.to_ascii_lowercase() == unit,
                    (true, false) => last == unit.to_ascii_lowercase(),
                    _ => false,
                },
            )
            .unwrap_or(false)
        {
            react.pop();
        } else {
            react.push(unit);
        }
    }
    Ok(react)
}

fn main() -> Result<(), Box<Error>> {
    let polymers = bench("parse_polymers", || parse_polymers(std::io::stdin().lock()))?;

    for polymer_stripped in polymers
        .iter()
        .map(|polymer| bench("len_normal", || strip_polymer(polymer.chars())))
    {
        let polymer_stripped = polymer_stripped?;
        let len_normal = polymer_stripped.len();

        let len_filtered = bench("len_filtered", || {
            let units: HashSet<_> = polymer_stripped
                .chars()
                .map(|unit| unit.to_ascii_lowercase())
                .collect();
            units.into_iter().try_fold(len_normal, |acc, unit| {
                let polymer_filtered = polymer_stripped
                    .chars()
                    .filter(|c| unit.to_ascii_lowercase() != c.to_ascii_lowercase());
                strip_polymer(polymer_filtered)
                    .map(|polymer_stripped| std::cmp::min(acc, polymer_stripped.len()))
            })
        })?;

        println!("Normal: {}", len_normal);
        println!("Filter: {}", len_filtered);
    }

    Ok(())
}
