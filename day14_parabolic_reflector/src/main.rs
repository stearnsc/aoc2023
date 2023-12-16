use std::collections::{BTreeMap};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::{fs, iter};
use sdk::*;

fn main() -> Result<()> {
    init();
    let load_direction = TiltDirection::North;

    let mut platform = Platform::parse(&fs::read_to_string("day14_parabolic_reflector/input.txt")?)?;
    debug!("Starting platform: {platform:?}");
    let load = platform.load(load_direction);
    debug!("Initial load: {load}");
    platform.tilt(TiltDirection::North);
    debug!("Tilted platform: {platform:?}");
    let load = platform.load(load_direction);
    info!("Tilted load: {load}");

    cycle(&mut platform, 1000000000);

    let load = platform.load(load_direction);
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

#[derive(Debug, Copy, Clone, Hash)]
enum Object {
    Rock,
    Fixed,
}

impl Object {
    fn char(&self) -> char {
        match self {
            Object::Rock => 'O',
            Object::Fixed => '#',
        }
    }

    fn from_char(c: char) -> Option<Self> {
        match c {
            'O' => Some(Object::Rock),
            '#' => Some(Object::Fixed),
            _ => None,
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.char())
    }
}

#[derive(Clone, Hash)]
struct Platform {
    inner: Vec<Vec<Option<Object>>>,
    width: usize,
    height: usize,
}

impl Debug for Platform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Platform ({} x {}):", self.width, self.height)?;
        for row in &self.inner {
            for &col in row {
                let char = col.map(|c| c.char()).unwrap_or('.');
                write!(f, "{char}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Platform {
    fn new(height: usize, width: usize) -> Self {
        let inner = iter::repeat_with(|| vec![None; width]).take(height).collect();
        Self { inner, height, width }
    }

    fn parse(input: &str) -> Result<Self> {
        let lines: Vec<_> = input.lines().collect();
        let height = lines.len();
        let width = lines[0].len();
        let mut platform = Platform::new(height, width);
        for (y, line) in lines.into_iter().enumerate() {
            for (x, c) in line.chars().enumerate() {
                if let Some(object) = Object::from_char(c) {
                    platform.put(x, y, object);
                }
            }
        }

        Ok(platform)
    }

    fn tilt(&mut self, direction: TiltDirection) {
        let mut blocks = self.edges(direction);
        for y in self.height_range(direction) {
            for x in self.width_range(direction) {
                match self.get(x, y) {
                    None => {}
                    Some(Object::Fixed) => {
                        self.set_block(&mut blocks, direction, x, y);
                    }
                    Some(Object::Rock) => {
                        self.take(x, y);
                        let (x, y) = self.stack_block(&mut blocks, direction, x, y);
                        self.put(x, y, Object::Rock);
                    }
                }
            }
        }
    }

    fn load(&self, direction: TiltDirection) -> usize {
        self.inner
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row
                    .iter()
                    .enumerate()
                    .map(move |(x, o)| ((x, y), o))
            })
            .filter(|(_, o)| matches!(o, Some(Object::Rock)))
            .map(|((x, y), _)| {
                match direction {
                    TiltDirection::North => self.height - y,
                    TiltDirection::East => x,
                    TiltDirection::South => y,
                    TiltDirection::West => self.width - x,
                }
            })
            .sum()
    }

    fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn height_range(&self, direction: TiltDirection) -> Box<dyn Iterator<Item=usize>> {
        match direction {
            TiltDirection::South => Box::new((0..self.height).rev()),
            _ => Box::new(0..self.height),
        }
    }

    fn width_range(&self, direction: TiltDirection) -> Box<dyn Iterator<Item=usize>> {
        match direction {
            TiltDirection::East => Box::new((0..self.width).rev()),
            _ => Box::new(0..self.width),
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<Object> {
        self.inner[y][x]
    }

    fn take(&mut self, x: usize, y: usize) -> Option<Object> {
        self.inner[y][x].take()
    }

    fn put(&mut self, x: usize, y: usize, object: Object) {
        self.inner[y][x] = Some(object)
    }

    fn stack_block(&self, blocks: &mut Vec<Option<usize>>, direction: TiltDirection, x: usize, y: usize) -> (usize, usize) {
        match direction {
            TiltDirection::North => {
                if let Some(y) = blocks[x] {
                    let new_y = y + 1;
                    blocks[x] = Some(new_y);
                    (x, new_y)
                } else {
                    blocks[x] = Some(0);
                    (x, 0)
                }
            }
            TiltDirection::East => {
                if let Some(x) = blocks[y] {
                    let new_x = x - 1;
                    blocks[y] = Some(new_x);
                    (new_x, y)
                } else {
                    let new_x = self.width - 1;
                    blocks[y] = Some(new_x);
                    (new_x, y)
                }
            }
            TiltDirection::South => {
                if let Some(y) = blocks[x] {
                    let new_y = y - 1;
                    blocks[x] = Some(new_y);
                    (x, new_y)
                } else {
                    let new_y = self.height - 1;
                    blocks[x] = Some(new_y);
                    (x, new_y)
                }
            }
            TiltDirection::West => {
                if let Some(x) = blocks[y] {
                    let new_x = x + 1;
                    blocks[y] = Some(new_x);
                    (new_x, y)
                } else {
                    blocks[y] = Some(0);
                    (0, y)
                }
            }
        }
    }

    fn edges(&self, direction: TiltDirection) -> Vec<Option<usize>> {
        match direction {
            TiltDirection::North => vec![None; self.width],
            TiltDirection::East => vec![Some(self.width); self.height],
            TiltDirection::South => vec![Some(self.height); self.width],
            TiltDirection::West => vec![None; self.height],
        }
    }

    fn set_block(&self, blocks: &mut Vec<Option<usize>>, direction: TiltDirection, x: usize, y: usize) {
        match direction {
            TiltDirection::North | TiltDirection::South => blocks[x] = Some(y),
            TiltDirection::East | TiltDirection::West => blocks[y] = Some(x),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum TiltDirection {
    North,
    East,
    South,
    West,
}
