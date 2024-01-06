use std::{collections::HashMap, ops::Index};

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

    pub fn check_if_pattern_is_allowed(&self, current_pattern_index: usize, direction: Direction, next_pattern_index: usize) -> bool
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
                    let current_tile = train_data.get(x, y);

                    match current_tile {
                        Some(value) =>
                        {
                            if *value == -1
                            {
                                continue;
                            }

                            pattern.flat_definition.push(*value)
                        },
                        None => panic!("Fehler bei Slice"),
                    };
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

fn get_valid_directions(x: i32, y: i32, output_edge_length: usize) -> Vec<Direction>
{
    match (x, y)
    {
        (0, 0) => vec! [ Direction::Right, Direction::Down, Direction::DownRight ],
        (0, y) if y == output_edge_length as i32 - 1 => vec! [ Direction::Right, Direction::Up, Direction::UpRight ],
        (0, _) => vec! [ Direction::Right, Direction::Down, Direction::DownRight, Direction::Up, Direction::UpRight ],
        (x, 0) if x == output_edge_length as i32 -1 => vec! [ Direction::Left, Direction::Down, Direction::DownLeft ],
        (x, y) if x == output_edge_length as i32 -1 && y == output_edge_length as i32 -1 => vec! [ Direction::Left, Direction::Down, Direction::DownLeft, Direction::Up, Direction::UpLeft ],
        (x, _) if x == output_edge_length as i32 -1 => vec! [ Direction::Left, Direction::Down, Direction::DownLeft, Direction::Up, Direction::UpLeft ],
        (_, 0) => vec! [ Direction::Left, Direction::Right, Direction::Down, Direction::DownLeft, Direction::DownRight ],
        (_, y) if y == output_edge_length as i32 -1 => vec! [ Direction::Left, Direction::Right, Direction::Up, Direction::UpLeft, Direction::UpRight ],
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


fn get_tile_position_with_minimal_entropy(
    possibilites_for_tiles: &Vec<Vec<Vec<usize>>>,
    output_edge_length: usize,
    patterns: &Vec<Pattern>,
    random_number_generator: &mut StdRng) -> Option<(usize, usize)>
{
    let mut min_entropy = f32::MAX;
    let mut min_entropy_position = None;

    for x in 0..output_edge_length
    {
        for y in 0..output_edge_length
        {
            let entropy = get_shannon_entropy_for_tile(x, y, possibilites_for_tiles, patterns, random_number_generator);

            if entropy.abs() <= f32::EPSILON
            {
                continue;
            }

            if entropy < min_entropy
            {
                min_entropy = entropy;
                min_entropy_position = Some((x, y));
            }
        }
    }

    min_entropy_position
}

fn choose_one_possibility(
    possibilites_for_tiles: &mut Vec<Vec<Vec<usize>>>,
    output_edge_length: usize,
    patterns: &Vec<Pattern>,
    random_number_generator: &mut StdRng) -> Option<(usize, usize)>
{
    let (tile_x, tile_y) = get_tile_position_with_minimal_entropy(possibilites_for_tiles, output_edge_length, patterns, random_number_generator)?;

    let possible_pattern_indices = &possibilites_for_tiles[tile_x][tile_y];

    let highest_probability = possible_pattern_indices
        .iter()
        .map(|pattern_index| patterns[*pattern_index].probability)
        .reduce(f32::max)?;

    let possibilities_with_highest_probability: Vec<&usize> = possible_pattern_indices
        .iter()
        .filter(|pattern_index| (patterns[**pattern_index].probability - highest_probability).abs() <= f32::EPSILON)
        .collect();

    let chosen_possibility_index = random_number_generator.gen_range(0..possibilities_with_highest_probability.len());

    let possibilities_for_tile = possibilites_for_tiles
        .get_mut(tile_x)
        .expect("Unmöglicher Index")
        .get_mut(tile_y)
        .expect("Unmöglicher Index");

    possibilities_for_tile.clear();
    possibilities_for_tile.push(chosen_possibility_index);

    Some((tile_x, tile_y))
}

fn propagate_chosen_possibility(
    x_chosen_tile: usize,
    y_chosen_tile: usize,
    output_edge_length: usize,
    possibilites_for_tiles: &mut Vec<Vec<Vec<usize>>>,
    rules_checker: &RulesChecker) -> Option<()>
{
    let mut missing_propagations = Vec::<(usize, usize)>::new();

    loop {
        let (x_current_tile, y_current_tile)  = missing_propagations.pop()?;

        for direction in get_valid_directions(x_current_tile as i32, y_current_tile as i32, output_edge_length)
        {
            let direction_as_tuple: (i32, i32) = direction.into();

            let (next_x, next_y) = ((x_current_tile as i32 + direction_as_tuple.0) as usize, (y_current_tile as i32 + direction_as_tuple.1) as usize);

            let mut possible_next_patterns = possibilites_for_tiles
                .get_mut(next_x)
                .expect("next_x is not in possible tiles!")
                .get_mut(next_y)
                .expect("next_y is not in possible tiles!");

            let mut updated_possible_next_patterns: Vec<usize> = possible_next_patterns
                .iter()
                .filter(|possible_next_pattern| possible_next_patterns.iter().any(|afterwards_possible_next_pattern| rules_checker.check_if_pattern_is_allowed(
                    **possible_next_pattern,
                    direction,
                    *afterwards_possible_next_pattern)))
                .map(|pattern_index| *pattern_index)
                .collect();

            possible_next_patterns.clear();

            possible_next_patterns.append(&mut updated_possible_next_patterns);
        }
    }
}

fn create_output_tiles(
    possibilites_for_tiles: &Vec<Vec<Vec<usize>>>,
    patterns: &Vec<Pattern>,
    output_edge_length: usize,
    output_edge_length_with_space_for_patterns: usize,
    pattern_edge_length: usize) -> MultiVec<i32>
{
    let mut output_tiles = MultiVec::new(-1, output_edge_length, output_edge_length);

    for x in 0..output_edge_length_with_space_for_patterns
    {
        for y in 0..output_edge_length_with_space_for_patterns
        {
            let chosen_pattern_index = possibilites_for_tiles[x][y].first().expect("No possibility left");

            let chosen_pattern = &patterns[*chosen_pattern_index];

            for flat_pattern_index in 0..chosen_pattern.flat_definition.len()
            {
                let output_tile = output_tiles
                    .get_mut(x + flat_pattern_index % pattern_edge_length, y + flat_pattern_index / pattern_edge_length)
                    .expect("Out of bound in output tiles");

                *output_tile = chosen_pattern.flat_definition[flat_pattern_index];
            }
        }
    }

    output_tiles
}

pub fn create_map(
    train_data: MultiVec<i32>,
    output_edge_length: usize,
    pattern_edge_length: usize,
    seed: u64) -> MultiVec<i32>
{
    let mut random_number_generator = StdRng::seed_from_u64(seed);

    let patterns = &slice_into_patterns(train_data, pattern_edge_length);
    let pattern_indices = (0..patterns.len()).collect();
    let rules_checker = &mut RulesChecker::new(patterns);
    let possibilites_for_tiles: &mut Vec<Vec<Vec<usize>>> = &mut Vec::new();

    let output_edge_length_with_space_for_patterns = output_edge_length - pattern_edge_length * pattern_edge_length;

    train_rules(patterns, pattern_edge_length, rules_checker);

    initialize_possibilities_for_tiles(
        output_edge_length_with_space_for_patterns,
        pattern_indices,
        possibilites_for_tiles);

    while !is_finished(possibilites_for_tiles)
    {
        let chosen_possibility = choose_one_possibility(
            possibilites_for_tiles,
            output_edge_length_with_space_for_patterns,
            patterns,
            &mut random_number_generator);

        match chosen_possibility {
            Some((x_chosen_tile, y_chosen_tile)) =>
            {
                propagate_chosen_possibility(
                    x_chosen_tile,
                    y_chosen_tile,
                    output_edge_length_with_space_for_patterns,
                    possibilites_for_tiles,
                    rules_checker);
            },
            None => {
                return create_output_tiles(possibilites_for_tiles, patterns, output_edge_length, output_edge_length_with_space_for_patterns, pattern_edge_length);
            },
        }
    }

    return create_output_tiles(possibilites_for_tiles, patterns, output_edge_length, output_edge_length_with_space_for_patterns, pattern_edge_length);
}