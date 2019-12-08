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

fn parse_inputs<T>(input: T) -> Result<Vec<usize>, Box<dyn Error>>
where
    T: BufRead,
{
    let inputs: Vec<_> = input.lines().collect::<Result<_, _>>()?;
    inputs
        .iter()
        .flat_map(|line| {
            line.split(',').map(|token| {
                let n = token.parse()?;
                Ok(n)
            })
        })
        .collect()
}

fn answer_one(inputs: &[usize]) -> usize {
    let mut ret = inputs.to_vec();

    ret[1] = 12;
    ret[2] = 2;
    for i in (0..ret.len()).step_by(4) {
        let n = match ret[i] {
            1 => ret[ret[i + 1]] + ret[ret[i + 2]],
            2 => ret[ret[i + 1]] * ret[ret[i + 2]],
            99 => break,
            _ => panic!("noooooooooooooo"),
        };
        let j = ret[i + 3];
        ret[j] = n;
    }
    ret[0]
}

fn answer_two(inputs: &[usize]) -> usize {
    for (i, j) in (0..100).flat_map(|i| (0..100).map(move |j| (i, j))) {
        let mut ret = inputs.to_vec();

        ret[1] = i;
        ret[2] = j;
        for i in (0..ret.len()).step_by(4) {
            let n = match ret[i] {
                1 => ret[ret[i + 1]] + ret[ret[i + 2]],
                2 => ret[ret[i + 1]] * ret[ret[i + 2]],
                99 => break,
                _ => panic!("noooooooooooooo"),
            };
            let j = ret[i + 3];
            ret[j] = n;
        }
        if ret[0] == 19690720 {
            return 100 * i + j;
        }
    }
    panic!("no solution");
}

fn main() -> Result<(), Box<dyn Error>> {
    let inputs = bench("parse_inputs", || parse_inputs(std::io::stdin().lock()))?;
    let one = bench("answer_one", || answer_one(&inputs));
    let two = bench("answer_two", || answer_two(&inputs));

    println!("Answer One: {:?}", one);
    println!("Answer Two: {:?}", two);

    Ok(())
}
