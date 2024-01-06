use std::collections::HashMap;

use enum_iterator::{Sequence, all};

use crate::multi_vec::*;

// all::<Direction>()
#[derive(Sequence, PartialEq, Eq, Hash)]
enum Direction
{
    None,
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
            Direction::None => (0, 0),
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

struct RulesChecker
{
    rules: HashMap<usize, HashMap<Direction, Vec<usize>>>,
    // fn add_rule(&mut self)
}

impl RulesChecker
{
    pub fn new(patterns: Vec<Pattern>) -> Self
    {
        let mut empty_rules = HashMap::<usize, HashMap<Direction, Vec<usize>>>::new();

        for pattern_index in 0..patterns.len()
        {
            empty_rules.insert(pattern_index, HashMap::new());

            for direction in all::<Direction>()
            {
                let possible_patterns = empty_rules.get_mut(&pattern_index).expect("Alles ist kaputt :(");

                possible_patterns.insert(direction, Vec::new());
            }
        }

        Self
        {
            rules: empty_rules,
        }
    }

    pub fn add_rule(&mut self, current_pattern_index: usize, direction: Direction, next_pattern_index: usize)
    {
        let possible_patterns = self.rules
            .get_mut(&current_pattern_index)
            .expect("Rules ist nicht mit allen Patterns gefüllt")
            .get_mut(&direction)
            .expect("Rules ist nicht komplett mit Directions gefüllt");

        possible_patterns.push(next_pattern_index);
    }

    pub fn check_if_pattern_is_allowed(self, current_pattern_index: usize, direction: Direction, next_pattern_index: usize) -> bool
    {
        self.rules[&current_pattern_index][&direction].contains(&next_pattern_index)
    }
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

fn get_relevant_tiles_for_checking_overlapping_patterns(
    pattern: Pattern,
    direction_for_checking_overlapping: Direction,
    pattern_edge_length: usize) -> Vec<i32>
{
    match direction_for_checking_overlapping {
        Direction::None => pattern.flat_definition,
        Direction::UpLeft => vec! [ pattern.flat_definition[pattern_edge_length + 1] ],
        Direction::Up => pattern.flat_definition[pattern_edge_length..2*pattern_edge_length].to_vec(),
        Direction::UpRight => vec! [ pattern.flat_definition[pattern_edge_length] ],
        Direction::Left => vec! [ pattern.flat_definition[1], pattern.flat_definition[pattern_edge_length + 1] ],
        Direction::Right => vec! [ pattern.flat_definition[0], pattern.flat_definition[pattern_edge_length] ],
        Direction::DownLeft => vec! [ pattern.flat_definition[1] ],
        Direction::Down => pattern.flat_definition[0..pattern_edge_length].to_vec(),
        Direction::DownRight => vec! [ pattern.flat_definition[0] ]
    }
}

pub fn create_map(
    train_data: MultiVec<i32>,
    output_edge_length: usize,
    pattern_edge_length: usize,
    seed: usize)
{
    let patterns = slice_into_patterns(train_data, pattern_edge_length);
}