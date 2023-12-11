use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::mem;
use itertools::Itertools;
use sdk::*;
use sdk::anyhow::anyhow;

fn main() -> Result<()> {
    init();
    let mut image = Image::parse(lines("day11_cosmic_expansion/input.txt")?)?;
    debug!("{image:?}");
    debug!("{image}");
    image.expand(1000000);
    if image.width < 1000 && image.height < 1000 {
        debug!("Expanded: {image}");
    } else {
        debug!("Expanded: width: {}, height: {}, galaxies: {}", image.width, image.height, image.galaxies.len());
    }
    let sum_of_distances = image.sum_of_distances();
    info!("Sum of distances: {sum_of_distances}");
    Ok(())
}

#[derive(Debug, Clone)]
struct Image {
    width: usize,
    height: usize,
    galaxies: BTreeSet<(usize, usize)>
}

impl Image {
    fn parse(input: impl Iterator<Item=String>) -> Result<Self> {
        let mut width = 0;
        let mut height = 0;
        let mut galaxies = BTreeSet::new();
        for (y, line) in input.enumerate() {
            width = line.len();
            height += 1;
            for (x, char) in line.chars().enumerate() {
                match char {
                    '#' => {
                        galaxies.insert((x, y));
                    },
                    '.' => {}
                    _ => return Err(anyhow!("Unepected character in input: {char}"))
                }
            }
        }
        Ok(Image { width, height, galaxies })
    }

    fn expand(&mut self, factor: usize) {
        let mut galactic_columns = BTreeSet::new();
        let mut galactic_rows = BTreeSet::new();
        for (x, y) in &self.galaxies {
            galactic_columns.insert(*x);
            galactic_rows.insert(*y);
        }
        let expanding_columns: BTreeSet<_> = (0..self.width).filter(|c| !galactic_columns.contains(c)).collect();
        let expanding_rows: BTreeSet<_> = (0..self.height).filter(|r| !galactic_rows.contains(r)).collect();

        // Replace each row with <factor> rows (the `-1` accounts for removing the original row)
        self.width += expanding_columns.len() * (factor - 1);
        self.height += expanding_rows.len() * (factor - 1);

        for (x, y) in mem::take(&mut self.galaxies) {
            let column_expansion = expanding_columns.iter().filter(|c| **c < x).count();
            let row_expansion = expanding_rows.iter().filter(|r| **r < y).count();
            self.galaxies.insert((x + (column_expansion * (factor - 1)), y + (row_expansion * (factor - 1))));
        }
    }

    fn sum_of_distances(&self) -> usize {
        self.galaxies
            .iter()
            .copied()
            .tuple_combinations()
            .map(|(a, b)| distance(a, b))
            .sum()
    }
}

fn distance((from_x, from_y): (usize, usize), (to_x, to_y): (usize, usize)) -> usize {
    from_x.abs_diff(to_x) + from_y.abs_diff(to_y)
}

impl Display for Image {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Image:")?;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.galaxies.contains(&(x, y)) {
                    write!(f, "#")?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}