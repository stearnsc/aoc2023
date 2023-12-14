use std::cmp::min;
use std::fmt::{Debug, Formatter};
use std::fs;
use sdk::*;
use sdk::anyhow::anyhow;
use sdk::winnow::combinator::{separated};
use sdk::winnow::{Parser, PResult};
use sdk::winnow::token::take_while;

fn main() -> Result<()> {
    init();
    let input = fs::read_to_string("day13_point_of_incidence/input.txt")?;
    let mut sum = 0;
    let mut smudged_sum = 0;
    for mut pattern in parse(&input)? {
        debug!("{pattern:?}");
        if let Some(vert_pivot) = pattern.find_vert_pivot() {
            debug!("Vert pivot: {vert_pivot:?}");
            sum += vert_pivot;
        }
        if let Some(horiz_pivot) = pattern.find_horiz_pivot() {
            debug!("Horiz pivot: {horiz_pivot:?}");
            sum += 100 * horiz_pivot;
        }
        if let Some(vert_smudge) = pattern.find_vert_smudged_pivot() {
            debug!("Vert smudged_pivot: {vert_smudge:?}");
            smudged_sum += vert_smudge;
        }
        if let Some(horiz_smudge) = pattern.find_horiz_smudge() {
            debug!("Horiz smudged pivot: {horiz_smudge:?}");
            smudged_sum += 100 * horiz_smudge;
        }


    }
    info!("Pivot sum: {sum}");
    info!("Smudged pivot sum: {smudged_sum}");
    Ok(())
}

fn parse(input: &str) -> Result<Vec<Pattern>> {
    separated(1.., Pattern::parse, "\n\n")
        .parse(input)
        .map_err(|e| anyhow!("{e:?}"))
}

struct Pattern {
    inner: Vec<Vec<char>>,
    height: usize,
    width: usize,
}

impl Debug for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Pattern: ")?;
        for line in &self.inner {
            for c in line {
                write!(f, "{c}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Pattern {
    fn parse(input: &mut &str) -> PResult<Pattern> {
        fn parse_line(line: &mut &str) -> PResult<Vec<char>> {
            take_while(1.., ['#', '.'])
                .map(|l: &str| l.chars().collect())
                .parse_next(line)
        }
        separated(1.., parse_line,'\n')
            .map(|lines: Vec<Vec<char>>| {
                let height = lines.len();
                let width = lines[0].len();
                Pattern { inner: lines, height, width }
            })
            .parse_next(input)
    }

    fn column(&self, col: usize) -> Vec<char> {
        let mut column = Vec::with_capacity(self.height);
        for i in 0..self.height {
            column.push(self.inner[i][col]);
        }
        column
    }

    fn find_vert_pivot(&self) -> Option<usize> {
        let mut pivots = Vec::new();
        'pivots: for i in 1..self.width {
            for row in &self.inner {
                if !is_reflected(row, i) {
                    continue 'pivots;
                }
            }
            pivots.push(i);
        }
        pivots.into_iter().max_by_key(|i| min(*i, self.width - i))
    }

    // returns the pivot that would work with a single error correction
    fn find_vert_smudged_pivot(&self) -> Option<usize> {
        'pivots: for pivot in 1..self.width {
            let mut smudge = false;
            for row in &self.inner {
                match find_error(&row, pivot) {
                    // No reflections possible - this pivot won't work
                    None => continue 'pivots,
                    // Reflects on this pivot - smudge not on this row
                    Some(None) => {}
                    Some(Some(_)) => {
                        if smudge {
                            // we've already used up our smudge
                            continue 'pivots;
                        } else {
                            smudge = true;
                        }
                    }
                }
            }
            if smudge {
                return Some(pivot);
            }
        }
        None
    }

    fn find_horiz_smudge(&self) -> Option<usize> {
        'pivots: for pivot in 1..self.height {
            let mut smudge = false;
            for x in 0..self.width {
                let col = self.column(x);
                match find_error(&col, pivot) {
                    // No reflections possible - this pivot won't work
                    None => continue 'pivots,
                    // Reflects on this pivot - smudge not on this row
                    Some(None) => {}
                    Some(Some(_)) => {
                        if smudge {
                            continue 'pivots;
                        } else {
                            smudge = true;
                        }
                    }
                }
            }
            if smudge {
                return Some(pivot);
            }
        }
        None
    }

    fn find_horiz_pivot(&self) -> Option<usize> {
        let mut pivots = Vec::new();
        'pivots: for i in 1..self.height {
            for col in 0..self.width {
                let col = self.column(col);
                if !is_reflected(&col, i) {
                    continue 'pivots;
                }
            }
            pivots.push(i);
        }
        pivots.into_iter().max_by_key(|i| min(*i, self.height - i))
    }
}

fn find_error(section: &[char], pivot: usize) -> Option<Option<(usize, usize)>> {
    let (start, end) = section.split_at(pivot);
    let differences: Vec<_> = start
        .iter()
        .enumerate()
        .rev()
        .zip(end.iter().enumerate())
        .filter_map(|((i, a), (j, b))| {
            if a != b {
                Some([i, j + pivot])
            } else {
                None
            }
        })
        .flatten()
        .collect();
    match differences.as_slice() {
        &[single, reflection] => Some(Some((single, reflection))),
        &[] => Some(None),
        _ => None
    }
}

fn is_reflected(section: &[char], pivot: usize) -> bool {
    if pivot == 0 {
        return false
    }
    let (start, end) = section.split_at(pivot);
    let reflected = start.iter().rev().zip(end).all(|(a, b)| a == b);
    if reflected {
        trace!("{section:?} IS reflected over {pivot} ({start:?}, {end:?})");
    } else {
        trace!("{section:?} NOT reflected over {pivot} ({start:?}, {end:?})");
    }
    reflected
}
