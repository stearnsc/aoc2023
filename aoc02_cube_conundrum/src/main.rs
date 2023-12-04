use std::collections::BTreeMap;
use std::iter::Sum;
use std::ops::{Add, Sub};
use std::str::FromStr;
use sdk::{debug, info, init, lines, trace};
use sdk::*;
use sdk::anyhow::anyhow;

fn main() -> Result<()> {
    init();
    let mut games: BTreeMap<Game, Vec<Counts>> = BTreeMap::new();
    for line in lines("aoc02_cube_conundrum/input.txt")? {
        let line = line?;
        let (game, pulls) = Counts::parse_line(&line).ok_or(anyhow!("Unable to parse counts from {line}"))?;
        games.insert(game, pulls);
    }

    let min_bag_size: BTreeMap<Game, Counts> = games
        .iter()
        .map(|(game, counts)| {
            let bag = counts
                .iter()
                .copied()
                .fold(Counts::default(), |a, b| a.max(b));
            (*game, bag)
        }).collect();

    let sum_of_min_powers: i32 = min_bag_size
        .into_iter()
        .map(|(_, bag)| bag.power())
        .sum();

    let valid_game_count: usize = games.iter()
        .filter(|(_, counts)| counts.iter().all(|c| is_valid(*c)))
        .map(|(game, _)| game.0)
        .sum();

    info!("Valid game ID sum: {valid_game_count}");
    info!("Minimum pag power: {sum_of_min_powers:?}");
    Ok(())
}


fn is_valid(counts: Counts) -> bool {
    const MAX_COUNTS: Counts = Counts {
        green: 13,
        red: 12,
        blue: 14,
    };
    let diff = MAX_COUNTS - counts;

    // If we counted more than our max of any color, this game doesn't count
    if diff.red < 0 || diff.blue < 0 || diff.green < 0 {
        debug!("{counts:?} has single color greater than max {MAX_COUNTS:?}");
        return false;
    }

    // If the total number of blocks pulled is greater than our max, doesn' count
    if diff.total() > MAX_COUNTS.total() {
        debug!("{counts:?} has total count {} greater than max {MAX_COUNTS:?} {}", diff.total(), MAX_COUNTS.total());
        return false;
    }

    true
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
struct Game(usize);

impl Game {
    // Game 1
    fn parse(game: &str) -> Option<Self> {
        trace!("Parsing game from {game}");
        let (game, number) = game.split_once(" ")?;
        if game != "Game" {
            return None;
        }
        let game = Game(usize::from_str(number).ok()?);
        trace!("Parsed {game:?}");
        Some(game)
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct Counts {
    green: i32,
    red: i32,
    blue: i32,
}

impl Add for Counts {
    type Output = Counts;

    fn add(self, rhs: Self) -> Self::Output {
        Counts {
            green: self.green + rhs.green,
            red: self.red + rhs.red,
            blue: self.blue + rhs.blue,
        }
    }
}

impl<'a> Add for &'a Counts {
    type Output = Counts;

    fn add(self, rhs: Self) -> Self::Output {
        Counts {
            green: self.green + rhs.green,
            red: self.red + rhs.red,
            blue: self.blue + rhs.blue,
        }
    }
}

impl Sum for Counts {
    fn sum<I: Iterator<Item=Self>>(iter: I) -> Self {
        iter.fold(Counts::default(), |a, b| a + b)
    }
}

impl Sub for Counts {
    type Output = Counts;

    fn sub(self, rhs: Self) -> Self::Output {
        Counts {
            green: self.green - rhs.green,
            red: self.red - rhs.red,
            blue: self.blue - rhs.blue,
        }
    }
}

impl Counts {
    // Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
    fn parse_line(line: &str) -> Option<(Game, Vec<Self>)> {
        trace!("Parsing line {line}");

        // 3 red
        fn parse_pull(pull: &str) -> Option<Counts> {
            trace!("Parsing pull from {pull}");
            let (number, color) = pull.split_once(" ")?;
            let number = i32::from_str(number).ok()?;
            let mut pull = Counts::default();
            match color {
                "red" => pull.red = number,
                "blue" => pull.blue = number,
                "green" => pull.green = number,
                _ => {}
            }
            trace!("Parsed {pull:?}");
            Some(pull)
        }

        let (game, pulls) = line.split_once(": ")?;
        let game = Game::parse(game)?;
        let pulls = pulls
            .split("; ")
            .map(|g| {
                g.split(", ")
                    .filter_map(parse_pull)
                    .sum()
            })
            .collect();

        trace!("Parsed line {line} to {game:?}, {pulls:?}");
        Some((game, pulls))
    }

    fn total(&self) -> i32 {
        self.red + self.green + self.blue
    }

    fn max(self, other: Self) -> Self {
        Counts {
            green: self.green.max(other.green),
            red: self.red.max(other.red),
            blue: self.blue.max(other.blue),
        }
    }

    fn power(self) -> i32 {
        self.red * self.blue * self.green
    }
}
