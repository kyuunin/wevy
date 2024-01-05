use bevy::utils::hashbrown::HashSet;

use crate::multi_vec::*;

enum Direction
{
    Up,
    Left,
    Down,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl From<Direction> for (i32, i32)
{
    fn from(value: Direction) -> Self {
        match value {
            Direction::Up => (0, -1),
            Direction::Left => (-1, 0),
            Direction::Down => (0, 1),
            Direction::Right => (1, 0),
            Direction::UpLeft => (-1, -1),
            Direction::UpRight => (1, -1),
            Direction::DownLeft => (-1, 1),
            Direction::DownRight => (1, 1),
        }
    }
}

struct Pattern
{
    occurrences: usize,
    flat_definition: Vec<i32>,
    probability: f32,
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
                occurrences: 0,
                flat_definition: Vec::new(),
                probability: 0f32,
            };

            for y in y_start..(y_start + pattern_size)
            {
                for x in x_start..(x_start + pattern_size)
                {
                    match train_data.get(x, y) {
                        Some(value) => pattern.flat_definition.push(*value),
                        None => panic!("Böse :("),
                    }
                }
            }

            match patterns.iter().position(|old_pattern: &Pattern| old_pattern.flat_definition == pattern.flat_definition) {
                Some(index) =>
                {
                    patterns[index].occurrences += 1;
                    
                    continue;
                },
                None => { },
            }

            pattern.occurrences = 1;

            patterns.push(pattern);
        }
    }

    let sum_occurrences: usize = patterns
        .iter()
        .map(|x| x.occurrences)
        .sum();

    for pattern in patterns.iter_mut()
    {
        pattern.probability = pattern.occurrences as f32 / sum_occurrences as f32;
    }

    patterns
}

fn get_valid_directions(x: i32, y: i32, output_length: usize) -> Vec<Direction>
{
    match (x, y)
    {
        (0, 0) => vec! [ Direction::Right, Direction::Down, Direction::DownRight ],
        (0, y) if y == output_length as i32 - 1 => vec! [ Direction::Right, Direction::Up, Direction::UpRight ],
        (0, _) => vec! [ Direction::Right, Direction::Down, Direction::DownRight, Direction::Up, Direction::UpRight ],
        (x, 0) if x == output_length as i32 -1 => vec! [ Direction::Left, Direction::Down, Direction::DownLeft ],
        (x, y) if x == output_length as i32 -1 && y == output_length as i32 -1 => vec! [ Direction::Left, Direction::Down, Direction::DownLeft, Direction::Up, Direction::UpLeft ],
        (x, _) if x == output_length as i32 -1 => vec! [ Direction::Left, Direction::Down, Direction::DownLeft, Direction::Up, Direction::UpLeft ],
        (_, 0) => vec! [ Direction::Left, Direction::Right, Direction::Down, Direction::DownLeft, Direction::DownRight ],
        (_, y) if y == output_length as i32 -1 => vec! [ Direction::Left, Direction::Right, Direction::Up, Direction::UpLeft, Direction::UpRight ],
        (_, _) => vec! [ Direction::Left, Direction::Up, Direction::UpLeft, Direction::UpRight, Direction::Down, Direction::DownLeft, Direction::DownRight ],
    }
}

pub fn create_map(
    train_data: MultiVec<i32>,
    output_length: usize,
    pattern_size: usize,
    seed: usize)
{
    let patterns = slice_into_patterns(train_data, pattern_size);
}