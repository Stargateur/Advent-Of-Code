use std::error::Error;
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
    character::complete::{alphanumeric1, char},
    combinator::all_consuming,
    multi::fold_many0,
    sequence::{separated_pair, terminated},
    IResult,
};

use std::io::Read;

use petgraph::{
    algo::dijkstra,
    graphmap::UnGraphMap,
    visit::{VisitMap, Visitable},
};

fn links(input: &str) -> IResult<&str, UnGraphMap<&str, ()>> {
    all_consuming(fold_many0(
        terminated(
            separated_pair(alphanumeric1, char(')'), alphanumeric1),
            char('\n'),
        ),
        UnGraphMap::new(),
        |mut graph, (a, b)| {
            let a = graph.add_node(a);
            let b = graph.add_node(b);
            graph.add_edge(a, b, ());
            graph
        },
    ))(input)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut buffer = String::new();
    let (_, graph) = bench("parse_inputs", || -> Result<_, Box<dyn Error>> {
        std::io::stdin().lock().read_to_string(&mut buffer)?;
        Ok(links(&buffer).unwrap())
    })?;

    let com = "COM";
    let one = bench("calc_one", || {
        let mut to_visit = vec![(com, 0)];
        let mut acc = 0usize;
        let mut visit_map = graph.visit_map();
        visit_map.visit(com);
        while let Some((node, i)) = to_visit.pop() {
            acc += i;
            for child in graph.neighbors(node).filter(|child| visit_map.visit(child)) {
                to_visit.push((child, i + 1));
            }
        }
        acc
    });

    let sam = "SAN";
    let you = "YOU";
    let two = bench("calc_two", || dijkstra(&graph, you, Some(sam), |_| 1));

    println!("Answer One: {:?}", one);
    println!("Answer Two: {:?}", two[sam] - 2);

    Ok(())
}
