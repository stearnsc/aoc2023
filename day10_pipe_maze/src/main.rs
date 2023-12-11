use std::cmp::min;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use sdk::*;
use sdk::anyhow::anyhow;

fn main() -> Result<()> {
    init();
    let maze = parse_maze(lines("day10_pipe_maze/input.txt")?)?;
    debug!("Maze: {maze}");
    let (farthest_node, dist) = maze.farthest_location();
    info!("Farthest location: {farthest_node} at {dist} steps");
    let area = maze.interior_area()?;
    info!("Interior area {area}");
    Ok(())
}

fn parse_maze(input: impl Iterator<Item=impl AsRef<str>>) -> Result<Maze> {
    let mut height = 0;
    let mut width = 0;
    let mut connections: BTreeMap<Coordinates, Vec<Coordinates>> = BTreeMap::new();
    let mut pipes: BTreeMap<Coordinates, char> = BTreeMap::new();
    let mut start = None;
    for (y, line) in input.enumerate() {
        height += 1;
        width = line.as_ref().len();
        for (x, c) in line.as_ref().chars().enumerate() {
            // Coordinates of neighbors can be negative
            let x = x as isize;
            let y = y as isize;

            let current = (x, y).into();
            let new_connections = match c {
                'S' => {
                    start = Some(current);
                    vec![
                        (x - 1, y),
                        (x + 1, y),
                        (x, y - 1),
                        (x, y + 1),
                    ]
                },
                '-' => vec![(x - 1, y), (x + 1, y)],
                '|' => vec![(x, y - 1), (x, y + 1)],
                'F' => vec![(x + 1, y), (x, y + 1)],
                'J' => vec![(x - 1, y), (x, y - 1)],
                'L' => vec![(x + 1, y), (x, y - 1)],
                '7' => vec![(x - 1, y), (x, y + 1)],
                _ => continue,
            };
            pipes.insert(current, c);
            connections.insert(current, Vec::new());
            let connections = connections.entry(current).or_default();
            for neighbor in new_connections.iter().copied() {
                let neighbor = neighbor.into();
                if !connections.contains(&neighbor) {
                    connections.push(neighbor);
                }
            }
        }
    }
    // Remove any one-way connections
    trace!("Removing one-way connections from {pipes:?}");
    let mut to_remove = BTreeSet::new();
    for (pipe, pipe_connections) in connections.iter() {
        for connection in pipe_connections {
            let two_way = connections
                .get(connection)
                .map(|conns_conns| conns_conns.contains(pipe))
                .unwrap_or_default();
            if !two_way {
                trace!("One-way connection found from {pipe} to {connection}");
                to_remove.insert((*pipe, *connection));
            }
        }
    }
    let start = start.ok_or(anyhow!("No start found"))?;

    connections.iter_mut().for_each(|(a, v)| v.retain(|b| !to_remove.contains(&(*a, *b))));
    let mut main_loop = BTreeSet::new();
    let mut stack = vec![start];
    while let Some(next) = stack.pop() {
        if main_loop.insert(next) {
            for connection in connections.get(&next).unwrap() {
                stack.push(*connection);
            }
        }
    }

    pipes.retain(|c, _| main_loop.contains(c));
    Ok(Maze { height, width, start, connections, pipes })
}

#[derive(Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
struct Coordinates {
    x: isize,
    y: isize,
}

impl From<(isize, isize)> for Coordinates {
    fn from((x, y): (isize, isize)) -> Self {
        Coordinates { x, y }
    }
}

impl From<(usize, usize)> for Coordinates {
    fn from((x, y): (usize, usize)) -> Self {
        Coordinates { x: x as isize, y: y as isize }
    }
}

impl Display for Coordinates {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Coordinates { x, y } = self;
        write!(f, "({x}, {y})")
    }
}

impl Debug for Coordinates {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(Debug, Clone)]
struct Maze {
    height: usize,
    width: usize,
    start: Coordinates,
    pipes: BTreeMap<Coordinates, char>,
    connections: BTreeMap<Coordinates, Vec<Coordinates>>,
}

impl Maze {
    fn farthest_location(&self) -> (Coordinates, usize) {
        let mut visited: BTreeMap<Coordinates, usize> = BTreeMap::new();
        let mut next = VecDeque::new();
        next.push_back((self.start, 0));
        while let Some((node, dist)) = next.pop_front() {
            if let Some(existing) = visited.get_mut(&node) {
                trace!("{node} found at {existing} (current: {dist})");
                *existing = min(*existing, dist);
                continue;
            } else {
                trace!("Adding {node} at {dist}");
                visited.insert(node, dist);
            }
            let connections = self.connections.get(&node).map(|n| n.as_slice()).unwrap_or_default();
            trace!("Adding next to check: {connections:?}");
            for connection in connections {
                next.push_back((*connection, dist + 1));
            }
        }
        let mut distances: Vec<_> = visited.into_iter().collect();
        distances.sort_by_key(|(_, dist)| *dist);
        debug!("distances: {distances:?}");
        print_distances(&distances);

        distances.last().unwrap().clone()
    }

    fn interior_area(&self) -> Result<usize> {
        // Direction to pipe when squeezing between pipes
        #[derive(Debug, Copy, Clone)]
        enum SqueezeDirection {
            Above,
            Below,
        }
        use SqueezeDirection::*;
        let mut area = 0;
        let mut lines = Vec::with_capacity(self.height);
        for y in 0..self.height {
            let mut line = String::with_capacity(self.width);
            let mut inside = false;
            let mut squeezing = None;
            for x in 0..self.width {
                // If we're at a pipe
                let coords = (x, y).into();
                if let Some(mut current) = self.pipes.get(&coords).copied() {
                    if current == 'S' {
                        current = self.start_char();
                    }
                    line.push(current);
                    match (squeezing, current) {
                        (_, '|') => {
                            inside = !inside;
                            squeezing = None;
                        }
                        (None, 'F') => {
                            squeezing = Some(Above);
                        }
                        (None, 'L') => {
                            squeezing = Some(Below);
                        }
                        (Some(_), '-') => {}
                        (Some(Above), 'J') => {
                            squeezing = None;
                            inside = !inside;
                        }
                        (Some(Below), 'J') => {
                            squeezing = None;
                        }
                        (Some(Above), '7') => {
                            squeezing = None;
                        }
                        (Some(Below), '7') => {
                            squeezing = None;
                            inside = !inside;
                        }
                        _ => {
                            return Err(anyhow!("Illegal state at {coords}: {current}, squeezing: {squeezing:?}"));
                        }
                    }
                } else {
                    squeezing = None;
                    if inside {
                        area += 1;
                        line.push('I');
                    } else {
                        line.push('O');
                    }
                }
            }
            lines.push(line);
        }
        println!("Inside outside map:");
        for line in lines {
            println!("{line}");
        }
        Ok(area)
    }

    fn start_char(&self) -> char {
        let start = self.start;
        let connections = self.connections.get(&self.start).expect("Illegal map state");
        let left = connections.iter().any(|c| c.x < start.x);
        let right = connections.iter().any(|c| c.x > start.x);
        let above = connections.iter().any(|c| c.y < start.y);
        let below = connections.iter().any(|c| c.y > start.y);
        match (left, right, above, below) {
            (false, false, true, true) => '|',
            (false, true, false, true) => 'F',
            (false, true, true, false) => 'L',
            (true, false, false, true) => '7',
            (true, false, true, false) => 'J',
            (true, true, false, false) => '-',
            _ => panic!("Illegal map state")
        }
    }
}

impl Display for Maze {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Maze:")?;
        for y in 0..self.height {
            for x in 0..self.width {
                let char = self.pipes.get(&(x, y).into()).copied().unwrap_or('.');
                write!(f, "{char}")?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

fn print_distances(distances: &[(Coordinates, usize)]) {
    let height = distances.iter().map(|(c, _)| c.y).max().unwrap();
    let width = distances.iter().map(|(c, _)| c.x).max().unwrap();
    let distances: BTreeMap<Coordinates, usize> = distances.into_iter()
        .map(|(c, d)| (*c, *d))
        .collect();
    let max_dist = distances.values().max().unwrap();
    let max_dist_size = max_dist.to_string().len();
    let print_size = max_dist_size + 2;
    for y in 0..=height {
        let mut line = String::with_capacity(width as usize);
        for x in 0..=width {
            if let Some(dist) = distances.get(&(x, y).into()) {
                line.push_str(&format!("{:^width$}", dist, width = print_size));
            } else {
                line.push_str(&format!("{:.^width$}", ".", width = print_size));
            }
        }
        println!("{line}");
    }
}
