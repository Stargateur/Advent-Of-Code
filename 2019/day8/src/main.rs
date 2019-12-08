use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::io::BufRead;
use std::time::Instant;

use nom::{character::complete::*, combinator::*, error::ErrorKind, multi::*, IResult};

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

#[derive(Debug, Copy, Clone)]
enum Pixel {
    Black,
    White,
    Transparent,
}

fn pixel(input: &str) -> IResult<&str, Pixel> {
    let (input, c) = anychar(input)?;
    match c {
        '0' => Ok((input, Pixel::Black)),
        '1' => Ok((input, Pixel::White)),
        '2' => Ok((input, Pixel::Transparent)),
        _ => Err(nom::Err::Error((input, ErrorKind::Tag))),
    }
}

fn layers(input: &str, wide: usize, tall: usize) -> IResult<&str, Vec<Image>> {
    all_consuming(many0(map(count(pixel, wide * tall), |image| Image {
        image,
        wide,
        tall,
    })))(input)
}

#[derive(Debug)]
struct Image {
    image: Vec<Pixel>,
    wide: usize,
    tall: usize,
}

impl fmt::Display for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let p = match self {
            Pixel::Black => " ",
            Pixel::White => "8",
            Pixel::Transparent => " ",
        };
        write!(f, "{}", p)
    }
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.image.chunks_exact(self.wide) {
            for p in row {
                write!(f, "{}", p)?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let layers = bench("parse_inputs", || {
        std::io::stdin()
            .lock()
            .lines()
            .flatten()
            .map(|line| layers(&line, 25, 6).unwrap().1)
            .next()
            .unwrap()
    });
    let one = bench("calc_one", || {
        layers
            .iter()
            .map(|layer| {
                layer.image.iter().fold((0, 0, 0), |(b, w, t), p| match p {
                    Pixel::Black => (b + 1, w, t),
                    Pixel::White => (b, w + 1, t),
                    Pixel::Transparent => (b, w, t + 1),
                })
            })
            .min_by_key(|(b, _, _)| *b)
            .map(|(_, w, t)| w * t)
            .unwrap()
    });
    let two = bench("calc_two", || {
        let image = layers
            .iter()
            .skip(1)
            .fold(layers[0].image.clone(), |mut acc, layer| {
                for (acc, p) in acc.iter_mut().zip(layer.image.iter()) {
                    *acc = *match acc {
                        Pixel::Black | Pixel::White => acc,
                        Pixel::Transparent => p,
                    };
                }
                acc
            });
        Image {
            image,
            wide: 25,
            tall: 6,
        }
    });
    println!("Answer One: {:?}", one);
    println!("Answer Two:\n{}", two);

    Ok(())
}
