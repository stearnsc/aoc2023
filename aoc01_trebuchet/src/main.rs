use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use sdk::*;
use sdk::anyhow::anyhow;
use std::collections::BTreeMap;

lazy_static! {
        static ref DIGITS: BTreeMap<&'static str, &'static str> = [
            ("one", "1"),
            ("two", "2"),
            ("three", "3"),
            ("four", "4"),
            ("five", "5"),
            ("six", "6"),
            ("seven", "7"),
            ("eight", "8"),
            ("nine", "9"),
        ].into_iter().collect();
    }

fn main() -> Result<()> {
    init();
    let file = File::open("aoc01_trebuchet/input.txt")?;
    let reader = BufReader::new(file);
    let mut sum = 0;
    for line in reader.lines() {
        let line = line?;
        let (a, b) = extract_digits_and_words(&line).ok_or(anyhow!("Unable to extract digits"))?;
        let digit: usize = [a, b].into_iter().collect::<String>().parse()?;
        trace!("Calibration for {line}: {digit}");
        sum += digit;
    }
    info!("Sum: {sum}");
    Ok(())
}

enum Parsed<'a> {
    Match(&'a str, usize),
    Partial,
    None,
}

fn extract_digits_and_words(s: &str) -> Option<(&str, &str)> {
    let mut start = 0;
    let mut end = 1;
    let mut matches = Vec::new();
    'outer: loop {
        'inner: loop {
            if start == s.len() {
                break 'outer;
            } else if end > s.len() {
                break 'inner;
            }
            let check = &s[start..end];
            if check.len() == 1 && u8::from_str(check).is_ok() {
                matches.push(check);
            }
            if let Some(value) = DIGITS.get(check) {
                matches.push(*value);
            }
            end += 1;
        }
        start += 1;
        end = start + 1;
    }
    if !matches.is_empty() {
        Some((matches[0], matches[matches.len() - 1]))
    } else {
        None
    }
}

fn replace_words(s: &str) -> String {
    let mut s = s.to_owned();
    let mut start = 0;
    let mut end = 1;
    'outer: loop {
        loop {
            if start == s.len() {
                break 'outer;
            } else if end > s.len() {
                break;
            }
            let to_check = &s[start..end];
            if let Some(&value) = DIGITS.get(to_check) {
                s = (&s[..start]).to_owned() + value + &s[end..];
                start += value.len();
                end = start + 1;
            } else {
                end += 1;
            }
        }
        start += 1;
        end = start + 1;
    }
    s
}

fn extract_digits(mut text: &str) -> Option<(char, char)> {
    let mut digits: Vec<_> = text.chars().filter(|c| c.is_digit(10)).collect();
    if !digits.is_empty() {
        Some((digits[0], digits[digits.len() - 1]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use sdk::init;
    use crate::{extract_digits, replace_words};

    #[test]
    fn digits() {
        init();
        let cases = [
            ("12", '1', '2'),
            ("sdlfk3abc4sdbp", '3', '4'),
            ("1234f5", '1', '5'),
            ("stuff6stuff1", '6', '1'),
            ("stuff7stuff", '7', '7'),
        ];
        for (case, expected_a, expected_b) in cases {
            let (a, b) = extract_digits(case).unwrap();
            assert_eq!(expected_a, a);
            assert_eq!(expected_b, b);
        }
    }

    #[test]
    fn replace() {
        let cases = [
            ("onetwothree", "123"),
            ("xonetwothree", "x123"),
            ("12onetwox", "1212x"),
        ];
        for (case, expected) in cases {
            assert_eq!(expected, replace_words(case));
        }
    }
}
