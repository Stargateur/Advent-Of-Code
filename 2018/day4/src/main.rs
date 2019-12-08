#[macro_use]
extern crate nom;
extern crate chrono;

use chrono::naive::NaiveDateTime;
use chrono::Timelike;
use nom::types::CompleteStr;
use std::collections::HashMap;
use std::error::Error;
use std::io::BufRead;
use std::str::FromStr;
use std::time::Instant;

#[derive(Debug)]
struct Event {
    date_time: NaiveDateTime,
    action: Action,
}

#[derive(Debug)]
enum Action {
    Sleep,
    Wake,
    Shift(usize),
}

named!(usize_dec<CompleteStr, usize>,
    map_res!(take_while!(|c: char| c.is_digit(10)), |CompleteStr(s)| FromStr::from_str(&s))
);

named!(parse_date_time<CompleteStr, NaiveDateTime>,
    map_res!(delimited!(char!('['), take_till!(|ch| ch == ']'), char!(']')),
    |CompleteStr(s)| NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M"))
);

named!(parse_shift<CompleteStr, Action>,
    do_parse!(
        ws!(tag!("Guard")) >>
        ws!(tag!("#")) >>
        id: usize_dec >>
        ws!(tag!("begins")) >>
        ws!(tag!("shift")) >>
        (Action::Shift(id))
    )
);

named!(parse_action<CompleteStr, Action>,
    alt!(
        do_parse!(ws!(tag!("falls")) >> ws!(tag!("asleep")) >> (Action::Sleep)) |
        do_parse!(ws!(tag!("wakes")) >> ws!(tag!("up")) >> (Action::Wake)) |
        parse_shift
    )
);

named!(event<CompleteStr, Event>,
  do_parse!(
    date_time: ws!(parse_date_time) >>
    action: ws!(parse_action) >>
    (Event { date_time, action })
  )
);

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

fn parse_events<T>(input: T) -> Result<Vec<Event>, Box<Error>>
where
    T: BufRead,
{
    let mut events = input
        .lines()
        .map(|line| {
            let line = line?;
            Ok(event(CompleteStr(&line)).map_err(|e| format!("{}", e))?.1)
        })
        .collect::<Result<Vec<Event>, Box<Error>>>()?;
    events.sort_unstable_by_key(|event| event.date_time);
    Ok(events)
}

type Stats = HashMap<usize, (u32, Vec<u32>)>;

fn create_stats(events: &Vec<Event>) -> Result<Stats, Box<Error>> {
    let mut id = None;
    let mut time = None;
    let mut stats = HashMap::new();
    for event in events.iter() {
        match event.action {
            Action::Shift(shift) => {
                id = Some(shift);
                if time.is_some() {
                    Err(format!("Shift before Wake: {:?}", event))?;
                }
            }
            Action::Sleep => {
                id.ok_or("Sleep before Shift")?;
                if time.is_some() {
                    Err(format!("Sleep twice: {:?}", event))?;
                }
                time = Some(event.date_time.time())
            }
            Action::Wake => {
                let time = time.take().ok_or("Wake before Sleep")?;
                let id = id.ok_or("Wake before Shift")?;
                let start = time.minute();
                let end = event.date_time.time().minute();
                let (total, counts) = stats.entry(id).or_insert((0, vec![0; 60]));
                *total += end - start;
                for step in &mut counts[start as usize..end as usize] {
                    *step += 1;
                }
            }
        }
    }
    Ok(stats)
}

fn create_results(stats: &Stats) -> Result<Vec<(usize, u32)>, Box<Error>> {
    let mut iter = stats.iter();
    let (id, (_, counts)) = iter.next().ok_or("Stats is empty")?;
    let mut results: Vec<_> = counts.iter().map(|count| (*id, *count)).collect();
    for (id, (_, counts)) in iter {
        for ((prev_id, prev_count), count) in results.iter_mut().zip(counts.iter()) {
            if *prev_count < *count {
                *prev_id = *id;
                *prev_count = *count;
            }
        }
    }
    Ok(results)
}

fn main() -> Result<(), Box<Error>> {
    let events = bench("parse_events", || parse_events(std::io::stdin().lock()))?;

    let stats = bench("create_stats", || create_stats(&events))?;

    let answer = bench("create_answer", || {
        stats
            .iter()
            .max_by_key(|(_, (total, _))| total)
            .map(|(id, (_, counts))| {
                counts
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, count)| *count)
                    .map(|(index, _)| index * id)
                    .ok_or("There is no count")
            })
            .ok_or("There is no guard")?
    })?;

    let results = bench("create_results", || create_results(&stats))?;

    println!("Strategie 1: {:?}", answer);
    println!(
        "Strategie 2: {}",
        results
            .iter()
            .enumerate()
            .max_by_key(|(_, (_, count))| count)
            .map(|(index, (id, _))| id * index)
            .ok_or("There is no result")?
    );

    Ok(())
}
