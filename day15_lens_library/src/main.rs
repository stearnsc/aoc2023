use std::array::IntoIter;
use std::fs;
use sdk::*;
use sdk::anyhow::anyhow;
use sdk::winnow::combinator::{opt, separated};
use sdk::winnow::{Parser, PResult};
use sdk::winnow::ascii::dec_uint;
use sdk::winnow::error::{ErrorKind, InputError};
use sdk::winnow::token::{one_of, take_until1, take_while};

fn main() -> Result<()> {
    init();
    let input = fs::read_to_string("day15_lens_library/input.txt")?;
    let sum: usize = input.split(',').map(hash).sum();
    info!("Hash sum {sum}");

    let commands = parse_input(&input)?;
    trace!("Commands: {commands:?}");
    let mut map = HashMap::new();
    for command in commands {
        debug!("{command:?}");
        match command {
            Command::Insert { key, value } => map.insert(key, value),
            Command::Remove { key } => map.remove(&key),
        }
        trace!("{map:?}");
    }
    /*
        One plus the box number of the lens in question.
        The slot number of the lens within the box: 1 for the first lens, 2 for the second lens, and so on.
        The focal length of the lens.
     */
    let sum: u32 = map
        .into_iter()
        .enumerate()
        .flat_map(|(box_number, bucket)| {
            bucket
                .into_iter()
                .enumerate()
                .map(move |(position, (_, v))| (box_number as u32 + 1) * (position as u32 + 1) * v)
        })
        .sum();
    info!("Focusing power: {sum}");
    Ok(())
}

fn hash(s: &str) -> usize {
    s.chars().fold(0_u32, |hash, next| {
        ((hash + (next as u32)) * 17) % 256
    }) as usize
}

#[derive(Debug, Clone)]
struct HashMap {
    buckets: [Vec<(String, u32)>; 256],
}

impl HashMap {
    fn new() -> Self {
        HashMap {
            buckets: [(); 256].map(|_| Vec::new()),
        }
    }

    fn insert(&mut self, key: String, value: u32) {
        let mut bucket = &mut self.buckets[hash(&key)];
        if let Some((_, existing)) = bucket.iter_mut().find(|(k, _)| k == &key) {
            *existing = value;
            return;
        }
        bucket.push((key, value));
    }

    fn remove(&mut self, key: &String) {
        self.buckets[hash(&key)].retain(|(k, _)| k != key)
    }
}

impl IntoIterator for HashMap {
    type Item = Vec<(String, u32)>;
    type IntoIter = IntoIter<Vec<(String, u32)>, 256>;

    fn into_iter(self) -> Self::IntoIter {
        self.buckets.into_iter()
    }
}

#[derive(Debug, Clone)]
enum Command {
    Insert {
        key: String,
        value: u32,
    },
    Remove {
        key: String,
    },
}

impl Command {
    fn parse(input: &mut &str) -> PResult<Self> {
        (
            take_while(1.., char::is_alphabetic),
            one_of(['=', '-']),
            opt(dec_uint::<_, u32, _>)
        ).try_map(|(key, op, value): (&str, char, Option<u32>)| {
            match (op, value) {
                ('=', Some(value)) => Ok(Command::Insert { key: key.to_owned(), value }),
                ('-', None) => Ok(Command::Remove { key: key.to_owned() }),
                _ => {
                    let input = format!("{key}{op}{}", value.unwrap_or_default());
                    error!("Cannot parse input `{input}`: (key: {key}, op: {op}, value: {value:?})");
                    Err(InputError::new(input, ErrorKind::Verify))
                }
            }
        })
            .parse_next(input)
    }
}

fn parse_input(input: &str) -> Result<Vec<Command>> {
    separated(0.., Command::parse, ',')
        .parse(input)
        .map_err(|e| anyhow!("{e:?}"))
}
