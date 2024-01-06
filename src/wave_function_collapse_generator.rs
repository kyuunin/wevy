use std::collections::HashMap;

use enum_iterator::{Sequence, all};
use rand::{rngs::StdRng, SeedableRng, Rng};

use crate::multi_vec::*;

// all::<Direction>()
#[derive(Sequence, PartialEq, Eq, Hash, Clone, Copy)]
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

impl TryFrom<(i32, i32)> for Direction {
    type Error = String;

    fn try_from(value: (i32, i32)) -> Result<Self, Self::Error> {
        match value {
            (0, 0) => Ok(Direction::None),
            (0, -1) => Ok(Direction::Up),
            (-1, 0) => Ok(Direction::Left),
            (0, 1) => Ok(Direction::Down),
            (1, 0) => Ok(Direction::Right),
            (-1, -1) => Ok(Direction::UpLeft),
            (1, -1) => Ok(Direction::UpRight),
            (-1, 1) => Ok(Direction::DownLeft),
            (1, 1) => Ok(Direction::DownRight),
            (x, y) => Err(format!("Wrong direction! X = {x}, Y = {y}")),
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
}

impl RulesChecker
{
    pub fn new(patterns: &Vec<Pattern>) -> Self
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
    pattern: &Pattern,
    direction_for_checking_overlapping: Direction,
    pattern_edge_length: usize) -> Vec<i32>
{
    match direction_for_checking_overlapping {
        Direction::None => pattern.flat_definition.clone(),
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

fn train_rules(patterns: &Vec<Pattern>, pattern_edge_length: usize, rules_checker: &mut RulesChecker)
{
    for current_pattern_for_rule_extraction_index in 0..patterns.len()
    {
        for direction in all::<Direction>()
        {
            for next_pattern_for_rule_extraction_index in 0..patterns.len()
            {
                let overlapping_tiles_next_pattern = get_relevant_tiles_for_checking_overlapping_patterns(&patterns[next_pattern_for_rule_extraction_index], direction, pattern_edge_length);
                
                let direction_as_tuple:(i32, i32) = direction.into();
                let opposite_direction_as_tuple = (direction_as_tuple.0 * -1, direction_as_tuple.1 * -1);

                let opposite_direction: Direction = opposite_direction_as_tuple.try_into().expect("Error when getting opposite direction");
                
                let overlapping_tiles_current_pattern = get_relevant_tiles_for_checking_overlapping_patterns(&patterns[current_pattern_for_rule_extraction_index], opposite_direction, pattern_edge_length);

                if overlapping_tiles_next_pattern == overlapping_tiles_current_pattern
                {
                    rules_checker.add_rule(current_pattern_for_rule_extraction_index, direction, next_pattern_for_rule_extraction_index)
                }
            }
        }
    }
}

fn initialize_possibilities_for_tiles(output_edge_length: usize, all_pattern_indices: Vec<usize>, possibilites_for_tiles: &mut Vec<Vec<Vec<usize>>>)
{
    for x in 0..output_edge_length
    {
        let mut possibilites_per_tile: Vec<Vec<usize>> = Vec::new();

        for y in 0..output_edge_length
        {
            possibilites_per_tile.push(all_pattern_indices.clone());
        }

        possibilites_for_tiles.push(possibilites_per_tile);
    }
}

fn is_finished(possibilites_for_tiles: &Vec<Vec<Vec<usize>>>) -> bool
{
    for row in possibilites_for_tiles
    {
        for possibilities_for_one_tile in row
        {
            if possibilities_for_one_tile.len() > 1
            {
                return false;
            }
        }
    }

    true
}

fn get_shannon_entropy_for_tile(
    x: usize,
    y: usize,
    possibilites_for_tiles: &Vec<Vec<Vec<usize>>>,
    patterns: &Vec<Pattern>,
    random_number_generator: &mut StdRng) -> f32
{
    if possibilites_for_tiles[x][y].len() == 1
    {
        return 0f32;
    }

    let shanon_entropy_without_noise = possibilites_for_tiles[x][y]
        .iter()
        .map(|pattern_index| patterns[*pattern_index].probability)
        .map(|probability| - probability * probability.log2())
        .sum::<f32>();

    shanon_entropy_without_noise - random_number_generator.gen_range(0f32..0.1) as f32
}



pub fn create_map(
    train_data: MultiVec<i32>,
    output_edge_length: usize,
    pattern_edge_length: usize,
    seed: u64)
{
    let random_number_generator = StdRng::seed_from_u64(seed);

    let patterns = &slice_into_patterns(train_data, pattern_edge_length);
    let pattern_indices = (0..patterns.len()).collect();
    let rules_checker = &mut RulesChecker::new(patterns);
    let possibilites_for_tiles: &mut Vec<Vec<Vec<usize>>> = &mut Vec::new();

    train_rules(patterns, pattern_edge_length, rules_checker);

    initialize_possibilities_for_tiles(
        output_edge_length,
        pattern_indices,
        possibilites_for_tiles);
}