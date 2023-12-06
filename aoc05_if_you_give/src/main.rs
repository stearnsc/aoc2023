use std::cmp::{max, min};
use std::collections::{BTreeMap};
use std::{mem};
use std::ops::Range;
use std::path::Path;
use std::str::FromStr;
use sdk::*;
use sdk::anyhow::anyhow;

fn main() -> Result<()> {
    init();
    let almanac = Almanac::from_file("aoc05_if_you_give/input.txt")?;
    debug!("Parsed almanac: {almanac:?}");
    let transformed = almanac.run_seeds();
    debug!("Final seed locations: {transformed:?}");
    let (closest_seed, closest_location) = transformed.iter()
        .min_by_key(|(seed, location)| **location)
        .ok_or(anyhow!("No seeds mapped!"))?;
    info!("Closest location (seeds): {closest_location} (seed {closest_seed})");


    let location_ranges = almanac.run_ranges();
    debug!("Final location ranges: {location_ranges:?}");
    let min_location = location_ranges
        .iter()
        .map(|r| r.start)
        .min()
        .ok_or(anyhow!("No seeds mapped!"))?;

    info!("Closest location (ranges): {min_location}");

    Ok(())
}

#[derive(Debug, Clone)]
struct Transformation {
    from: String,
    to: String,
    ranges: Vec<TransformationRange>,
}

impl Transformation {
    fn transform_single(&self, input: usize) -> usize {
        for range in &self.ranges {
            if let Some(result) = range.transform_single(input) {
                return result;
            }
        }
        input
    }

    fn transform_ranges(&self, input: &[Range<usize>]) -> Vec<(Range<usize>, Range<usize>)> {
        // Keep track of the output for each input range
        let mut input: Vec<(Range<usize>, Option<Range<usize>>)> = input.iter().map(|r| (r.clone(), None)).collect();
        for range in &self.ranges {
            // collect anything not part of this range to be added back to input
            for (input_range, output) in mem::take(&mut input) {
                // Don't try to re-map what's already been mapped
                if output.is_some() {
                    input.push((input_range, output));
                    continue;
                }

                let transformed = range.transform_range(&input_range);
                // Add new mappings back to the dataset
                if let Some((mapped_input, mapped_output)) = transformed.transformed {
                    input.push((mapped_input, Some(mapped_output)));
                }

                // Add anything left over back to be iterated over again
                input.extend(transformed.excluded.into_iter().map(|r| (r, None)));
            }
        }
        input
            .into_iter()
            .map(|(input, output)| {
                // Anything that didn't get mapped is mapped 1:1 with input
                let output = output.unwrap_or_else(|| input.clone());
                (input, output)
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct TransformationRange {
    from: Range<usize>,
    to_start: usize,
}

impl TransformationRange {
    fn transform_single(&self, input: usize) -> Option<usize> {
        if self.from.contains(&input) {
            Some(self.to_start + (input - self.from.start))
        } else {
            None
        }
    }

    fn transform_range(&self, input: &Range<usize>) -> RangeTransformation {
        let mut transformation = RangeTransformation::default();
        if input.overlaps(&self.from) {
            if input.start < self.from.start {
                transformation.excluded.push(input.start..self.from.start);
            }
            if input.end > self.from.end {
                transformation.excluded.push(self.from.end..input.end)
            }
            let input_start = max(input.start, self.from.start);
            let input_end = min(input.end, self.from.end);
            let output_start = self.to_start + (input_start - self.from.start);
            let output_end = self.to_start + (input_end - self.from.start);
            transformation.transformed = Some((input_start..input_end, output_start..output_end));
        } else {
            transformation.excluded.push(input.clone());
        }

        transformation
    }
}

#[derive(Debug, Clone, Default)]
struct RangeTransformation {
    transformed: Option<(Range<usize>, Range<usize>)>,
    excluded: Vec<Range<usize>>,
}

#[derive(Debug, Clone)]
struct Almanac {
    // part 1
    seeds: Vec<usize>,
    // part 2
    seed_ranges: Vec<Range<usize>>,
    transformations: Vec<Transformation>,
}

impl Almanac {
    fn from_file(file: impl AsRef<Path>) -> Result<Self> {
        fn parse_header(line: &str) -> Option<(String, String)> {
            if !line.ends_with(" map:") {
                return None;
            }
            match line.trim_end_matches(" map:").split("-").collect::<Vec<_>>().as_slice() {
                &[from, "to", to] => Some((from.to_owned(), to.to_owned())),
                _ => {
                    error!("Unable to parse header {line}");
                    None
                }
            }
        }

        fn parse_seeds(seeds: &str) -> Result<Vec<usize>> {
            if !seeds.starts_with("seeds: ") {
                return Err(anyhow!("Unexpected start to seeds line: {seeds}"));
            }
            let seeds = seeds
                .trim_start_matches("seeds: ")
                .split(" ")
                .map(|s| usize::from_str(s))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            Ok(seeds)
        }

        fn parse_seed_ranges(seeds: &str) -> Result<Vec<Range<usize>>> {
            if !seeds.starts_with("seeds: ") {
                return Err(anyhow!("Unexpected start to seeds line: {seeds}"));
            }
            let mut ranges = Vec::new();
            let mut seeds = seeds.trim_start_matches("seeds: ")
                .split(" ");
            while let Some((start, length)) = seeds.next().zip(seeds.next()) {
                let start = usize::from_str(start)?;
                let length = usize::from_str(length)?;
                ranges.push(start..(start + length));
            }
            Ok(ranges)
        }

        fn parse_range(line: &str) -> Result<TransformationRange> {
            let numbers = line
                .split(" ")
                .map(|n| usize::from_str(n))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let (to, from, len) = match numbers.as_slice() {
                &[to, from, len] => (to, from, len),
                _ => return Err(anyhow!("Unable to parse number triple from {line}")),
            };
            Ok(TransformationRange {
                from: from..(from + len),
                to_start: to,
            })
        }

        let mut lines = lines(file)?;
        let seeds = lines.next().ok_or(anyhow!("Missing seeds line in input"))??;
        let seed_ranges = parse_seed_ranges(&seeds)?;
        let seeds = parse_seeds(&seeds)?;

        let mut transformations = Vec::new();
        let mut active = None;

        for line in lines {
            let line = line?;
            if let Some((from, to)) = parse_header(&line) {
                if let Some(active) = active.take() {
                    transformations.push(active);
                }
                active = Some(Transformation { from, to, ranges: Vec::new() });
            } else if line.starts_with(|c: char| c.is_numeric()) {
                let active = active.as_mut().ok_or(anyhow!("Found number line without active transofmration"))?;
                active.ranges.push(parse_range(&line)?);
            }
        }
        if let Some(active) = active {
            transformations.push(active)
        }
        Ok(Almanac {
            seeds,
            seed_ranges,
            transformations,
        })
    }

    // part 1
    fn run_seeds(&self) -> BTreeMap<usize, usize> {
        let mut seed_to_end = BTreeMap::new();
        for seed in &self.seeds {
            let mut stage_index = *seed;
            for transformation in &self.transformations {
                stage_index = transformation.transform_single(stage_index);
            }
            seed_to_end.insert(*seed, stage_index);
        }
        seed_to_end
    }

    fn run_ranges(&self) -> Vec<Range<usize>> {
        // Map of input (seed ranges) to current stage. After transformations, values will be final stage (locations)
        let mut ranges = self.seed_ranges.clone();
        for stage in &self.transformations {
            let range_maps = mem::take(&mut ranges);
            for (_, output) in stage.transform_ranges(&range_maps) {
                ranges.push(output)
            }
        }
        ranges
    }
}

trait RangeOverlap {
    fn overlaps(&self, other: &Self) -> bool;
}

impl<Idx: PartialOrd> RangeOverlap for Range<Idx> {
    fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }
}

#[cfg(test)]
mod tests {
    use crate::{RangeOverlap, TransformationRange};

    #[test]
    fn test_transformation_range() {
        let range = TransformationRange {
            from: 15..25,
            to_start: 35,
        };
        let cases = [
            (15, Some(35)),
            (25, None),
            (24, Some(44)),
            (0, None),
            (30, None),
        ];
        for (input, expected) in cases {
            assert_eq!(range.transform_single(input), expected);
        }
    }

    #[test]
    fn range_overlaps() {
        assert!((0..10).overlaps(&(0..10)));
        assert!((0..10).overlaps(&(9..10)));
        assert!((0..10).overlaps(&(9..20)));
        assert!((0..10).overlaps(&(3..4)));
        assert!((0..10).overlaps(&(9..10)));
        assert!((0..10).overlaps(&(0..1)));
        assert!(!(0..10).overlaps(&(10..20)));
        assert!((0..10).overlaps(&(0..10)));
        assert!((9..10).overlaps(&(0..10)));
        assert!((9..20).overlaps(&(0..10)));
        assert!((3..4).overlaps(&(0..10)));
        assert!((9..10).overlaps(&(0..10)));
        assert!((0..1).overlaps(&(0..10)));
        assert!(!(10..20).overlaps(&(0..10)));
    }
}
