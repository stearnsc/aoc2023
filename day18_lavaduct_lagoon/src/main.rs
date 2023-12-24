use std::cmp::{max, min};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::ops::RangeInclusive;
use colored::{Colorize, CustomColor};
use itertools::Itertools;
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

    let plotted = plot_plans(&plans.inner);
    let calculated_area = calculate_area(&plotted)?;
    info!("Calculated dig area: {calculated_area}");
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

#[derive(Debug, Clone, Copy)]
struct PlottedPlan {
    x: usize,
    y: usize,
    direction: Direction,
    len: usize,
}

fn plot_plans(plans: &[TrenchPlan]) -> Vec<PlottedPlan> {
    let mut min_y = 0;
    let mut max_y = 0;
    let mut min_x = 0;
    let mut max_x = 0;
    let mut x = 0;
    let mut y = 0;

    let positions: Vec<_> = plans.iter().map(|plan| {
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
        start
    }).collect();

    let start_x = max(0, -1 * min_x) as usize;
    let start_y = max(0, -1 * min_y) as usize;
    positions
        .into_iter()
        .zip(plans)
        .map(|((x, y), plan)| {
            PlottedPlan {
                x: (x + start_x as i32) as usize,
                y: (y + start_y as i32) as usize,
                direction: plan.direction,
                len: plan.len,
            }
        })
        .collect()
}

fn calculate_area(plans: &[PlottedPlan]) -> Result<usize> {
    let mut corners: BTreeSet<_> = plans.iter().map(|p| (p.x, p.y)).collect();

    // first point is above second point
    let verticals: Vec<_> = plans
        .iter()
        .chain([&plans[0]])
        .tuple_windows()
        .filter_map(|(a, b)| {
            (a.x == b.x).then(|| ((a.x, min(a.y, b.y)), (a.x, max(a.y, b.y))))
        })
        .collect();

    let (xs, ys): (BTreeSet<_>, BTreeSet<_>) = corners.iter().copied().unzip();

    let corners: BTreeSet<_> = xs.into_iter().flat_map(|x| ys.iter().copied().map(move |y| (x, y))).collect();

    // for plan in plans {
    //     match plan.direction {
    //         Right => {
    //             for x in xs.range(plan.x..(plan.x + plan.len)) {
    //                 corners.insert((*x, plan.y));
    //             }
    //         }
    //         Left => {
    //             for x in xs.range((plan.x - plan.len)..plan.x) {
    //                 corners.insert((*x, plan.y));
    //             }
    //         }
    //         Up => {
    //             for y in ys.range((plan.y - plan.len)..plan.y) {
    //                 corners.insert((plan.x, *y));
    //             }
    //         }
    //         Down => {
    //             for y in ys.range(plan.y..(plan.y + plan.len)) {
    //                 corners.insert((plan.x, *y));
    //             }
    //         }
    //     }
    // }

    let mut corners_by_x: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
    let mut corners_by_y: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
    for (x, y) in corners {
        corners_by_x.entry(x).or_default().insert(y);
        corners_by_y.entry(y).or_default().insert(x);
    }

    // returns whether the square with top-left corner (x, y) is inside the structure
    let is_interior = |x, y| -> bool {
       verticals.iter()
           .filter(|&&((x1, y1), (x2, y2))| {
               x1 <= x && y1 <= y && y2 > y
           }).count() % 2 == 1
    };

    let mut area = 0;

    trace!("Verticals: {verticals:?}");

    let mut counted_verticals: BTreeMap<usize, Vec<RangeInclusive<usize>>> = BTreeMap::new();
    let mut counted_horizontals: BTreeMap<usize, Vec<RangeInclusive<usize>>> = BTreeMap::new();

    for (&y1, xs) in &corners_by_y {
        for (&x1, &x2) in xs.iter().tuple_windows() {
            let y2 = corners_by_x.get(&x1).and_then(|ys| ys.range((y1 + 1)..).next());
            if let Some(&y2) = y2 {
                if is_interior(x1, y1) {
                    let delta_area = ((x2 - x1) + 1) * ((y2 - y1) + 1);
                    let mut overlap = 0;
                    let mut corners = BTreeSet::new();

                    let y1_ranges = counted_horizontals.get(&y1);
                    let y1_overlap = y1_ranges.map(|r| intersect_size(&(x1..=x2), r));
                    let y2_ranges = counted_horizontals.get(&y2);
                    let y2_overlap = y2_ranges.map(|r| intersect_size(&(x1..=x2), r));
                    let x1_ranges = counted_verticals.get(&x1);
                    let x1_overlap = x1_ranges.map(|r| intersect_size(&(y1..=y2), r));
                    let x2_ranges = counted_verticals.get(&x2);
                    let x1_overlap = x2_ranges.map(|r| intersect_size(&(y1..=y2), r));

                    if let Some(ranges) = counted_horizontals.get(&y1) {
                        let range = x1..=x2;
                        overlap += intersect_size(&range, ranges);
                        if !corners.insert((x1, y1)) {
                            area += 1;
                        }
                        if !corners.insert((x2, y1)) {
                            area += 1;
                        }
                    }
                    if let Some(ranges) = counted_horizontals.get(&y2) {
                        let range = x1..=x2;
                        overlap += intersect_size(&range, ranges);
                        if !corners.insert((x1, y2)) {
                            area += 1;
                        }
                        if !corners.insert((x2, y2)) {
                            area += 1;
                        }
                    }
                    if let Some(ranges) = counted_verticals.get(&x1) {
                        let range = y1..=y2;
                        overlap += intersect_size(&range, ranges);
                        if !corners.insert((x1, y1)) {
                            area += 1;
                        }
                        if !corners.insert((x1, y2)) {
                            area += 1;
                        }
                    }
                    if let Some(ranges) = counted_verticals.get(&x2) {
                        let range = y1..=y2;
                        overlap += intersect_size(&range, ranges);
                        if !corners.insert((x2, y1)) {
                            area += 1;
                        }
                        if !corners.insert((x2, y2)) {
                            area += 1;
                        }
                    }
                    if overlap < delta_area {
                        area += (delta_area - overlap);
                    }
                    counted_verticals.entry(x1).or_default().push(y1..=y2);
                    counted_verticals.entry(x2).or_default().push(y1..=y2);
                    counted_horizontals.entry(y1).or_default().push(x1..=x2);
                    counted_horizontals.entry(y2).or_default().push(x1..=x2);
                    trace!("({x1}, {y1}), ({x2}, {y2}) INTERIOR (area {delta_area}, total {area})");
                } else {
                    trace!("({x1}, {y1}) ({x2}, {y2}) EXTERIOR");
                }
            }
        }
    }


    Ok(area)
}

fn intersect_size(range: &RangeInclusive<usize>, ranges: &[RangeInclusive<usize>]) -> usize {
    let ranges: Vec<_> = ranges.into_iter().collect::<HashSet<_>>().into_iter().collect();
    let mut union = Vec::new();
    for rr in ranges {
        if union.is_empty() {
            union.push(rr.clone());
        } else {
            union = union.into_iter().flat_map(|u| range_union(&u, rr)).collect();
        }
    }
    let mut sum = 0;
    for rr in union {
        let next = range_intersection(range, &rr);
        let count = next.count();
        sum += count
    }
    // union.iter().map(|rr| range_intersection(range, rr).count()).sum()
    sum
}

fn range_intersection(a: &RangeInclusive<usize>, b: &RangeInclusive<usize>) -> RangeInclusive<usize> {
    max(*a.start(), *b.start())..=min(*a.end(), *b.end())
}

fn range_union(a: &RangeInclusive<usize>, b: &RangeInclusive<usize>) -> Vec<RangeInclusive<usize>> {
    if a.end() == b.start() || a.start() == b.end() {
        return vec![min(*a.start(), *b.start())..=max(*a.end(), *b.end())];
    }
    let intersection = range_intersection(a, b);
    let start = min(*a.start(), *b.start());
    let end = max(*a.end(), *b.end());
    let union = [
        (start < *intersection.start()).then(|| start..=(intersection.start() - 1)),
        (intersection.clone().count() > 0).then(|| intersection.clone()),
        (end > *intersection.end()).then(|| (intersection.end() + 1)..=end),
    ];
    union.into_iter().flatten().filter(|r| r.clone().count() > 0).collect()
}

#[cfg(test)]
mod tests {
    use sdk::{info, init};
    use crate::{intersect_size, range_intersection, range_union};

    #[test]
    fn test() {
        init();
        let ranges = vec![
            0..=5,
            2..=10,
        ];

        info!("{:?}", intersect_size(&(1..=2), &ranges));
    }
}
