#[macro_use]
extern crate nom;

use nom::types::CompleteStr;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Claim {
    pub pos_x: usize,
    pub pos_y: usize,
    pub seg_x: usize,
    pub seg_y: usize,
}

named!(usize_dec<CompleteStr, usize>,
  map_res!(take_while!(|c: char| c.is_digit(10)), |CompleteStr(s)| FromStr::from_str(&s))
);

named!(claim<CompleteStr, (usize, Claim)>,
  do_parse!(
    tag!("#") >>
    id: usize_dec >>
    ws!(tag!("@")) >>
    pos_x: usize_dec >>
    tag!(",") >>
    pos_y: usize_dec >>
    ws!(tag!(":")) >>
    seg_x: usize_dec >>
    tag!("x") >>
    seg_y: usize_dec >>
    (id, Claim { pos_x, pos_y, seg_x, seg_y })
  )
);

use std::collections::{HashMap, HashSet};
use std::io::BufRead;

use std::cmp;

use std::time::Instant;

fn parse_claims<T>(input: T) -> HashMap<usize, Claim>
where
    T: BufRead,
{
    let mut claims = HashMap::new();
    for (id, claim) in input
        .lines()
        .filter_map(|line| line.map_err(|e| eprintln!("{}", e)).ok())
        .filter_map(|line| match claim(CompleteStr(&line)) {
            Ok((_, res)) => Some(res),
            Err(e) => {
                eprintln!("{}", e);
                None
            }
        })
    {
        if let Some(previous_claim) = claims.insert(id, claim) {
            eprintln!(
                "Duplicate Claim Id: {}, old one was: {:?}",
                id, previous_claim
            );
        }
    }
    claims
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

type Fabric = Vec<Vec<Vec<usize>>>;

fn create_fabrics(claims: &HashMap<usize, Claim>) -> Fabric {
    let (x, y) = claims.values().fold((0, 0), |(x, y), claim| {
        (
            cmp::max(x, claim.pos_x + claim.seg_x),
            cmp::max(y, claim.pos_y + claim.seg_y),
        )
    });
    let mut fabric: Fabric = (0..y)
        .map(|_| (0..x).map(|_| Vec::new()).collect())
        .collect();
    for (id, claim) in claims {
        for row in &mut fabric[claim.pos_y..claim.pos_y + claim.seg_y] {
            for square in &mut row[claim.pos_x..claim.pos_x + claim.seg_x] {
                square.push(*id);
            }
        }
    }
    fabric
}

fn main() {
    let claims = bench("parse_claims", || parse_claims(std::io::stdin().lock()));
    let fabric = bench("create_fabrics", || create_fabrics(&claims));

    bench("create_not_overlaps_and_count_claims", || {
        let claims_id: HashSet<_> = claims.keys().collect();
        let mut count = 0;
        let claims_overlaps: HashSet<_> = fabric
            .iter()
            .flatten()
            .filter(|square| {
                if square.len() > 1 {
                    count += 1;
                    true
                } else {
                    false
                }
            })
            .flatten()
            .collect();
        println!("Number of Square that overlap {:?}", count);
        for not_overlap in claims_id.difference(&claims_overlaps) {
            println!("Id: {}, Claim: {:?}", not_overlap, claims[not_overlap]);
        }
    });
}
