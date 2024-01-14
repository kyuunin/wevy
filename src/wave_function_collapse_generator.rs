use std::collections::{HashSet, VecDeque, HashMap};
use bevy::prelude::*;
use enum_iterator::{Sequence, all};
use rand::{rngs::StdRng, SeedableRng, Rng};
use bit_set::BitSet;

use crate::multi_vec::*;

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
    flat_definition: [i32; 4],
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
    rules: HashMap<(usize, Direction), BitSet>,
}

impl RulesChecker
{
    pub fn new() -> Self
    {
        Self
        {
            rules: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, current_pattern_index: usize, direction: Direction, next_pattern_index: usize)
    {
        // if we insert one direction, insert all directions that are not inserted yet
        if !self.rules.contains_key(&(current_pattern_index, direction))
        {
            for direction in all::<Direction>()
            {
                if !self.rules.contains_key(&(current_pattern_index, direction))
                {
                    self.rules.insert((current_pattern_index, direction), BitSet::new());
                }
            }
        }

        self.rules.get_mut(&(current_pattern_index, direction)).unwrap().insert(next_pattern_index);
    }

    pub fn get_possible_patterns(&self, current_pattern_index: usize, direction: Direction) -> &BitSet
    {
        self.rules.get(&(current_pattern_index, direction)).expect("No rule for this pattern and direction!")
    }
}

fn slice_into_patterns(train_data: MultiVec<i32>, pattern_size: usize) -> Vec<Pattern> {
    let mut patterns = Vec::new();
    let mut occurrences = HashMap::<usize, i32>::new();

    for y in 0..(train_data.h - pattern_size)
    {
        for x in 0..(train_data.w - pattern_size)
        {
            let pattern = Pattern { flat_definition: [
                *train_data.get(x, y).expect("Error on slicing"),
                *train_data.get(x + 1, y).expect("Error on slicing"),
                *train_data.get(x, y + 1).expect("Error on slicing"),
                *train_data.get(x + 1, y + 1).expect("Error on slicing"),
            ], probability: 0f32 };

            if pattern.flat_definition.iter().any(|x| *x == -1) { continue; }

            match patterns.iter().position(|old_pattern: &Pattern| old_pattern.flat_definition == pattern.flat_definition) {
                None => {
                    patterns.push(pattern);
                    occurrences.insert(patterns.len() - 1, 1);
                } Some(index) => {
                    *occurrences.get_mut(&index).unwrap() += 1;
                }
            }
        }
    }

    let sum_occurrences: i32 = occurrences.values().sum();
    for (i, pattern) in patterns.iter_mut().enumerate()
    {
        pattern.probability = occurrences[&i] as f32 / sum_occurrences as f32;
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
        Direction::None => pattern.flat_definition.into(),
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

fn plot_possibilities(poss: &MultiVec<Possibilities>) {
    // cursor to top left => stable terminal animation
    let esc = 27 as char;
    println!("{}[H", esc);

    for y in 0..poss.h
    {
        for x in 0..poss.w
        {
            print!("{:3} ", poss.get(x, y).unwrap().len());
        }
        println!();
    }
}

struct PropagateCaches
{
    work_queue: VecDeque<(usize, usize)>,
    already_handled: HashSet<(usize, usize)>,
    has_valid_current_pattern: BitSet,
}

#[derive(Clone, Copy)]
struct EdgeLength {
    output: usize,
    pattern: usize,
}

impl EdgeLength {
    fn output_with_space_for_patterns(&self) -> usize { self.output - self.pattern }
}

struct CollapsedTiles {
    queue: VecDeque<(usize, usize, i32)>, // (x, y, tile_id)
}

impl CollapsedTiles {
    fn insert_pattern(&mut self, pattern: &Pattern, pos: (usize, usize), edge_length: EdgeLength)
    {
        let data = &pattern.flat_definition;

        let output_x2 = pos.0 == edge_length.output_with_space_for_patterns() - 1;
        let output_y2 = pos.1 == edge_length.output_with_space_for_patterns() - 1;

        self.queue.push_back((pos.0, pos.1, data[0]));
        if output_x2 { self.queue.push_back((pos.0 + 1, pos.1, data[1])); }
        if output_y2 { self.queue.push_back((pos.0, pos.1 + 1, data[2])); }
        if output_x2 && output_y2 {
            self.queue.push_back((pos.0 + 1, pos.1 + 1, data[3]));
        }
    }

    fn clear(&mut self) { self.queue.clear(); }
}

pub struct WaveFunctionCollapseGenerator
{
    edge_length: EdgeLength,
    patterns: Vec<Pattern>,
    rules_checker: RulesChecker,
    possibilities_for_tiles: MultiVec<Possibilities>,
    entropy_for_tile: MultiVec<f32>,
    random_number_generator: StdRng,
    propagate_caches: PropagateCaches, // keep this around to avoid allocations
    collapsed_tiles: CollapsedTiles, // output queue for iterator interface
}

impl WaveFunctionCollapseGenerator
{
    fn train_rules(&mut self)
    {
        for (current_pattern_index, current_pattern) in self.patterns.iter().enumerate()
        {
            for (next_pattern_index, next_pattern) in self.patterns.iter().enumerate()
            {
                for direction in all::<Direction>()
                {
                    let direction_as_tuple:(i32, i32) = direction.into();
                    let opposite_direction_as_tuple = (direction_as_tuple.0 * -1, direction_as_tuple.1 * -1);
                    let opposite_direction: Direction = opposite_direction_as_tuple.try_into().expect("Error when getting opposite direction");
                    
                    let overlapping_tiles_next_pattern = get_relevant_tiles_for_checking_overlapping_patterns(next_pattern, direction, self.edge_length.pattern);
                    let overlapping_tiles_current_pattern = get_relevant_tiles_for_checking_overlapping_patterns(current_pattern, opposite_direction, self.edge_length.pattern);
                    
                    if overlapping_tiles_next_pattern == overlapping_tiles_current_pattern
                    {
                        self.rules_checker.add_rule(current_pattern_index, direction, next_pattern_index);
                    }
                }
            }
        }
    }

    fn is_finished(&self) -> bool
    {
        self.possibilities_for_tiles.iter().all(|possibilities_for_tile| possibilities_for_tile.len() <= 1)
    }

    fn get_shannon_entropy_for_tile(&mut self, x: usize, y: usize) -> f32
    {
        let possibilites_for_tile = self.possibilities_for_tiles.get(x, y).unwrap();
        if possibilites_for_tile.len() == 1
        {
            return 0f32;
        }
    
        let shanon_entropy_without_noise = possibilites_for_tile
            .iter()
            .map(|pattern_index| self.patterns[pattern_index].probability)
            .map(|probability| - probability * probability.log2())
            .sum::<f32>();
    
        shanon_entropy_without_noise - self.random_number_generator.gen_range(0f32..0.1) as f32
    }

    fn get_tile_position_with_minimal_entropy(&self) -> Option<(usize, usize)>
    {
        let i = self.entropy_for_tile
            .iter()
            .enumerate()
            .filter(|(_, entropy)| entropy.abs() > f32::EPSILON)
            .min_by(|(_, entropy_a), (_, entropy_b)| entropy_a.partial_cmp(entropy_b).unwrap())?
            .0;
        self.entropy_for_tile.index_to_xy(i)
    }

    fn collapse_one_possibility(&mut self) -> Option<(usize, usize)>
    {
        let (tile_x, tile_y) = self.get_tile_position_with_minimal_entropy()?;
        
        let possible_pattern_indices = self.possibilities_for_tiles.get(tile_x, tile_y)?;
        debug!("We want to collapse {} {} to one possibility of {:?}", tile_x, tile_y, possible_pattern_indices);
    
        let distribution = possible_pattern_indices
            .iter()
            .map(|pattern_index| &self.patterns[pattern_index])
            .scan(0.0f32, |acc, pattern| { *acc += pattern.probability; Some(*acc) })
            .collect::<Vec<f32>>();
        
        let sample = self.random_number_generator.gen_range(0.0..(*distribution.last()?));
        let chosen_possibility_index = distribution.iter().enumerate().find(|(_, prefix_sum)| sample < **prefix_sum)?.0;
    
        let index_vec: Vec<usize> = possible_pattern_indices.iter().collect();
        let chosen_pattern_index = index_vec[chosen_possibility_index];
    
        debug!("We have chosen {} {} to be pattern {:?}", tile_x, tile_y, chosen_pattern_index);
    
        // collapse wave function
        let possibilities_for_tile = self.possibilities_for_tiles.get_mut(tile_x, tile_y).expect("Unmöglicher Index");
        possibilities_for_tile.clear();
        possibilities_for_tile.insert(chosen_pattern_index);
    
        *self.entropy_for_tile.get_mut(tile_x, tile_y).unwrap() = 0f32;

        self.collapsed_tiles.insert_pattern(&self.patterns[chosen_pattern_index], (tile_x, tile_y), self.edge_length);
    
        Some((tile_x, tile_y))
    }

    fn propagate_chosen_possibility(&mut self, x_chosen_tile: usize, y_chosen_tile: usize)
    {
        let work_queue = &mut self.propagate_caches.work_queue;
        let already_handled = &mut self.propagate_caches.already_handled;
        work_queue.clear();
        already_handled.clear();

        debug!("we are starting propagation. Behold the mysteries of the universe!");
        #[cfg(debug_assertions)]
        plot_possibilities(&self.possibilities_for_tiles);
    
        work_queue.push_back((x_chosen_tile, y_chosen_tile));
        already_handled.insert((x_chosen_tile, y_chosen_tile));
    
        let mut recompute_entropy_set = HashSet::<(usize, usize)>::new();
    
        while let Some((x_current_tile, y_current_tile)) = work_queue.pop_front() 
        {
            let (possible_current_patterns, mut rest_list) = self.possibilities_for_tiles.isolate(x_current_tile, y_current_tile).unwrap();
    
            // println!("Filter neighbors of {} {}", x_current_tile, y_current_tile);
    
            for direction in get_valid_directions(x_current_tile as i32, y_current_tile as i32, self.edge_length.output_with_space_for_patterns())
            {
                let direction_as_tuple: (i32, i32) = direction.into();
    
                let (next_x, next_y) = ((x_current_tile as i32 + direction_as_tuple.0) as usize, (y_current_tile as i32 + direction_as_tuple.1) as usize);
                let possible_next_patterns = rest_list.get_mut(next_x, next_y).expect("next_xy is not in possible tiles!");
    
                let before_len = possible_next_patterns.len();
    
                let has_valid_current_pattern = &mut self.propagate_caches.has_valid_current_pattern;
                has_valid_current_pattern.clear();
                for current_pattern_index in possible_current_patterns.iter()
                {
                    has_valid_current_pattern.union_with(self.rules_checker.get_possible_patterns(current_pattern_index, direction));
                }
                possible_next_patterns.bits.intersect_with(&has_valid_current_pattern);
    
                let after_len = possible_next_patterns.len();
    
                if after_len == 0
                {
                    panic!("No possible pattern left for {} {} in direction {:?}", next_x, next_y, direction);
                }

                let has_changed = after_len != before_len;
    
                // println!("We have filtered {} {}, new possibilities", next_x, next_y);
                // plot_possibilities(possibilites_for_tiles);

                if has_changed && after_len == 1
                {
                    let pattern_index = possible_next_patterns.iter().next().unwrap();
                    let pattern = &self.patterns[pattern_index];
                    self.collapsed_tiles.insert_pattern(pattern, (next_x, next_y), self.edge_length);
                }
    
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
            *self.entropy_for_tile.get_mut(x, y).unwrap() = self.get_shannon_entropy_for_tile(x, y);
        }
    }

    fn create_output_tiles(&self) -> MultiVec<i32>
    {
        let mut output_tiles = MultiVec::new(-1, self.edge_length.output, self.edge_length.output);
    
        for y in 0..self.possibilities_for_tiles.h
        {
            for x in 0..self.possibilities_for_tiles.w
            {
                let chosen_pattern_index = self.possibilities_for_tiles.get(x, y).unwrap()
                    .iter()
                    .next()
                    .expect("No possibility left");
    
                let chosen_pattern = &self.patterns[chosen_pattern_index];
    
                for flat_pattern_index in 0..chosen_pattern.flat_definition.len()
                {
                    let output_tile = output_tiles
                        .get_mut(x + flat_pattern_index % self.edge_length.pattern, y + flat_pattern_index / self.edge_length.pattern)
                        .expect("Out of bound in output tiles");
    
                    *output_tile = chosen_pattern.flat_definition[flat_pattern_index];
                }
            }
        }
    
        output_tiles
    }

    pub fn new(
        train_data: MultiVec<i32>,
        output_edge_length: usize,
        pattern_edge_length: usize,
        seed: u64) -> WaveFunctionCollapseGenerator
    {
        info!("Slice into patterns...");
        let patterns = slice_into_patterns(train_data, pattern_edge_length);

        let mut generator = WaveFunctionCollapseGenerator {
            edge_length: EdgeLength {
                output: output_edge_length,
                pattern: pattern_edge_length,
            },
            patterns,
            rules_checker: RulesChecker::new(),
            possibilities_for_tiles: MultiVec::new(Possibilities::new(0), 0, 0),
            entropy_for_tile: MultiVec::new(0f32, 0, 0),
            random_number_generator: StdRng::seed_from_u64(seed),
            propagate_caches: PropagateCaches {
                work_queue: VecDeque::new(),
                already_handled: HashSet::new(),
                has_valid_current_pattern: BitSet::new(),
            },
            collapsed_tiles: CollapsedTiles{ queue: VecDeque::new() },
        };
        
        info!("train rules with top secret ultra complex algorithm");
        generator.train_rules();

        info!("init possibilites for each pattern position");
        generator.possibilities_for_tiles = MultiVec::new(
            Possibilities::new(generator.patterns.len()),
            generator.edge_length.output_with_space_for_patterns(),
            generator.edge_length.output_with_space_for_patterns(),
        );

        info!("init entropy cache data structure");
        generator.entropy_for_tile = MultiVec::new(
            1e9,
            generator.edge_length.output_with_space_for_patterns(),
            generator.edge_length.output_with_space_for_patterns()
        );

        for i in 0..generator.entropy_for_tile.data.len()
        {
            let (x, y) = generator.entropy_for_tile.index_to_xy(i).unwrap();
            *generator.entropy_for_tile.get_mut(x, y).unwrap() = generator.get_shannon_entropy_for_tile(x, y);
        }

        info!("generator initialized.");

        return generator
    }

    pub fn generate(&mut self) -> MultiVec<i32>
    {
        while !self.is_finished()
        {
            info!("collapse single pattern position...");
            let chosen_possibility = self.collapse_one_possibility();

            self.collapsed_tiles.clear(); // ignore collapsed tiles
            
            match chosen_possibility {
                Some((x_chosen_tile, y_chosen_tile)) =>
                {
                    self.propagate_chosen_possibility(x_chosen_tile, y_chosen_tile);

                    self.collapsed_tiles.clear(); // ignore collapsed tiles
                },
                None =>
                {
                    info!("create output tiles...");
                    return self.create_output_tiles();
                },
            }
        }

        info!("create output tiles...");
        self.create_output_tiles()
    }
}

impl Iterator for WaveFunctionCollapseGenerator
{
    type Item = (usize, usize, i32); // (x, y, tile_type)

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.is_finished() { return None; }

        if !self.collapsed_tiles.queue.is_empty() {
            return Some(self.collapsed_tiles.queue.pop_front().unwrap());
        }
        
        debug!("collapse single pattern position...");
        match self.collapse_one_possibility()
        {
            Some((x,y)) => {
                debug!("Pattern at {x},{y} was collapsed");
                self.propagate_chosen_possibility(x, y);

                Some(self.collapsed_tiles.queue.pop_front().unwrap())
            }
            None => None,
        }
    }
}
