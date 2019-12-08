use std::collections::HashMap;
use std::io::BufRead;

fn main() -> Result<(), Box<std::error::Error>> {
    let stdin = std::io::stdin();
    let box_ids: Vec<_> = stdin
        .lock()
        .lines()
        .filter_map(|line| line.map_err(|e| eprintln!("{}", e)).ok())
        .collect();

    let checksum_maps: Vec<_> = box_ids
        .iter()
        .map(|line| {
            line.chars().fold(HashMap::new(), |mut map, c| {
                *map.entry(c).or_insert(0usize) += 1;
                map
            })
        })
        .collect();

    let (two, three) = checksum_maps.iter().fold((0usize, 0), |(two, three), map| {
        match map.values().fold((0usize, 0), |(two, three), n| match n {
            2 => (two + 1, three),
            3 => (two, three + 1),
            _ => (two, three),
        }) {
            (0, 0) => (two, three),
            (0, _) => (two, three + 1),
            (_, 0) => (two + 1, three),
            _ => (two + 1, three + 1),
        }
    });
    println!("Checksum: {}", two * three);

    let possible_box_ids: Vec<_> = box_ids
        .iter()
        .magic()
        .filter_map(|(iter, id)| {
            let possible_box_ids: Vec<_> = iter
                .filter_map(|other_id| {
                    if id
                        .chars()
                        .zip(other_id.chars())
                        .filter(|(a, b)| a != b)
                        .count()
                        == 1
                    {
                        Some(other_id.as_str())
                    } else {
                        None
                    }
                })
                .collect();

            if !possible_box_ids.is_empty() {
                Some((id, possible_box_ids))
            } else {
                None
            }
        })
        .collect();

    let correct_box_ids: Vec<String> = possible_box_ids
        .iter()
        .flat_map(|(id, possible_box_ids)| {
            possible_box_ids.iter().map(move |other_id| {
                id.chars()
                    .zip(other_id.chars())
                    .filter_map(|(a, b)| if a == b { Some(a) } else { None })
                    .collect()
            })
        })
        .collect();
    println!("Possible correct box:");
    correct_box_ids.iter().for_each(|id| println!("{}", id));

    Ok(())
}

pub struct Magic<I> {
    iter: I,
}

impl<I> Magic<I> {
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I> Iterator for Magic<I>
where
    I: Iterator + Clone,
{
    type Item = (I, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.iter.next() {
            Some((self.iter.clone(), item))
        } else {
            None
        }
    }
}

trait IterPlus: Iterator {
    fn magic(self) -> Magic<Self>
    where
        Self: Sized,
    {
        Magic::new(self)
    }
}

impl<T: ?Sized> IterPlus for T where T: Iterator {}
