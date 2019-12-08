use std::collections::HashSet;
use std::io::BufRead;

pub struct Mapscan<I, St, F> {
    state: St,
    iter: I,
    f: F,
}

impl<I, St, F> Mapscan<I, St, F> {
    pub fn new(iter: I, state: St, f: F) -> Self {
        Self { iter, state, f }
    }
}

impl<I, St, F, O> Iterator for Mapscan<I, St, F>
where
    I: Iterator,
    F: FnMut(&mut St, I::Item) -> O,
{
    type Item = O;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .and_then(|item| Some((self.f)(&mut self.state, item)))
    }
}

trait IterPlus: Iterator {
    fn map_scan<St, F, O>(self, state: St, f: F) -> Mapscan<Self, St, F>
    where
        Self: Sized,
        F: FnMut(&mut St, Self::Item) -> O,
    {
        Mapscan::new(self, state, f)
    }
}

impl<T: ?Sized> IterPlus for T where T: Iterator {}

fn main() -> Result<(), Box<std::error::Error>> {
    let stdin = std::io::stdin();
    let freqs_change: Vec<_> = stdin
        .lock()
        .lines()
        .flat_map(|line| line.map_err(|e| eprintln!("{}", e)))
        .map(|line| i32::from_str_radix(line.trim(), 10))
        .flat_map(|result| result.map_err(|e| eprintln!("{}", e)))
        .collect();

    println!(
        "Final frequence: {}",
        freqs_change
            .iter()
            .fold(0, |freq, freq_change| freq + freq_change)
    );

    let mut set = HashSet::new();
    let first_repeat = Some(0)
        .into_iter()
        .chain(
            freqs_change
                .iter()
                .cycle()
                .map_scan(0, |freq, freq_change| {
                    *freq += freq_change;
                    *freq
                }),
        )
        .find_map(|item| set.replace(item));

    println!("First repeated frequence: {}", first_repeat.unwrap_or(0));
    Ok(())
}
