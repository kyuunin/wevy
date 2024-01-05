use crate::multi_vec::*;

#[derive(PartialEq, Eq)]
struct Pattern
{
    occurences: usize,
    definition: Vec<i32>
}

fn slice_into_patterns(
    train_data: MultiVec<i32>,
    pattern_size: usize) -> Vec<Pattern>
{
    let mut patterns = Vec::new();

    for y_start in (0..(train_data.h - pattern_size)).step_by(pattern_size)
    {
        for x_start in (0..(train_data.w - pattern_size)).step_by(pattern_size)
        {
            let mut pattern = Pattern
            {
                occurences: 0,
                definition: Vec::new()
            };

            for y in y_start..(y_start + pattern_size)
            {
                for x in x_start..(x_start + pattern_size)
                {
                    match train_data.get(x, y) {
                        Some(value) => pattern.definition.push(*value),
                        None => panic!("Böse :("),
                    }
                }
            }

            match patterns.iter().position(|old_pattern: &Pattern| *old_pattern == pattern) {
                Some(index) =>
                {
                    patterns[index].occurences += 1;
                    continue;
                },
                None => { },
            }

            pattern.occurences = 1;

            patterns.push(pattern);
        }
    }

    return patterns;
}

pub fn create_map(
    train_data: MultiVec<i32>,
    output_width: usize,
    output_height: usize,
    pattern_size: usize)
{
    let patterns = slice_into_patterns(train_data, pattern_size);
}