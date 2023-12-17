use std::fmt::{Debug, Formatter};
use std::fs;
use sdk::*;
use sdk::anyhow::anyhow;
use crate::LightDirection::{Down, Left, Right, Up};

fn main() -> Result<()> {
    init();
    let input = fs::read_to_string("day16_floor_lava/input.txt")?;
    let grid = Grid::parse(&input);
    debug!("{grid:?}");
    let light = grid.light(0, 0, Right);
    debug!("{light:?}");
    let lit_count = light.lit_count();
    info!("Lit count: {lit_count}");

    let (max_count, (x, y, dir)) = grid.edges().into_iter()
        .map(|(x, y, dir)| {
            let lit_count = grid.light(x, y, dir).lit_count();
            (lit_count, (x, y, dir))
        })
        .max_by_key(|(lit_count, _)| *lit_count)
        .ok_or(anyhow!("no max found"))?;

    info!("Max start: ({x}, {y}, {dir:?}): {max_count}");
    Ok(())
}

#[derive(Clone, Copy)]
enum Optic {
    SplitterH,
    SplitterV,
    MirrorRight,
    MirrorLeft,
}

impl Optic {
    fn as_char(&self) -> char {
        match self {
            Optic::SplitterH => '-',
            Optic::SplitterV => '|',
            Optic::MirrorRight => '/',
            Optic::MirrorLeft => '\\',
        }
    }

    fn deflect(&self, light_direction: LightDirection) -> Vec<LightDirection> {
        match (light_direction, self) {
            (Right | Left, Optic::SplitterV) => vec![Up, Down],
            (Up | Down, Optic::SplitterH) => vec![Left, Right],
            (Right, Optic::MirrorLeft) => vec![Down],
            (Left, Optic::MirrorLeft) => vec![Up],
            (Up, Optic::MirrorLeft) => vec![Left],
            (Down, Optic::MirrorLeft) => vec![Right],
            (Right, Optic::MirrorRight) => vec![Up],
            (Left, Optic::MirrorRight) => vec![Down],
            (Up, Optic::MirrorRight) => vec![Right],
            (Down, Optic::MirrorRight) => vec![Left],
            _ => vec![light_direction],
        }
    }
}

impl Debug for Optic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl TryFrom<char> for Optic {
    type Error = ();

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        match value {
            '-' => Ok(Optic::SplitterH),
            '|' => Ok(Optic::SplitterV),
            '/' => Ok(Optic::MirrorRight),
            '\\' => Ok(Optic::MirrorLeft),
            _ => Err(()),
        }
    }
}

struct Grid {
    height: usize,
    width: usize,
    inner: Vec<Vec<Option<Optic>>>,
}

impl Grid {
    fn parse(input: &str) -> Self {
        let lines: Vec<_> = input.lines().collect();
        let height = lines.len();
        let width = lines.len();
        let mut grid = Grid {
            height,
            width,
            inner: vec![vec![None; width]; height],
        };
        for (y, line) in lines.iter().enumerate() {
            for (x, char) in line.chars().enumerate() {
                if let Ok(optic) = Optic::try_from(char) {
                    grid.put(x, y, optic);
                }
            }
        }
        grid
    }

    fn edges(&self) -> Vec<(usize, usize, LightDirection)> {
        let mut edges = Vec::new();
        for y in 0..self.height {
            edges.push((0, y, Right));
            edges.push((self.width - 1, y, Left));
        }
        for x in 0..self.width {
            edges.push((x, 0, Down));
            edges.push((x, self.height - 1, Up));
        }
        edges
    }

    fn put(&mut self, x: usize, y: usize, optic: Optic) {
        self.inner[y][x] = Some(optic);
    }

    fn light(&self, start_x: usize, start_y: usize, start_dir: LightDirection) -> GridLight {
        use LightDirection::*;

        let mut lit = GridLight::unlit(self);

        let next_direction = |x: usize, y: usize, dir: LightDirection| -> Option<(usize, usize)> {
            match dir {
                Right if x < self.width - 1 => Some((x + 1, y)),
                Left if x > 0 => Some((x - 1, y)),
                Up if y > 0 => Some((x, y - 1)),
                Down if y < self.width - 1 => Some((x, y + 1)),
                _ => None
            }
        };

        let mut stack: Vec<(usize, usize, LightDirection)> = vec![(start_x, start_y, start_dir)];

        while let Some((x, y, direction)) = stack.pop() {
            if lit.set(x, y, direction) {
                // This path has already been recorded
                continue;
            }
            let directions = self.inner[y][x]
                .map(|optic| optic.deflect(direction))
                .unwrap_or(vec![direction]);
            for direction in directions {
                if let Some((x, y)) = next_direction(x, y, direction) {
                    stack.push((x, y, direction));
                }
            }
        }

        lit
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Grid:")?;
        for row in &self.inner {
            for optic in row {
                let c = optic.as_ref().map(|o| o.as_char()).unwrap_or('.');
                write!(f, "{c}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum LightDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, Default)]
struct LitFrom {
    left: bool,
    right: bool,
    above: bool,
    below: bool,
}

struct GridLight {
    inner: Vec<Vec<Option<LitFrom>>>
}

impl GridLight {
    fn unlit(grid: &Grid) -> Self {
        GridLight { inner: vec![vec![None; grid.width]; grid.height] }
    }

    // Sets the correct lit direction, and returns whether that light direction has already been
    // recorded for that point
    fn set(&mut self, x: usize, y: usize, dir: LightDirection) -> bool {
        let mut lit_from = &mut self.inner[y][x];
        if lit_from.is_none() {
            *lit_from = Some(LitFrom::default());
        }
        let lit_from = lit_from.as_mut().unwrap();

        match dir {
            Left => {
                let set = lit_from.right;
                lit_from.right = true;
                set
            },
            Right => {
                let set = lit_from.left;
                lit_from.left = true;
                set
            },
            Up => {
                let set = lit_from.below;
                lit_from.below = true;
                set
            },
            Down => {
                let set = lit_from.above;
                lit_from.above = true;
                set
            },
        }
    }

    fn lit_count(&self) -> usize {
        self.inner.iter().flatten().filter(|o| o.is_some()).count()
    }
}

impl Debug for GridLight {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "GridLight:")?;
        for row in &self.inner {
            for place in row {
                let char = place.as_ref().is_some().then_some('#').unwrap_or('.');
                write!(f, "{char}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
