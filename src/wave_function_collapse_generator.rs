use std::collections::{HashSet, VecDeque};

use bevy::log::info;
use enum_iterator::{Sequence, all};
use rand::{rngs::StdRng, SeedableRng, Rng};
use bit_set::BitSet;

use crate::multi_vec::*;

// all::<Direction>()
#[derive(Sequence, PartialEq, Eq, Hash, Clone, Copy, Debug)]
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

#[derive(Debug, Clone)]
struct Possibilities
{
    bits: BitSet,
}

impl Possibilities
{
    fn new(num_possibilities: usize) -> Self
    {
        let mut bits = BitSet::new();
        for i in 0..num_possibilities { bits.insert(i); }
        Self{ bits }
    }

    fn len(&self) -> usize
    {
        self.bits.len()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = usize> + 'a
    {
        self.bits.iter()
    }
    fn insert(&mut self, index: usize)
    {
        self.bits.insert(index);
    }
    fn remove(&mut self, index: usize)
    {
        self.bits.remove(index);
    }
    fn clear(&mut self)
    {
        self.bits.clear();
    }

    fn contains(&self, index: usize) -> bool
    {
        self.bits.contains(index)
    }
}

struct RulesChecker
{
    rules: HashSet<(usize, Direction, usize)>,
}

impl RulesChecker
{
    pub fn new() -> Self
    {
        Self
        {
            rules: HashSet::new(),
        }
    }

    pub fn add_rule(&mut self, current_pattern_index: usize, direction: Direction, next_pattern_index: usize)
    {
        self.rules.insert((current_pattern_index, direction, next_pattern_index));
    }

    pub fn check_if_pattern_is_allowed(&self, current_pattern_index: usize, direction: Direction, next_pattern_index: usize) -> bool
    {
        self.rules.contains(&(current_pattern_index, direction, next_pattern_index))
    }
}


fn slice_into_patterns(
    train_data: MultiVec<i32>,
    pattern_size: usize) -> Vec<Pattern>
{
    let mut patterns = Vec::new();

    for y_start in 0..(train_data.h - pattern_size)
    {
        'tile_x_loop: for x_start in 0..(train_data.w - pattern_size)
        {
            let mut pattern = Pattern
            {
                occurrences: 1,
                flat_definition: Vec::new(),
                probability: 0f32,
            };

            for y in y_start..(y_start + pattern_size)
            {
                for x in x_start..(x_start + pattern_size)
                {
                    let current_tile = train_data.get(x, y);

                    let tile_value = *current_tile.expect("Error on slicing");

                    if tile_value == -1
                    {
                        continue 'tile_x_loop;
                    }

                    pattern.flat_definition.push(tile_value);
                }
            }

            match patterns.iter().position(|old_pattern: &Pattern| old_pattern.flat_definition == pattern.flat_definition) {
                Some(index) => patterns[index].occurrences += 1,
                None => patterns.push(pattern),
            }
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

    info!("sliced into {} patterns.", patterns.len());

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
        (x, y) if x == output_edge_length as i32 -1 && y == output_edge_length as i32 -1 => vec! [ Direction::Left, Direction::Up, Direction::UpLeft],
        (x, _) if x == output_edge_length as i32 -1 => vec! [ Direction::Left, Direction::Down, Direction::DownLeft, Direction::Up, Direction::UpLeft ],
        (_, 0) => vec! [ Direction::Left, Direction::Right, Direction::Down, Direction::DownLeft, Direction::DownRight ],
        (_, y) if y == output_edge_length as i32 -1 => vec! [ Direction::Left, Direction::Right, Direction::Up, Direction::UpLeft, Direction::UpRight ],
        (_, _) => vec! [ Direction::Left, Direction::Up, Direction::Right, Direction::UpLeft, Direction::UpRight, Direction::Down, Direction::DownLeft, Direction::DownRight ],
    }
}

fn get_relevant_tiles_for_checking_overlapping_patterns(
    pattern: &Pattern,
    direction_for_checking_overlapping: Direction,
    pattern_edge_length: usize) -> Vec<i32>
{
    if pattern_edge_length != 2 { panic!("pattern_edge_length needs to be 2!"); }

    match direction_for_checking_overlapping {
        Direction::None => pattern.flat_definition.clone(),
        Direction::UpLeft => vec! [ pattern.flat_definition[3] ],
        Direction::Up => pattern.flat_definition[2..=3].to_vec(),
        Direction::UpRight => vec! [ pattern.flat_definition[2] ],
        Direction::Left => vec! [ pattern.flat_definition[1], pattern.flat_definition[3] ],
        Direction::Right => vec! [ pattern.flat_definition[0], pattern.flat_definition[2] ],
        Direction::DownLeft => vec! [ pattern.flat_definition[1] ],
        Direction::Down => pattern.flat_definition[0..=1].to_vec(),
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
                    rules_checker.add_rule(current_pattern_for_rule_extraction_index, direction, next_pattern_for_rule_extraction_index);
                }
            }
        }
    }
}

fn initialize_possibilities_for_tiles(output_edge_length: usize, patterns: &Vec<Pattern>, possibilites_for_tiles: &mut MultiVec<Possibilities>)
{
    for x in 0..output_edge_length
    {
        for y in 0..output_edge_length
        {
            *possibilites_for_tiles.get_mut(x, y).unwrap() = Possibilities::new(patterns.len());
        }
    }
}

fn is_finished(possibilites_for_tiles: &MultiVec<Possibilities>) -> bool
{
    for y in 0..possibilites_for_tiles.h
    {
        for x in 0..possibilites_for_tiles.w
        {
            if possibilites_for_tiles.get(x,y).unwrap().len() > 1
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
    possibilites_for_tile: &Possibilities,
    patterns: &Vec<Pattern>,
    random_number_generator: &mut StdRng) -> f32
{
    if possibilites_for_tile.len() == 1
    {
        return 0f32;
    }

    let shanon_entropy_without_noise = possibilites_for_tile
        .iter()
        .map(|pattern_index| patterns[pattern_index].probability)
        .map(|probability| - probability * probability.log2())
        .sum::<f32>();

    shanon_entropy_without_noise - random_number_generator.gen_range(0f32..0.1) as f32
}


fn get_tile_position_with_minimal_entropy(
    possibilites_for_tiles: &MultiVec<Possibilities>,
    output_edge_length: usize,
    patterns: &Vec<Pattern>,
    random_number_generator: &mut StdRng,
    entropy_for_tile: &MultiVec<f32>
) -> Option<(usize, usize)>
{
    let mut min_entropy = f32::MAX;
    let mut min_entropy_position = None;

    for x in 0..output_edge_length
    {
        for y in 0..output_edge_length
        {
            let entropy = *entropy_for_tile.get(x, y).unwrap();

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

fn collapse_one_possibility(
    possibilites_for_tiles: &mut MultiVec<Possibilities>,
    output_edge_length: usize,
    patterns: &Vec<Pattern>,
    random_number_generator: &mut StdRng,
    entropy_for_tile: &mut MultiVec<f32>
) -> Option<(usize, usize)>
{
    let (tile_x, tile_y) = get_tile_position_with_minimal_entropy(possibilites_for_tiles, output_edge_length, patterns, random_number_generator, &entropy_for_tile)?;
    
    let possible_pattern_indices = possibilites_for_tiles.get(tile_x, tile_y)?;
    println!("We want to collapse {} {} to one possibility of {:?}", tile_x, tile_y, possible_pattern_indices);

    // let highest_probability = possible_pattern_indices
    //     .iter()
    //     .map(|pattern_index| patterns[*pattern_index].probability)
    //     .reduce(f32::max)?;
    // let possibilities_with_highest_probability: Vec<&usize> = possible_pattern_indices
    //     .iter()
    //     .filter(|pattern_index| (patterns[**pattern_index].probability - highest_probability).abs() <= f32::EPSILON)
    //     .collect();
    // let chosen_possibility_index = random_number_generator.gen_range(0..possibilities_with_highest_probability.len());

    let distribution = possible_pattern_indices
        .iter()
        .map(|pattern_index| &patterns[pattern_index])
        .scan(0.0f32, |acc, pattern| { *acc += pattern.probability; Some(*acc) })
        .collect::<Vec<f32>>();
    
    let sample = random_number_generator.gen_range(0.0..(*distribution.last()?));
    let chosen_possibility_index = distribution.iter().enumerate().find(|(_, prefix_sum)| sample < **prefix_sum)?.0;

    let index_vec: Vec<usize> = possible_pattern_indices.iter().collect();
    let chosen_pattern_index = index_vec[chosen_possibility_index];

    println!("We have chosen {} {} to be {:?}", tile_x, tile_y, chosen_possibility_index);

    // collapse wave function
    let possibilities_for_tile = possibilites_for_tiles
        .get_mut(tile_x, tile_y).expect("Unmöglicher Index");
    possibilities_for_tile.clear();
    possibilities_for_tile.insert(chosen_pattern_index);

    *entropy_for_tile.get_mut(tile_x, tile_y).unwrap() = 0f32;

    Some((tile_x, tile_y))
}

fn plot_possibilities(poss: &MultiVec<Possibilities>) {
    for y in 0..poss.h
    {
        for x in 0..poss.w
        {
            print!("{:3} ", poss.get(x, y).unwrap().len());
        }
        println!();
    }
}

fn propagate_chosen_possibility(
    x_chosen_tile: usize,
    y_chosen_tile: usize,
    output_edge_length: usize,
    possibilites_for_tiles: &mut MultiVec<Possibilities>,
    rules_checker: &RulesChecker,
    entropy_per_tile: &mut MultiVec<f32>,
    patterns: &Vec<Pattern>,
    random_number_generator: &mut StdRng,
    work_queue: &mut VecDeque<(usize, usize)>,
) {
    println!("we are starting propagation. Behold the mysteries of the universe!");
    plot_possibilities(possibilites_for_tiles);

    let mut already_handled = HashSet::<(usize,usize)>::new();
    
    work_queue.push_back((x_chosen_tile, y_chosen_tile));
    already_handled.insert((x_chosen_tile, y_chosen_tile));

    let mut recompute_entropy_set = HashSet::<(usize, usize)>::new();

    while let Some((x_current_tile, y_current_tile)) = work_queue.pop_front() 
    {
        let (possible_current_patterns, mut rest_list) = possibilites_for_tiles.isolate(x_current_tile, y_current_tile).unwrap();

        // println!("Filter neighbors of {} {}", x_current_tile, y_current_tile);

        for direction in get_valid_directions(x_current_tile as i32, y_current_tile as i32, output_edge_length)
        {
            let direction_as_tuple: (i32, i32) = direction.into();

            let (next_x, next_y) = ((x_current_tile as i32 + direction_as_tuple.0) as usize, (y_current_tile as i32 + direction_as_tuple.1) as usize);
            let possible_next_patterns = rest_list.get_mut(next_x, next_y).expect("next_xy is not in possible tiles!");

            let before_len = possible_next_patterns.len();

            let mut new_next_patterns = possible_next_patterns.clone();
            for pattern_index in possible_next_patterns.iter()
            {
                let is_valid = possible_current_patterns
                    .iter()
                    .any(|current_pattern_to_check| rules_checker.check_if_pattern_is_allowed(
                        current_pattern_to_check,
                        direction,
                        pattern_index));
                
                if !is_valid
                {
                    new_next_patterns.remove(pattern_index);
                }

            }
            *possible_next_patterns = new_next_patterns;

            let after_len = possible_next_patterns.len();

            if after_len == 0
            {
                panic!("No possible pattern left for {} {} in direction {:?}", next_x, next_y, direction);
            }

            let has_changed = after_len != before_len;

            // println!("We have filtered {} {}, new possibilities", next_x, next_y);
            // plot_possibilities(possibilites_for_tiles);

            if has_changed && !already_handled.contains(&(next_x, next_y))
            {
                recompute_entropy_set.insert((next_x, next_y));

                work_queue.push_back((next_x, next_y));
                already_handled.insert((next_x, next_y));
            }
        }
    }

    for (x, y) in recompute_entropy_set
    {
        *entropy_per_tile.get_mut(x, y).unwrap() = get_shannon_entropy_for_tile(
            x,
            y,
            possibilites_for_tiles.get(x, y).unwrap(),
            &patterns,
            random_number_generator,
        );
    }
}

fn create_output_tiles(
    possibilities_for_tiles: &MultiVec<Possibilities>,
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
            let chosen_pattern_index = possibilities_for_tiles.get(x, y).unwrap()
                .iter()
                .next()
                .expect("No possibility left");

            println!();
            print!("{:2}", chosen_pattern_index);

            let chosen_pattern = &patterns[chosen_pattern_index];

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
    // let mut random_number_generator = StdRng::seed_from_u64(seed);
    let mut random_number_generator = StdRng::from_entropy();

    info!("Slice into patterns...");
    let patterns = &slice_into_patterns(train_data, pattern_edge_length);

    info!("train rules with top secret ultra complex algorithm");
    let rules_checker = &mut RulesChecker::new();
    train_rules(patterns, pattern_edge_length, rules_checker);
    
    // Print all Rules (very big output)
    // for direction in all::<Direction>()
    // {
    //     info!("Rules for {:?}:", direction);
    //     for (current_pattern_index, possible_patterns) in rules_checker.rules.iter()
    //     {
    //         let src_pattern = &patterns[*current_pattern_index];
    //         for next_pattern_index in possible_patterns.get(&direction).unwrap()
    //         {
    //             let dst_pattern = &patterns[*next_pattern_index];
    //             println!("{:2} {:2} -> {:2} {:2} direction {:?}\n{:2} {:2} -> {:2} {:2}\n",
    //                 src_pattern.flat_definition[0], src_pattern.flat_definition[1], dst_pattern.flat_definition[0], dst_pattern.flat_definition[1],
    //                 direction,
    //                 src_pattern.flat_definition[2], src_pattern.flat_definition[3], dst_pattern.flat_definition[2], dst_pattern.flat_definition[3]);
    //         }
    //     }
    // }

    info!("init possibilites for each pattern position");
    let output_edge_length_with_space_for_patterns = output_edge_length - pattern_edge_length;
    // let possibilites_for_tiles: &mut Vec<Vec<Vec<usize>>> = &mut Vec::new();
    let possibilities_for_tiles: &mut MultiVec<Possibilities> = &mut MultiVec::new(
        Possibilities::new(0),
        output_edge_length_with_space_for_patterns, 
        output_edge_length_with_space_for_patterns);
    initialize_possibilities_for_tiles(
        output_edge_length_with_space_for_patterns,
        &patterns,
        possibilities_for_tiles);

    // init entropy cache data structure
    let mut entropy_per_tile = MultiVec::<f32>::new(1e9, output_edge_length_with_space_for_patterns, output_edge_length_with_space_for_patterns);
    for y in 0..output_edge_length_with_space_for_patterns
    {
        for x in 0..output_edge_length_with_space_for_patterns
        {
            *entropy_per_tile.get_mut(x, y).unwrap() = get_shannon_entropy_for_tile(
                x,
                y,
                possibilities_for_tiles.get(x,y).unwrap(),
                patterns,
                &mut random_number_generator
            );
        }
    }

    let mut work_queue = VecDeque::<(usize, usize)>::new();
    
    while !is_finished(possibilities_for_tiles)
    {
        info!("collapse single pattern position...");
        let chosen_possibility = collapse_one_possibility(
            possibilities_for_tiles,
            output_edge_length_with_space_for_patterns,
            patterns,
            &mut random_number_generator,
            &mut entropy_per_tile,
        );
        
        match chosen_possibility {
            Some((x_chosen_tile, y_chosen_tile)) =>
            {
                info!("Pattern at {x_chosen_tile},{y_chosen_tile} was collapsed");
                info!("propagate possibilites...");
                work_queue.clear();
                propagate_chosen_possibility(
                    x_chosen_tile,
                    y_chosen_tile,
                    output_edge_length_with_space_for_patterns,
                    possibilities_for_tiles,
                    rules_checker,
                    &mut entropy_per_tile,
                    patterns,
                    &mut random_number_generator,
                    &mut work_queue,
                );
            },
            None =>
            {
                info!("create output tiles...");
                return create_output_tiles(possibilities_for_tiles, patterns, output_edge_length, output_edge_length_with_space_for_patterns, pattern_edge_length);
            },
        }
    }

    info!("create output tiles...");
    return create_output_tiles(possibilities_for_tiles, patterns, output_edge_length, output_edge_length_with_space_for_patterns, pattern_edge_length);
}