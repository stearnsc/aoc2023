use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;
use sdk::*;
use sdk::anyhow::anyhow;

fn main() -> Result<()> {
    init();
    let mut cards = Vec::new();
    for line in lines("day04_scratchcards/input.txt")? {
        let line = line?;
        let card = Card::parse(&line)?;
        cards.push(card);
    }
    debug!("Cards: {cards:?}");
    let points: usize = cards.iter().map(|c| c.points()).sum();
    info!("Points: {points}");
    let card_tally = tally(&cards);
    info!("Card counts: {card_tally}");
    Ok(())
}

fn tally(cards: &[Card]) -> usize {
    // card #, count
    let mut collected: BTreeMap<usize, usize> = BTreeMap::new();
    for card in cards {
        let num_collected = {
            let mut num_collected = collected.entry(card.number).or_default();
            *num_collected += 1;
            *num_collected
        };

        let matches = card.count_matches();
        if matches > 0 {
            let copies: Vec<_> = (0..matches).map(|i| i + 1 + card.number).collect();
            trace!("Card {}: {matches} matches, wins copies of {copies:?}", card.number);

            for card_num in copies {
                (*collected.entry(card_num).or_default()) += num_collected;
            }
        } else {
            trace!("Card {}: {matches} matches", card.number);
        }
    }
    collected.values().sum()
}

#[derive(Debug, Clone)]
struct Card {
    number: usize,
    winning_numbers: BTreeSet<usize>,
    picked_numbers: Vec<usize>,
}

impl Card {
    fn parse(input: &str) -> Result<Self> {
        trace!("Parsing {input}");
        let (header, body) = input.split_once(": ")
            .ok_or(anyhow!("unable to split header and body"))?;
        let (card, card_number) = header.split_once(" ")
            .ok_or(anyhow!("unable to split 'Card' and card number"))?;
        if card != "Card" {
            return Err(anyhow!("Header {card} is not 'Card'"));
        }
        let card_number = usize::from_str(card_number.trim())?;

        let number_lines: Vec<_> = body.split(" | ").filter(|s| !s.is_empty()).collect();
        let (winning_line, chosen_line) = match number_lines.as_slice() {
            &[winning, chosen] => (winning, chosen),
            other => return Err(anyhow!("Unable to parse number lines. Found: {other:?}")),
        };

        fn parse_numbers(number_line: &str) -> Result<Vec<usize>> {
            let mut numbers = Vec::new();
            for number in number_line.split(" ") {
                if !number.is_empty() {
                    numbers.push(usize::from_str(number)?);
                }
            }
            Ok(numbers)
        }

        let winning_numbers = parse_numbers(winning_line)?.into_iter().collect();
        let picked_numbers = parse_numbers(chosen_line)?;

        Ok(Card {
            number: card_number,
            winning_numbers,
            picked_numbers,
        })
    }

    fn points(&self) -> usize {
        match self.count_matches() {
            0 => 0,
            more => 2_usize.pow((more - 1) as u32)
        }
    }

    fn count_matches(&self) -> usize {
        self.picked_numbers
            .iter()
            .filter(|n| self.winning_numbers.contains(*n))
            .count()
    }
}
