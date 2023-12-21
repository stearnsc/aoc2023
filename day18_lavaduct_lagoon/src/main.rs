use std::cmp::{max, min};
use std::collections::{BTreeSet};
use std::fmt::{Debug, Formatter};
use colored::{Colorize, CustomColor};
use sdk::*;
use sdk::anyhow::anyhow;
use sdk::winnow::{Parser, PResult};
use sdk::winnow::ascii::dec_uint;
use sdk::winnow::combinator::{separated};
use sdk::winnow::token::{one_of, take};
use crate::Direction::{Down, Left, Right, Up};

fn main() -> Result<()> {
    init();
    let plans = Plans::parse(&std::fs::read_to_string("day18_lavaduct_lagoon/input.txt")?)?;
    debug!("{plans:?}");
    let mut field = Field::from_plans(&plans);
    debug!("Initial field: {field:?}");
    field.dig_edges();
    debug!("Edges: {field:?}");
    field.dig_center();
    debug!("Center: {field:?}");
    let dug_area = field.dug_area();
    info!("Dug area: {dug_area}");
    Ok(())
}

#[derive(Clone)]
struct Plans {
    inner: Vec<TrenchPlan>,
}

impl Plans {
    fn parse(input: &str) -> Result<Self> {
        separated(1.., TrenchPlan::parse, '\n')
            .map(|t| Plans { inner: t })
            .parse(input)
            .map_err(|e| anyhow!("{e:?}"))
    }
}

impl Debug for Plans {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Plans:")?;
        for t in &self.inner {
            writeln!(f, "{t:?}")?;
        }
        Ok(())
    }
}


#[derive(Copy, Clone, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl TryFrom<char> for Direction {
    type Error = ParseError;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        match value {
            'U' => Ok(Up),
            'D' => Ok(Down),
            'L' => Ok(Left),
            'R' => Ok(Right),
            _ => Err(ParseError(format!("cannot parse `{value}` as direction"))),
        }
    }
}

impl Debug for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Up => 'U',
            Down => 'D',
            Left => 'L',
            Right => 'R',
        };
        write!(f, "{c}")
    }
}

#[derive(Clone, Copy)]
struct TrenchPlan {
    direction: Direction,
    len: usize,
    color: CustomColor,
}

impl TrenchPlan {
    fn parse(input: &mut &str) -> PResult<Self> {
        fn parse_color(input: &mut &str) -> PResult<CustomColor> {
            let color_parser = take(6_usize)
                .try_map(|s: &str| hex::decode(s))
                .map(|n| CustomColor::new(n[0], n[1], n[2]));
            ('(', '#', color_parser, ')')
                .map(|(_, _, color, _)| color)
                .parse_next(input)
        }

        let parser = (
            one_of(['U', 'D', 'L', 'R']).try_map(Direction::try_from),
            ' ',
            dec_uint::<_, u32, _>,
            ' ',
            parse_color
        );
        parser
            .map(|(direction, _, len, _, color)| TrenchPlan { direction, len: len as usize, color })
            .parse_next(input)
    }

    fn span(&self, from: (usize, usize)) -> Vec<(usize, usize)> {
        let (x, y) = from;
        let mut span = Vec::with_capacity(self.len);
        for i in 1..=self.len {
            match self.direction {
                Up => span.push((x, y - i)),
                Down => span.push((x, y + i)),
                Left => span.push((x - i, y)),
                Right => span.push((x + i, y)),
            }
        }
        span
    }
}

struct FormatColor(CustomColor);

impl Debug for FormatColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut bytes = ['#' as u8, 0, 0, 0, 0, 0, 0];
        let CustomColor { r, g, b } = self.0;
        hex::encode_to_slice(&[r, g, b], &mut bytes[1..]).map_err(|_| std::fmt::Error)?;
        let s = std::str::from_utf8(&bytes).map_err(|_| std::fmt::Error)?;
        write!(f, "({})", s.custom_color(self.0))
    }
}

impl Debug for TrenchPlan {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {} (#{:?})", self.direction, self.len, FormatColor(self.color))
    }
}

struct Field {
    plans: Vec<((usize, usize), TrenchPlan)>,
    height: usize,
    width: usize,
    inner: Vec<Vec<Option<CustomColor>>>,
    start: (usize, usize),
}

impl Field {
    fn from_plans(plans: &Plans) -> Self {
        let mut min_y = 0;
        let mut max_y = 0;
        let mut min_x = 0;
        let mut max_x = 0;
        let mut x = 0;
        let mut y = 0;

        let mut plans: Vec<_> = plans.inner.iter().map(|plan| {
            let start = (x, y);
            match plan.direction {
                Up => y -= plan.len as i32,
                Down => y += plan.len as i32,
                Left => x -= plan.len as i32,
                Right => x += plan.len as i32,
            }
            min_x = min(x, min_x);
            max_x = max(x, max_x);
            min_y = min(y, min_y);
            max_y = max(y, max_y);
            (start, plan)
        }).collect();

        let height = (max_y - min_y + 1) as usize;
        let width = (max_x - min_x + 1) as usize;
        let start_x = max(0, -1 * min_x) as usize;
        let start_y = max(0, -1 * min_y) as usize;
        let plans: Vec<_> = plans.into_iter().map(|((x, y), plan)| {
            (
                ((x + start_x as i32) as usize, (y + start_y as i32) as usize),
                *plan
            )
        }).collect();

        Field {
            plans,
            height,
            width,
            start: (start_x, start_y),
            inner: vec![vec![None; width]; height],
        }
    }

    fn dig_edges(&mut self) {
        let plans = self.plans.clone();
        for (coords, plan) in plans {
            self.trench(coords, &plan);
        }
    }

    fn dig_center(&mut self) {
        const DEFAULT_COLOR: CustomColor = CustomColor { r: 255, g: 255, b: 255 };

        let mut boundaries = BTreeSet::new();
        let mut last_direction = self.plans.last().map(|(_, p)| p.direction);
        for (i, (coords, plan)) in self.plans.iter().enumerate() {
            let next_direction = self.plans
                .get(i + 1)
                .or_else(|| self.plans.get(0))
                .map(|(_, plan)| plan.direction);

            if matches!(plan.direction, Up | Down) {
                let span = plan.span(*coords);
                boundaries.extend(&span[..(plan.len - 1)]);
            } else if last_direction == next_direction {
                boundaries.extend(plan.span(*coords));
            }
            last_direction = Some(plan.direction);
        }

        for (y, row) in self.inner.iter_mut().enumerate() {
            let mut inside = false;
            let mut inside_boundary = false;
            for (x, place) in row.iter_mut().enumerate() {
                if boundaries.contains(&(x, y)) {
                    inside_boundary = true;
                    continue;
                } else if place.is_none() {
                    if inside_boundary {
                        inside = !inside;
                    }
                    inside_boundary = false;
                    if inside {
                        *place = Some(DEFAULT_COLOR);
                    }
                }
            }
        }
    }

    fn trench(&mut self, from: (usize, usize), plan: &TrenchPlan) -> (usize, usize) {
        use Direction::*;
        let span = plan.span(from);
        for (x, y) in &span {
            self.inner[*y][*x] = Some(plan.color);
        }
        span.last().copied().unwrap_or(from)
    }

    fn dug_area(&self) -> usize {
        self.inner.iter().flatten().filter(|d| d.is_some()).count()
    }
}

impl Debug for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Field (start: ({}, {})):", self.start.0, self.start.1)?;
        for row in &self.inner {
            for place in row {
                if let Some(color) = place {
                    write!(f, "{}", "#".custom_color(*color))?
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}