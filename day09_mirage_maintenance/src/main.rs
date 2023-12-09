use anyhow::anyhow;
use itertools::Itertools;
use winnow::ascii::{dec_int};
use winnow::combinator::{separated};
use winnow::{Parser, PResult};
use sdk::*;

fn main() -> Result<()> {
    init();
    info!("Hello, world!");
    let mut nexts_sum = 0;
    let mut prevs_sum = 0;
    for line in lines("day09_mirage_maintenance/input.txt")? {
        let line = line?;
        let history = parse_history.parse(&line)
            .map_err(|e| anyhow!(e.to_string()))?;
        let (prev, next) = extrapolate(&history);
        debug!("Prev for {history:?}: {prev}");
        debug!("Next for {history:?}: {next}");
        prevs_sum += prev;
        nexts_sum += next;
    }
    // 1702218515
    info!("Sum of prev values: {prevs_sum}");
    info!("Sum of next values: {nexts_sum}");

    Ok(())
}

fn parse_history<'a>(input: &mut &'a str) -> PResult<Vec<i64>> {
    separated(0.., dec_int::<_, i64, _>, ' ').parse_next(input)
}

fn extrapolate(history: &[i64]) -> (i64, i64) {
    if history.is_empty() {
        panic!("Empty history");
    }
    fn differences(history: &[i64]) -> Vec<i64> {
        history.iter().tuple_windows().map(|(a, b)| b - a).collect()
    }

    let mut current = history.to_vec();
    let mut prev_stack = Vec::new();
    let mut next_stack = Vec::new();
    trace!("{history:?}");
    loop {
        current = differences(&current);
        trace!("{current:?}");
        if current.iter().all(|d| *d == 0) {
            break;
        }
        let Some((first, last)) = current.first().zip(current.last()) else {
            warn!("Empty differences for {history:?}");
            break;
        };
        prev_stack.push(*first);
        next_stack.push(*last);
    }

    let next = history.last().unwrap() + next_stack.iter().sum::<i64>();
    let prev = history.first().unwrap() - prev_stack.iter().enumerate().fold(0, |sum, (i, next)| {
        if i % 2 == 0 {
            sum + *next
        } else {
            sum - *next
        }
    });

    (prev, next)
}