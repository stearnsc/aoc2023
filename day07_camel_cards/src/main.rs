use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use sdk::*;
use sdk::anyhow::anyhow;

const JOKERS_WILD: bool = true;

fn main() -> Result<()> {
    init();
    let mut hands = Vec::new();
    for line in lines("day07_camel_cards/input.txt")? {
        let line = line?;
        hands.push(parse_line(&line)?);
    }

    debug!("Hands: {hands:?}");

    let hands = rank(&hands);

    for RankedHand { hand, bid, rank } in &hands {
        debug!("Hand {hand} with bid {bid} is ranked {rank} ({:?})", hand.type_());
    }

    let winnings: usize = hands.iter()
        .map(|h| h.bid * h.rank)
        .sum();

    info!("Winnings: {winnings}");
    Ok(())
}

fn parse_line(line: &str) -> Result<(Hand, usize)> {
    let (hand, bid) = match line.split_whitespace().collect::<Vec<_>>().as_slice() {
        &[hand, bid] => (hand, bid),
        _ => return Err(anyhow!("Unable to parse `{line}`: cannot split into hand and bid")),
    };
    let hand = Hand::parse(hand)?;
    let bid = usize::from_str(bid)?;
    Ok((hand, bid))
}

#[derive(Debug, Clone)]
struct RankedHand {
    hand: Hand,
    bid: usize,
    rank: usize,
}

fn rank(hands: &[(Hand, usize)]) -> Vec<RankedHand> {
    let mut hands = hands.to_vec();
    hands.sort();
    hands.into_iter()
        .enumerate()
        .map(|(i, (hand, bid))| RankedHand {
            hand,
            bid,
            rank: i + 1,
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Card(char);

impl Card {
    const CARDINALITY: usize = 14;

    fn rank(&self) -> usize {
        match self.0 {
            'A' => 14,
            'K' => 13,
            'Q' => 12,
            'J' => 11,
            'T' => 10,
            'X' => 1,
            other if other.is_numeric() => {
                other.to_digit(10).unwrap() as usize
            }
            _ => panic!("Illegal card")
        }
    }

    // Card index for use in arrays. Jokers ('X') are 0, '2's are 1, and going up from there
    fn index(&self) -> usize {
        self.rank() - 1
    }
}

impl TryFrom<char> for Card {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self> {
        let char = match value {
            '0'..='9' | 'T' | 'Q' | 'K' | 'A' => value,
            'J' if JOKERS_WILD => 'X',
            'J' => value,
            _ => return Err(anyhow!("Illegal card `{value}`")),
        };
        Ok(Card(char))
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Hand([Card; 5]);

impl Hand {
    fn parse(input: &str) -> Result<Self> {
        if input.len() != 5 {
            return Err(anyhow!("Illegal hand: {input} - expected 5 characters"));
        }
        let mut cards = [Card('2'); 5];
        for (i, card) in input.chars().enumerate() {
            cards[i] = card.try_into()?;
        }
        Ok(Hand(cards))
    }

    fn type_(&self) -> HandType {
        use HandType::*;

        let mut counts = [0_usize; Card::CARDINALITY];
        for card in &self.0 {
            counts[card.index()] += 1;
        }

        trace!("{self} counts: {counts:?}");

        let num_jokers = counts[0];
        let max_count = counts[1..].iter().cloned().max().unwrap_or(0);

        let type_ = if max_count == 5 {
            FiveOfAKind
        } else if max_count == 4 {
            FourOfAKind
        } else if max_count == 3 {
            if counts.iter().any(|&c| c == 2) {
                FullHouse
            } else {
                ThreeOfAKind
            }
        } else if max_count == 2 {
            if counts.iter().filter(|&&c| c == 2).count() == 2 {
                TwoPair
            } else {
                OnePair
            }
        } else {
            HighCard
        };

        trace!("{self} type before jokers: {type_:?}");

        let type_ = type_.with_jokers(num_jokers);
        trace!("{self} type after jokers: {type_:?}");
        type_
    }
}

impl Display for Hand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hand([")?;
        for card in &self.0 {
            write!(f, "{card}")?;
        }
        write!(f, "])")?;
        Ok(())
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.type_().cmp(&other.type_()) {
            Ordering::Equal => self.0.cmp(&other.0),
            non_equal => non_equal,
        }
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

impl HandType {
    fn with_jokers(self, num_jokers: usize) -> Self {
        use HandType::*;
        let mut type_ = self;
        for _ in 0..num_jokers {
            type_ = match type_ {
                HighCard => OnePair,
                OnePair => ThreeOfAKind,
                TwoPair => FullHouse,
                ThreeOfAKind | FullHouse => FourOfAKind,
                FourOfAKind | FiveOfAKind => FiveOfAKind,
            }
        }
        type_
    }

    fn rank(&self) -> usize {
        match self {
            HandType::HighCard => 0,
            HandType::OnePair => 1,
            HandType::TwoPair => 2,
            HandType::ThreeOfAKind => 3,
            HandType::FullHouse => 4,
            HandType::FourOfAKind => 5,
            HandType::FiveOfAKind => 6,
        }
    }
}

impl PartialOrd for HandType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HandType {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}
