use std::cmp::{max, min};
use std::collections::BTreeMap;
use std::fs;
use std::ops::Range;
use std::str::FromStr;
use either::Either;
use sdk::*;
use sdk::anyhow::anyhow;
use sdk::winnow::combinator::{alt, delimited, separated};
use sdk::winnow::{Parser, PResult};
use sdk::winnow::ascii::dec_uint;
use sdk::winnow::token::{any, one_of, tag, take_while};

fn main() -> Result<()> {
    init();
    let input = fs::read_to_string("day19_aplenty/input.txt")?;
    let (workflows, parts) = parse(&input)?;
    debug!("Workflows: {workflows:?}");
    debug!("Parts: {parts:?}");

    let workflows: BTreeMap<_, _> = workflows.into_iter().map(|w| (w.name.clone(), w)).collect();
    let start = workflows.get("in").ok_or(anyhow!("Missing `in` workflow"))?;
    let mut accepted = Vec::new();
    for part in &parts {
        let mut workflow = start;
        trace!("{part:?}");
        loop {
            match workflow.examine(*part) {
                Output::Accepted => {
                    trace!("{}: Accepted", workflow.name);
                    accepted.push(*part);
                    break;
                }
                Output::Rejected => {
                    trace!("{}: Rejected", workflow.name);
                    break;
                }
                Output::Forward(next) => {
                    trace!("{}: forward to {next}", workflow.name);
                    workflow = workflows.get(&next).ok_or(anyhow!("Missing `{next}` workflow"))?;
                }
            }
        }
    }

    debug!("Accepted: {accepted:?}");
    let accepted_sum: u32 = accepted.iter().map(|a| a.sum()).sum();
    info!("Accepted sum: {accepted_sum}");

    let mut accepted = Vec::new();
    let part = PartRanges::new();
    let mut stack = vec![(part, start)];
    while let Some((part, workflow)) = stack.pop() {
        trace!("{part:?}");
        for (part, output) in workflow.restrict(&part) {
            match output {
                Output::Accepted => {
                    trace!("{part:?} {}: Accepted", workflow.name);
                    accepted.push(part)
                },
                Output::Rejected => {
                    trace!("{part:?} {}: Rejected", workflow.name);
                }
                Output::Forward(next) => {
                    trace!("{part:?} {}: forward to {next}", workflow.name);
                    let next = workflows.get(&next).ok_or_else(|| anyhow!("Missing `{next}` workflow"))?;
                    stack.push((part, next));
                }
            }
        }
    }

    for range in &accepted {
        debug!("{range:?}: {}", range.product());
        let overlaps: Vec<_> = accepted.iter().filter(|r| *r != range && r.overlaps(range))
            .collect();
        if !overlaps.is_empty() {
            warn!("Overlaps found for {range:?}: {overlaps:?}");
        }
    }
    let permutations: u64 = accepted.iter().map(|p| p.product()).sum();
    info!("Acceptable permutations: {permutations:?}");


    Ok(())
}

fn parse(input: &str) -> Result<(Vec<Workflow>, Vec<Part>)> {
    (
        separated(1.., Workflow::parse, '\n'),
        tag("\n\n"),
        separated(1.., Part::parse, '\n')
    ).map(|(workflows, _, parts)| (workflows, parts))
        .parse(input)
        .map_err(|e| anyhow!("parse error: {e:?}"))
}

#[derive(Debug, Clone, Copy)]
enum Op {
    Gt,
    Lt,
}

impl TryFrom<char> for Op {
    type Error = ParseError;

    fn try_from(c: char) -> std::result::Result<Self, Self::Error> {
        match c {
            '>' => Ok(Op::Gt),
            '<' => Ok(Op::Lt),
            other => Err(ParseError(format!("can't parse `{other}` as Output"))),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Field {
    X,
    M,
    A,
    S,
}

impl TryFrom<char> for Field {
    type Error = ParseError;

    fn try_from(c: char) -> std::result::Result<Self, Self::Error> {
        match c {
            'x' => Ok(Field::X),
            'm' => Ok(Field::M),
            'a' => Ok(Field::A),
            's' => Ok(Field::S),
            other => Err(ParseError(format!("can't parse `{other}` as Output"))),
        }
    }
}

#[derive(Debug, Clone)]
enum Output {
    Accepted,
    Rejected,
    Forward(String),
}

impl FromStr for Output {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "A" => Ok(Output::Accepted),
            "R" => Ok(Output::Rejected),
            name if name.chars().all(|c| c.is_alphabetic()) => Ok(Output::Forward(name.to_owned())),
            other => Err(ParseError(format!("can't parse `{other}` as Output"))),
        }
    }
}

#[derive(Debug, Clone)]
struct Workflow {
    name: String,
    rules: Vec<Rule>,
    output: Output,
}

impl Workflow {
    fn parse(input: &mut &str) -> PResult<Self> {
        (
            take_while(1.., |c: char| c.is_alphabetic()),
            delimited(
                '{',
                separated(
                    1..,
                    alt((
                        Rule::parse.map(|r| Either::Left(r)),
                        take_while(1.., |c: char| c.is_alphabetic())
                            .parse_to::<Output>()
                            .map(|o| Either::Right(o))
                    )),
                    ','
                ).try_map(|es: Vec<Either<Rule, Output>>| {
                    let mut rules = Vec::new();
                    for e in &es {
                        match e {
                            Either::Left(r) => rules.push(r.clone()),
                            Either::Right(output) => return Ok((rules, output.clone())),
                        }
                    }
                    Err(ParseError("Incorrect sequencing of rules".to_string()))
                }),
                '}',
            )
        )
            .map(|(name, (rules, output))| {
                Workflow { name: name.to_owned(), rules, output }
            })
            .parse_next(input)
    }

    fn examine(&self, part: Part) -> Output {
        for rule in &self.rules {
            if rule.apply(part) {
                return rule.output.clone();
            }
        }
        return self.output.clone();
    }

    fn restrict(&self, part: &PartRanges) -> Vec<(PartRanges, Output)> {
        let mut possibilities = Vec::new();
        let mut current = part.clone();
        for rule in &self.rules {
            let (included, excluded) = rule.restrict(&current);
            if let Some(part) = included {
                possibilities.push((part, rule.output.clone()));
            }
            if let Some(part) = excluded {
                current = part;
            } else {
                return possibilities;
            }
        }
        possibilities.push((current, self.output.clone()));
        possibilities
    }
}

#[derive(Debug, Clone)]
struct Rule {
    field: Field,
    op: Op,
    value: u32,
    output: Output,
}

impl Rule {
    pub fn apply(&self, part: Part) -> bool {
        let value = match self.field {
            Field::X => part.x,
            Field::M => part.m,
            Field::A => part.a,
            Field::S => part.s
        };
        match self.op {
            Op::Gt => value > self.value,
            Op::Lt => value < self.value,
        }
    }

    pub fn invert(&self) -> Self {
        Rule {
            op: match self.op {
                Op::Gt => Op::Lt,
                Op::Lt => Op::Gt,
            },
            ..self.clone()
        }
    }

    pub fn restrict(&self, part: &PartRanges) -> (Option<PartRanges>, Option<PartRanges>) {
        let mut included_part = part.clone();
        let mut excluded_part = part.clone();
        let (included, excluded) = match self.field {
            Field::X => (&mut included_part.x, &mut excluded_part.x),
            Field::M => (&mut included_part.m, &mut excluded_part.m),
            Field::A => (&mut included_part.a, &mut excluded_part.a),
            Field::S => (&mut included_part.s, &mut excluded_part.s)
        };
        match self.op {
            Op::Lt => {
                *included = included.start..min(included.end, self.value);
                *excluded = max(excluded.start, self.value)..excluded.end;
            },
            Op::Gt => {
                *included = max(included.start, self.value + 1)..included.end;
                *excluded = excluded.start..min(excluded.end, self.value + 1);
            },
        }
        (
            (!included.is_empty()).then_some(included_part),
            (!excluded.is_empty()).then_some(excluded_part),
        )
    }
}

impl Rule {
    fn parse(input: &mut &str) -> PResult<Self> {
        (
            one_of(['x', 'm', 'a', 's']).try_map(Field::try_from),
            one_of(['<', '>']).try_map(Op::try_from),
            dec_uint,
            ':',
            take_while(1.., |c: char| c.is_alphabetic()).parse_to::<Output>()
        )
            .map(|(field, op, value, _, output)| Rule { field, op, value, output })
            .parse_next(input)
    }
}

#[derive(Debug, Clone, Copy)]
struct Part {
    x: u32,
    m: u32,
    a: u32,
    s: u32,
}

impl Part {
    fn parse(input: &mut &str) -> PResult<Self> {
        fn parse_kv(input: &mut &str) -> PResult<(char, u32)> {
            (any, '=', dec_uint)
                .map(|(k, _, v)| (k, v))
                .parse_next(input)
        }
        ('{', separated(4..5, parse_kv, ','), '}')
            .map(|(_, kv, _)| {
                let kv: Vec<(char, u32)> = kv;
                Part {
                    x: kv[0].1,
                    m: kv[1].1,
                    a: kv[2].1,
                    s: kv[3].1,
                }
            })
            .parse_next(input)
    }

    fn sum(&self) -> u32 {
        self.x + self.m + self.a + self.s
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PartRanges {
    x: Range<u32>,
    m: Range<u32>,
    a: Range<u32>,
    s: Range<u32>,
}

impl PartRanges {
    fn new() -> Self {
        PartRanges {
            x: 1..4001,
            m: 1..4001,
            a: 1..4001,
            s: 1..4001,
        }
    }

    fn product(&self) -> u64 {
        let PartRanges { x, m, a, s } = self;
        let x = x.len();
        let m = m.len();
        let a = a.len();
        let s = s.len();
        (x * m * a * s) as u64
    }

    fn overlaps(&self, other: &PartRanges) -> bool {
        (self.x.start < other.x.end && other.x.start < self.x.end) &&
            (self.m.start < other.m.end && other.m.start < self.m.end) &&
            (self.a.start < other.a.end && other.a.start < self.a.end) &&
            (self.s.start < other.s.end && other.s.start < self.s.end)
    }
}

#[cfg(test)]
mod tests {
    use crate::PartRanges;

    #[test]
    fn product() {
        let range = PartRanges {
            x: 1..3,
            m: 1..3,
            a: 2..3,
            s: 1..3,
        };

        println!("{}", range.product());
    }
}