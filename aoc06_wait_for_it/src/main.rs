use std::ops::Range;
use std::str::FromStr;
use sdk::*;
use sdk::anyhow::anyhow;

fn main() -> Result<()> {
    init();
    let input = std::fs::read_to_string("aoc06_wait_for_it/input.txt")?;

    // Part 1
    let races = Race::parse_races(&input)?;
    debug!("Races: {races:?}");

    let winning_hold_times: Vec<_> = races.iter().map(|r| r.winning_hold_times()).collect();
    debug!("Winning hold times: {winning_hold_times:?}");

    let winning_product = winning_hold_times.into_iter().fold(1, |product, next| {
        product * next.map(|r| r.end - r.start).unwrap_or(0)
    });
    info!("Winning combination product: {winning_product}");

    // Part 2
    let race = Race::parse_race(&input)?;
    debug!("Race: {race:?}");
    let winning_hold_times = race.winning_hold_times();
    debug!("Winning hold times: {winning_hold_times:?}");
    let winning_hold_times_count = winning_hold_times.map(|r| r.end - r.start).unwrap_or(0);
    info!("Winning hold times count: {winning_hold_times_count:?}");

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct Race {
    time: usize,
    min_distance: usize,
}

impl Race {
    // Part 2
    fn parse_race(input: &str) -> Result<Race> {
        fn parse_line(header: &str, line: &str) -> Result<usize> {
            if !line.starts_with(header) {
                return Err(anyhow!("Could not parse `{line}`; expected it to start with `{header}`"));
            }
            let number: usize = line
                .trim_start_matches(header)
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("")
                .parse()?;
            Ok(number)
        }

        let mut input = input.lines();
        let (times, distances) = input.next().zip(input.next())
            .ok_or(anyhow!("Could not parse - expected two lines"))?;

        let time = parse_line("Time:", times)?;
        let min_distance = parse_line("Distance:", distances)?;

        Ok(Race { time, min_distance })
    }

    // Part 1
    fn parse_races(input: &str) -> Result<Vec<Race>> {
        fn parse_line(header: &str, line: &str) -> Result<Vec<usize>> {
            if !line.starts_with(header) {
                return Err(anyhow!("Could not parse `{line}`; expected it to start with `{header}`"));
            }
            let numbers = line
                .trim_start_matches(header)
                .split_whitespace()
                .map(|n| usize::from_str(n))
                .collect::<std::result::Result<_, _>>()?;
            Ok(numbers)
        }

        let mut input = input.lines();
        let (times, distances) = input.next().zip(input.next())
            .ok_or(anyhow!("Could not parse - expected two lines"))?;

        let times = parse_line("Time:", times)?;
        let distances = parse_line("Distance:", distances)?;

        let races = times
            .into_iter()
            .zip(distances)
            .map(|(time, min_distance)| Race { time, min_distance })
            .collect();

        Ok(races)
    }

    fn winning_hold_times(&self) -> Option<Range<usize>> {
        // min_distance < (time - hold_time) * hold_time = time * hold_time - hold_time.pow(2)
        // hold_time.pow(2) - hold_time * time + min_distance < 0
        solve_quadratic_range(1.0, -1.0 * self.time as f64, self.min_distance as f64)
    }
}

// (-b±√(b²-4ac))/(2a)
fn solve_quadratic(a: f64, b: f64, c: f64) -> (f64, f64) {
    let first = ((-1.0 * b) + (b.powi(2) - (4.0 * a * c)).sqrt()) / (2.0 * a);
    let second = ((-1.0 * b) - (b.powi(2) - (4.0 * a * c)).sqrt()) / (2.0 * a);
    (first.min(second), first.max(second))
}

// Helper for finding the integer range which satisfies the quadratic
fn solve_quadratic_range(a: f64, b: f64, c: f64) -> Option<Range<usize>> {
    let (start, end) = solve_quadratic(a, b, c);
    let start = next_gt_int(start);
    let end = next_lt_int(end) + 1;
    (start >= 0).then(|| (start as usize)..(end as usize))
}

fn next_lt_int(f: f64) -> i64 {
    if f.floor() == f {
        (f as i64) - 1
    } else {
        f.floor() as i64
    }
}

fn next_gt_int(f: f64) -> i64 {
    if f.ceil() == f {
        (f as i64) + 1
    } else {
        f.ceil() as i64
    }
}

#[cfg(test)]
mod tests {
    use crate::{next_ge_int, next_lt_int, solve_quadratic_range};

    #[test]
    fn test_quadratic() {
        let cases = [((1.0, -7.0, 9.0), 2..6)];
        for ((a, b, c), expected) in cases {
            let range = solve_quadratic_range(a, b, c).unwrap();
            assert_eq!(range, expected);
        }
    }

    #[test]
    fn next_ints() {
        let cases = [
            (1.1, 1, 2),
            (1.0, 0, 1),
        ];
        for (f, lt, ge) in cases {
            assert_eq!(next_lt_int(f), lt);
            assert_eq!(next_ge_int(f), ge);
        }
    }
}