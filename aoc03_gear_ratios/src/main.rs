use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;
use sdk::*;

fn main() -> Result<()> {
    init();
    let input = std::fs::read_to_string("aoc03_gear_ratios/input.txt")?;
    let schematic = Schematic::parse(&input)?;

    let part_numbers: usize = schematic
        .neighbors()
        .iter()
        .filter_map(|(element, neighbors)| {
            element.element.number().filter(|_| neighbors.iter().any(|e| e.is_symbol()))
        })
        .sum();

    let gear_ratios: usize = schematic.neighbors().iter().filter_map(|(element, neighbors)| {
        element.element
            .symbol()
            .filter(|c| *c == '*')
            .and_then(|_| {
                let numeric_neighbors: Vec<_> = neighbors.iter().filter_map(|n| n.element.number()).collect();
                match numeric_neighbors.as_slice() {
                    &[a, b] => Some(a * b),
                    _ => None,
                }
            })
    }).sum();

    info!("Sum of part numbers: {part_numbers}");
    info!("Sum of gear ratios: {gear_ratios}");
    Ok(())
}

#[derive(Debug, Clone)]
struct Schematic {
    data: Vec<SchematicData>,
    rows: Vec<Vec<Option<SchematicData>>>,
    num_rows: usize,
    num_cols: usize,
}

impl Schematic {
    fn parse(input: &str) -> Result<Self> {
        let mut data = Vec::new();
        let mut rows = Vec::new();
        for line in input.lines() {
            println!("{line}");
        }
        for (row, line) in input.lines().enumerate() {
            let mut current_row = Vec::new();
            let mut current = TokenBuffer::default();
            for (col, char) in line.chars().enumerate() {
                current_row.push(None);
                if char.is_numeric() {
                    current.push(char, col);
                } else {
                    Self::finish_number(&mut data, &mut current_row, &mut current, row)?;
                    if char != '.' {
                        let element = SchematicData {
                            element: SchematicElement::Symbol(char),
                            row,
                            start: col,
                            end: col + 1,
                        };
                        data.push(element);
                        current_row[col] = Some(element);
                    }
                }

                if col + 1 == line.len() {
                    Self::finish_number(&mut data, &mut current_row, &mut current, row)?;
                }
            }
            rows.push(current_row);
        }
        let schematic = Schematic {
            num_rows: rows.len(),
            num_cols: rows[0].len(),
            data,
            rows,
        };
        trace!("Loaded schematic: {schematic:?}");
        Ok(schematic)
    }

    fn finish_number(data: &mut Vec<SchematicData>, row: &mut Vec<Option<SchematicData>>, number: &mut TokenBuffer, row_index: usize) -> Result<()> {
        if let Some((number, start)) = number.take() {
            let element = SchematicData {
                element: SchematicElement::Number(usize::from_str(&number)?),
                row: row_index,
                start,
                end: start + number.len(),
            };
            data.push(element);
            for i in start..start + number.len() {
                row[i] = Some(element);
            }
        }
        Ok(())
    }

    fn neighbors(&self) -> Vec<(SchematicData, Vec<SchematicData>)> {
        self.data.iter().map(|feature| {
            let element = feature.element;
            let mut neighbors = BTreeSet::new();

            let column_before = (feature.start > 0).then(|| feature.start - 1);
            let column_after = (feature.end + 1 < self.num_cols).then(|| feature.end);

            let row_above = (feature.row > 0).then(|| feature.row - 1);
            let row_below = (feature.row + 1 < self.num_rows).then(|| feature.row + 1);

            let mut add_neighbor = |row: usize, col: usize| {
                if let Some(element) = self.rows[row][col] {
                    neighbors.insert(element);
                }
            };

            if let Some(column) = column_before {
                add_neighbor(feature.row, column);
            }

            if let Some(column) = column_after {
                add_neighbor(feature.row, column);
            }

            if let Some(row) = row_above {
                let start = column_before.unwrap_or(feature.start);
                let end = column_after.unwrap_or(feature.end - 1);
                for column in start..=end {
                    add_neighbor(row, column);
                }
            }

            if let Some(row) = row_below {
                let start = column_before.unwrap_or(feature.start);
                let end = column_after.unwrap_or(feature.end - 1);
                for column in start..=end {
                    add_neighbor(row, column);
                }
            }

            trace!("{element} (row {}, col {}) does not match", feature.row, feature.start);
            (*feature, neighbors.into_iter().collect::<Vec<_>>())
        }).collect()
    }

    // A number is a part number if it is adjascent (including diagonals) to a symbol
    #[allow(unused)]
    fn part_numbers(&self) -> Vec<SchematicData> {
        self.data.iter().filter(|element| {
            let number = match element.element {
                SchematicElement::Number(n) => n,
                SchematicElement::Symbol(_) => return false,
            };

            let column_before = (element.start > 0).then(|| element.start - 1);
            let column_after = (element.end + 1 < self.num_cols).then(|| element.end);

            let row_above = (element.row > 0).then(|| element.row - 1);
            let row_below = (element.row + 1 < self.num_rows).then(|| element.row + 1);

            let is_symbol = |row: usize, col: usize| {
                let element = self.rows[row][col];
                element.map(|d| d.is_symbol()).unwrap_or(false)
            };

            if let Some(column) = column_before {
                if is_symbol(element.row, column) {
                    trace!("{number} (row {}, col {}) matches on previous column", element.row, element.start);
                    return true;
                }
            }

            if let Some(column) = column_after {
                if is_symbol(element.row, column) {
                    trace!("{number} (row {}, col {}) matches on following column", element.row, element.start);
                    return true;
                }
            }

            if let Some(row) = row_above {
                let start = column_before.unwrap_or(element.start);
                let end = column_after.unwrap_or(element.end - 1);
                for column in start..=end {
                    if is_symbol(row, column) {
                        trace!("{number} (row {}, col {}) matches on previous row, column {column}", element.row, element.start);
                        return true;
                    }
                }
            }

            if let Some(row) = row_below {
                let start = column_before.unwrap_or(element.start);
                let end = column_after.unwrap_or(element.end - 1);
                for column in start..=end {
                    if is_symbol(row, column) {
                        trace!("{number} (row {}, col {}) matches on following row, column {column}", element.row, element.start);
                        return true;
                    }
                }
            }

            trace!("{number} (row {}, col {}) does not match", element.row, element.start);
            false
        }).copied().collect()
    }
}

#[derive(Debug, Clone, Default)]
struct TokenBuffer {
    data: Option<(String, usize)>,
}

impl TokenBuffer {
    fn push(&mut self, c: char, i: usize) {
        match &mut self.data {
            Some((data, _)) => data.push(c),
            data @ None => *data = Some((c.to_string(), i)),
        }
    }

    fn take(&mut self) -> Option<(String, usize)> {
        self.data.take()
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
struct SchematicData {
    element: SchematicElement,
    row: usize,
    start: usize,
    end: usize,
}

impl SchematicData {
    fn is_symbol(&self) -> bool {
        match self.element {
            SchematicElement::Symbol(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
enum SchematicElement {
    Number(usize),
    Symbol(char),
}

impl SchematicElement {
    fn number(self) -> Option<usize> {
        match self {
            SchematicElement::Number(n) => Some(n),
            _ => None,
        }
    }

    fn symbol(self) -> Option<char> {
        match self {
            SchematicElement::Symbol(c) => Some(c),
            _ => None,
        }
    }

    #[allow(unused)]
    fn is_symbol(&self) -> bool {
        match self {
            SchematicElement::Symbol(_) => true,
            _ => false,
        }
    }

    #[allow(unused)]
    fn is_number(&self) -> bool {
        match self {
            SchematicElement::Symbol(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for SchematicElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchematicElement::Number(n) => write!(f, "{n}"),
            SchematicElement::Symbol(c) => write!(f, "{c}"),
        }
    }
}