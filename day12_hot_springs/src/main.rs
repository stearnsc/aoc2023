use std::fmt::{Debug, Display, Formatter};
use std::mem;
use sdk::*;
use winnow::PResult;
use sdk::anyhow::anyhow;
use sdk::winnow::ascii::digit1;
use sdk::winnow::combinator::{repeat, separated};
use sdk::winnow::error::{ErrorKind, InputError};
use sdk::winnow::Parser;
use sdk::winnow::token::{one_of};
use std::str::FromStr;
use itertools::Itertools;

fn main() -> Result<()> {
    init();
    info!("Hello, world!");
    let mut sum = 0;
    for line in lines("day12_hot_springs/example.txt")? {
        let mut row = Row::parse(&line)?;
        row.unfold();
        let permutations = row.permutations();
        debug!("{row:?}: {permutations} permutations");
        sum += permutations
    }
    info!("Total permutations: {sum}");
    Ok(())
}

#[derive(Clone, Copy)]
enum Spring {
    Working,
    Damaged,
    Unknown,
}

impl Spring {
    fn is_damaged(&self) -> bool {
        matches!(self, Spring::Damaged)
    }

    fn is_working(&self) -> bool {
        matches!(self, Spring::Working)
    }

    fn is_unknown(&self) -> bool {
        matches!(self, Spring::Unknown)
    }

    fn as_char(&self) -> char {
        match self {
            Spring::Working => '.',
            Spring::Damaged => '#',
            Spring::Unknown => '?',
        }
    }
}

impl TryFrom<char> for Spring {
    type Error = InputError<char>;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        match value {
            '.' => Ok(Spring::Working),
            '#' => Ok(Spring::Damaged),
            '?' => Ok(Spring::Unknown),
            _ => Err(InputError::new(value, ErrorKind::Verify)),
        }
    }
}

impl Display for Spring {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl Debug for Spring {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

#[derive(Clone)]
struct Row {
    springs: Vec<Spring>,
    counts: Vec<usize>,
}

impl Row {
    fn parse(input: &str) -> Result<Self> {
        fn parse_springs(input: &mut &str) -> PResult<Vec<Spring>> {
            repeat(
                1..,
                one_of(['.', '#', '?']).try_map(Spring::try_from),
            ).parse_next(input)
        }

        fn parse_counts(input: &mut &str) -> PResult<Vec<usize>> {
            separated(1.., digit1.try_map(usize::from_str), ",")
                .parse_next(input)
        }

        (parse_springs, ' ', parse_counts).parse(input)
            .map(|(springs, _, counts)| Row { springs, counts })
            .map_err(|e| anyhow!("{e}"))
    }

    fn unfold(&mut self) {
        let count_len = self.counts.len() * 5;
        self.counts = mem::take(&mut self.counts).into_iter().cycle().take(count_len).collect();
        self.springs.push(Spring::Unknown);
        let spring_len = self.springs.len() * 5;
        self.springs = mem::take(&mut self.springs).into_iter().cycle().take(spring_len).collect();
        self.springs.pop();
    }

    fn permutations(&self) -> usize {
        fn is_valid(row: &[Spring], counts: &[usize]) -> bool {
            let mut block_count = 0;
            let blocks_valid = row.split(|s| s.is_working())
                .filter(|s| !s.is_empty())
                .zip(counts)
                .all(|(block, count)| {
                    block_count += 1;
                    block.len() == *count
                });
            let result = blocks_valid && block_count == counts.len();
            trace!("Row {row:?} counts {counts:?}: {result}");
            result
        }

        let get_damaged_indices = |indices: &[usize]| {
            let mut springs = self.springs.clone();
            for index in indices {
                springs[*index] = Spring::Damaged;
            }
            for spring in &mut springs {
                if spring.is_unknown() {
                    *spring = Spring::Working;
                }
            }
            springs
        };

        let total_damaged: usize = self.counts.iter().sum();
        let known_damaged = self.springs.iter().filter(|s| s.is_damaged()).count();
        let unknown_damaged = total_damaged - known_damaged;
        self.springs.iter().enumerate()
            .filter(|(_, s)| s.is_unknown())
            .map(|(i, _)| i)
            .combinations(unknown_damaged)
            .filter(|damaged_indices| is_valid(&get_damaged_indices(damaged_indices), &self.counts))
            .count()
    }
}

impl Debug for Row {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Row {{ springs: ")?;
        for spring in &self.springs {
            write!(f, "{spring}")?;
        }
        write!(f, ", counts: {:?} }}", self.counts)
    }
}
