use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::fs;
use sdk::*;
use sdk::anyhow::anyhow;

fn main() -> Result<()> {
    init();
    info!("Hello world");
    let input = fs::read_to_string("aoc08_haunted_wasteland/example.txt")?;
    let (directions, body) = input.split_once('\n').ok_or(anyhow!("Unexpected input format - unable to split directions from network"))?;
    let directions = parse_directions(directions)?;
    let network = Network::parse(body.trim())?;
    debug!("Directions: {directions:?}");
    debug!("{network:?}");

    // Part 1
    let steps = network.traverse_until(|n| n == &Node("AAA"), |n| n == &Node("ZZZ"), ListLoop::new(&directions).cloned());
    info!("Steps from AAA to ZZZ: {steps:?}");

    // Part 2
    let steps = network.traverse_until(|n| n.0.ends_with("A"), |n| n.0.ends_with("Z"), ListLoop::new(&directions).cloned());
    info!("Steps from AXX to XXZ: {steps:?}");

    Ok(())
}

#[derive(Clone)]
struct Network<'a> {
    nodes: Vec<Node<'a>>,
    edges: BTreeMap<Node<'a>, (usize, usize)>,
}

impl<'a> Network<'a> {
    fn parse(input: &'a str) -> Result<Self> {
        let mut nodes = BTreeMap::new();
        let mut edges = BTreeMap::new();

        let mut index = 0_usize;
        let mut add_node = |node| {
            *nodes.entry(node).or_insert_with(|| {
                let i = index;
                index += 1;
                i
            })
        };

        for line in input.lines() {
            let (node, node_edges) = line.split_once(" = ").ok_or(anyhow!("Unable to parse node/edges from {line}"))?;
            let node = Node(node);
            add_node(node);

            let (left, right) = node_edges.trim_start_matches("(").trim_end_matches(")").split_once(", ")
                .ok_or(anyhow!("Unable to parse edges from {node_edges}"))?;
            let left_index = add_node(Node(left));
            let right_index = add_node(Node(right));
            edges.insert(node, (left_index, right_index));
        }
        let mut nodes_list = vec![Node(""); nodes.len()];
        for (node, i) in nodes {
            nodes_list[i] = node;
        }
        Ok(Network { nodes: nodes_list, edges })
    }

    fn traverse(&'a self, start: Node<'a>, directions: impl IntoIterator<Item=Direction>) -> Node<'a> {
        let mut node = start;
        for direction in directions.into_iter() {
            match direction {
                Direction::Left => {
                    node = *self.left(&node);
                }
                Direction::Right => {
                    node = *self.left(&node);
                }
            }
        }
        node
    }

    fn traverse_until(&'a self, is_start: impl Fn(&Node<'a>) -> bool, is_end: impl Fn(&Node<'a>) -> bool, directions: impl IntoIterator<Item=Direction>) -> Option<usize> {
        let mut steps = 0;
        let starts: Vec<_> = self.nodes.iter().filter(|n| is_start(n)).copied().collect();
        let mut nodes: Vec<_> = starts.clone();
        trace!("Starts: {starts:?}");
        for direction in directions.into_iter() {
            if nodes.iter().all(|n| is_end(n)) {
                return Some(steps);
            }
            if steps > 0 {
                if let Some((i, n)) = nodes.iter().enumerate().find(|(i, n)| **n == starts[*i]) {
                    trace!("Node {n:?} back at start (start {i}) after {steps} steps");
                    return None;
                }
            }

            for node in &mut nodes {
                *node = match direction {
                    Direction::Left => *self.left(&node),
                    Direction::Right => *self.right(&node),
                }
            }
            steps += 1;
            if starts.len() > 1 {
                trace!("After {direction:?}: {nodes:?}");
            }
        }
        None
    }

    fn left(&self, node: &Node<'a>) -> &Node {
        let (left, _) = self.edges.get(node).unwrap();
        &self.nodes[*left]
    }

    fn right(&self, node: &Node) -> &Node {
        let (_, right) = self.edges.get(node).unwrap();
        &self.nodes[*right]
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
struct Node<'a>(&'a str);

impl<'a> Debug for Network<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Network {{\n")?;
        for node in &self.nodes {
            write!(f, "    {node:?}: ({:?}, {:?})\n", self.left(node), self.right(node))?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

fn parse_directions(line: &str) -> Result<Vec<Direction>> {
    line.trim().chars().map(Direction::try_from).collect()
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Left,
    Right,
}

impl TryFrom<char> for Direction {
    type Error = anyhow::Error;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        match value {
            'L' => Ok(Direction::Left),
            'R' => Ok(Direction::Right),
            _ => Err(anyhow!("`{value}` is not a valid direction"))
        }
    }
}

pub struct ListLoop<'a, T> {
    list: &'a [T],
    i: usize,
}

impl<'a, T> ListLoop<'a, T> {
    fn new(list: &'a [T]) -> Self {
        ListLoop { list, i: 0 }
    }
}

impl<'a, T> Iterator for ListLoop<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.list.is_empty() {
            return None;
        }
        let next = self.i;
        self.i += 1;
        if self.i == self.list.len() {
            self.i = 0;
        }
        Some(&self.list[next])
    }
}
