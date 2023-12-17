use std::collections::{BTreeMap, HashMap, VecDeque};
use std::collections::hash_map::{DefaultHasher, Entry};
use std::fmt::{Debug, Display, Formatter};
use std::{fs, iter};
use sdk::*;
use sdk::anyhow::{anyhow, bail};
use crate::Direction::{Down, Left, Right, Up};

fn main() -> Result<()> {
    init();
    let input = fs::read_to_string("day17_clumsy_crucible/input.txt")?;
    let weights = Weights::parse(&input)?;
    debug!("{weights:?}");
    let start = (0, 0);
    let end = (weights.width - 1, weights.height - 1);
    let costs = weights.costs(start, 4, 10)?;
    debug!("{costs}");
    let (end_x, end_y) = end;
    if let Some(min_cost) = costs.get(end_x, end_y) {
        info!("Min cost: {min_cost}");
    } else {
        info!("No path to end found");
    }
    Ok(())
}

struct Weights {
    height: usize,
    width: usize,
    inner: Vec<Vec<u32>>,
}

impl Weights {
    fn parse(input: &str) -> Result<Self> {
        let lines: Vec<_> = input.lines().collect();
        let height = lines.len();
        let width = lines.first().ok_or(anyhow!("empty line"))?.len();
        let mut weights = Weights {
            height,
            width,
            inner: vec![vec![0; width]; height],
        };
        for (y, line) in lines.iter().enumerate() {
            for (x, weight) in line.chars().enumerate() {
                weights.inner[y][x] = weight.to_digit(10).ok_or(anyhow!("not a number"))?;
            }
        }
        Ok(weights)
    }

    fn get(&self, x: usize, y: usize) -> u32 {
        self.inner[y][x]
    }

    fn neighbors(&self, x: usize, y: usize) -> Vec<(Direction, (usize, usize))> {
        [
            (y > 0).then(|| (Up, (x, y - 1))),
            (x < self.width - 1).then(|| (Right, (x + 1, y))),
            (y < self.height - 1).then(|| (Down, (x, y + 1))),
            (x > 0).then(|| (Left, (x - 1, y))),
        ].into_iter().flatten().collect()
    }

    fn costs(&self, start: (usize, usize), min_run: usize, max_run: usize) -> Result<Costs> {
        let (x, y) = start;
        if x >= self.width || y >= self.height {
            bail!("dest outside of weight map");
        }
        let mut costs = Costs::new(self.height, self.width);
        let mut queue = VecDeque::new();
        // (coords, cost, run dir, run count)
        costs.set(x, y, 0, Up, 0);
        for (dir, neighbor) in self.neighbors(x, y) {
            queue.push_back((neighbor, 0_u32, dir, 1));
        }
        while let Some(((x, y), cost, dir, run)) = queue.pop_front() {
            let cost = cost + self.get(x, y);
            if costs.set(x, y, cost, dir, run) {
                for (next_dir, neighbor) in self.neighbors(x, y) {
                    if next_dir == dir {
                        let next_run = run + 1;
                        if next_run <= max_run {
                            queue.push_back((neighbor, cost, next_dir, run + 1));
                        }
                    } else if next_dir != dir.reverse() {
                        if run >= min_run {
                            queue.push_back((neighbor, cost, next_dir, 1));
                        }
                    }
                }
            }
        }
        Ok(costs)
    }
}

impl Debug for Weights {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Weights: ")?;
        for row in &self.inner {
            for weight in row {
                write!(f, "{weight}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
struct RunCost(HashMap<(Direction, usize), u32>);

impl RunCost {
    fn min(&self) -> Option<u32> {
        self.0.values().min().copied()
    }
}

struct Costs {
    inner: Vec<Vec<RunCost>>,
}

impl Costs {
    fn new(height: usize, width: usize) -> Self {
        let row: Vec<_> = iter::repeat_with(RunCost::default).take(width).collect();
        let inner: Vec<_> = iter::repeat_with(|| row.clone()).take(height).collect();
        Costs { inner }
    }

    // Sets the cost at the given coordinates iff the cost is less than the existing
    // cost at those coordinates. Returns whether the passed-in value was used
    fn set(&mut self, x: usize, y: usize, cost: u32, direction: Direction, run: usize) -> bool {
        let run_cost = &mut self.inner[y][x];
        match run_cost.0.entry((direction, run)) {
            Entry::Occupied(mut e) => {
                if *e.get() > cost {
                    e.insert(cost);
                    true
                } else {
                    false
                }
            }
            Entry::Vacant(e) => {
                e.insert(cost);
                true
            }
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<u32> {
        self.inner[y][x].min()
    }
}

impl Display for Costs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let block_size = self.inner
            .iter()
            .flatten()
            .map(|c| c.min().map(|c| c.to_string().len()).unwrap_or(1))
            .max()
            .unwrap_or(1) + 1;
        writeln!(f, "Costs")?;
        for line in &self.inner {
            for cost in line {
                if let Some(cost) = cost.min() {
                    write!(f, "{cost:w$}", w = block_size)?;
                } else {
                    write!(f, "{x:w$}", x = 'X', w = block_size)?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Debug for Costs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Costs: ")?;
        for (y, line) in self.inner.iter().enumerate() {
            for (x, cost) in line.iter().enumerate() {
                writeln!(f, "({x}, {y}): {:?}, ", cost.0)?;
            }
        }
        writeln!(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn reverse(&self) -> Direction {
        match self {
            Up => Down,
            Down => Up,
            Left => Right,
            Right => Left,
        }
    }
}
