use std::collections::{BTreeMap, BTreeSet};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::mem;
use sdk::*;
use sdk::anyhow::bail;

fn main() -> Result<()> {
    init();
    let mut platform = Platform::parse(lines("day14_parabolic_reflector/input.txt")?)?;
    debug!("Starting platform: {platform:?}");
    let load = platform.load();
    debug!("Initial load: {load}");
    platform.tilt(TiltDirection::North);
    debug!("Tilted platform: {platform:?}");
    let load = platform.load();
    info!("Tilted load: {load}");

    cycle(&mut platform, 1000000000);

    let load = platform.load();
    info!("Cycled load: {load}");

    Ok(())
}

fn cycle(platform: &mut Platform, count: u64) {
    use TiltDirection::*;
    let mut positions: BTreeMap<u64, u64> = BTreeMap::new();
    positions.insert(platform.get_hash(), 0);
    let mut i = 0;
    while i < count {
        [North, West, South, East].into_iter().for_each(|dir| platform.tilt(dir));
        i += 1;
        let hash = platform.get_hash();
        if let Some(prev) = positions.insert(hash, i) {
            let cycle = i - prev;
            debug!("Cycle found: {prev} to {i}");
            let remaining = count - i;
            let skip = cycle * (remaining / cycle);
            debug!("Skipping {skip} cycles");
            i += skip;
        }
    }
    debug!("Cycled {count} times: {platform:?}");
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct Point {
    x: usize,
    y: usize,
}

impl Debug for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Point { x, y } = self;
        write!(f, "({x}, {y})")
    }
}

#[derive(Clone, Default, Hash)]
struct Platform {
    rocks: BTreeSet<Point>,
    fixed: BTreeSet<Point>,
    width: usize,
    height: usize,
}

impl Debug for Platform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Platform ({} x {}):", self.width, self.height)?;
        for y in 0..self.height {
            for x in 0..self.width {
                let point = Point { x, y };
                let c = self.rocks.contains(&point).then_some('O')
                    .or_else(|| self.fixed.contains(&point).then_some('#'))
                    .unwrap_or('.');
                write!(f, "{c}")?;
            }
            writeln!(f);
        }
        Ok(())
    }
}

impl Platform {
    fn parse(input: impl Iterator<Item=String>) -> Result<Self> {
        let mut platform = Platform::default();
        for (y, line) in input.enumerate() {
            platform.height += 1;
            platform.width = line.len();
            for (x, c) in line.chars().enumerate() {
                match c {
                    'O' => {
                        platform.rocks.insert(Point { x, y });
                    },
                    '#' => {
                        platform.fixed.insert(Point { x, y });
                    },
                    '.' => {},
                    _ => bail!("Unexpected character: at ({x}, {y}): {c}")
                }
            }
        }

        Ok(platform)
    }

    fn tilt(&mut self, direction: TiltDirection) {
        let mut rocks: Vec<_> = mem::take(&mut self.rocks).into_iter().collect();
        // Rocks in direction of tilt need to be moved first
        rocks.sort_unstable_by_key(|f| direction.sort_key(f));
        trace!("Sorted for {direction:?}: {rocks:?}");
        for rock in rocks {
            // Find any fixed point or rock blocking the path
            let block = self.fixed
                .iter()
                .chain(&self.rocks)
                .filter(|f| direction.is_in_front_of(*f, &rock))
                .max_by_key(|f| direction.sort_key(f));
            self.rocks.insert(direction.stack(&rock, block, self.height, self.width));
        }
    }

    fn load(&self) -> usize {
        let get_rock_load = |y: usize| self.height - y;
        self.rocks.iter().map(|p| get_rock_load(p.y)).sum()
    }

    fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Debug, Clone, Copy)]
enum TiltDirection {
    North,
    East,
    South,
    West
}

impl TiltDirection {
    fn is_in_front_of(&self, a: &Point, b: &Point) -> bool {
        match self {
            TiltDirection::North => a.x == b.x && a.y < b.y,
            TiltDirection::East => a.y == b.y && a.x > b.x,
            TiltDirection::South => a.x == b.x && a.y > b.y,
            TiltDirection::West => a.y == b.y && a.x < b.x,
        }
    }

    fn sort_key(&self, point: &Point) -> isize {
        match self {
            TiltDirection::North => point.y as isize,
            TiltDirection::East => -1 * (point.x as isize),
            TiltDirection::South => -1 * (point.y as isize),
            TiltDirection::West => point.x as isize,
        }
    }

    fn stack(&self, rock: &Point, on: Option<&Point>, height: usize, width: usize) -> Point {
        match self {
            TiltDirection::North => {
                let y = on.map(|on| on.y + 1).unwrap_or(0);
                Point { y, ..*rock }
            },
            TiltDirection::East => {
                let x = on.map(|on| on.x - 1).unwrap_or_else(|| width - 1);
                Point { x, ..*rock }
            }
            TiltDirection::South => {
                let y = on.map(|on| on.y - 1).unwrap_or_else(|| height - 1);
                Point { y, ..*rock }
            }
            TiltDirection::West => {
                let x = on.map(|on| on.x + 1).unwrap_or(0);
                Point { x, ..*rock }
            }
        }
    }
}
