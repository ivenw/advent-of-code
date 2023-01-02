use indexmap::IndexMap;
use itertools::Itertools;
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
    fs,
    hash::Hash,
};

use day_17::{
    input_parser::{parse_input, Direction},
    rocks::{rock_formations, Coord, Rock, RockFormation},
};

extern crate day_17;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Chamber(BTreeMap<Coord, Rock>);

impl Chamber {
    fn new() -> Self {
        let mut chamber = BTreeMap::new();

        for x in 0..7 {
            chamber.insert(Coord(x, 0), Rock::Rock);
        }
        Self(chamber)
    }
}

impl Chamber {
    fn tower_height(&self) -> usize {
        *self.0.keys().map(|Coord(_, y)| y).max().unwrap()
    }

    fn can_place_rock_formation(&self, rock_formation: &RockFormation, pos: &Coord) -> bool {
        rock_formation
            .get_rock_positions(pos)
            .iter()
            .all(|coord| !self.0.contains_key(coord))
    }

    fn place_rock_formation(&mut self, rock_formation: &RockFormation, pos: &Coord) {
        for coord in &rock_formation.get_rock_positions(pos) {
            self.0.insert(*coord, Rock::Rock);
        }
    }

    fn full_row_idx(&self) -> usize {
        (1..=self.tower_height())
            .map(|y| {
                if (0..7).all(|x| self.0.contains_key(&Coord(x, y))) {
                    y
                } else {
                    0
                }
            })
            .max()
            .unwrap()
    }
}

impl Display for Chamber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let max_rock_height = self.tower_height();
        let y_range = 0..=max_rock_height;
        let x_range = 0..7;
        let results = y_range
            .rev()
            .cartesian_product(x_range)
            .chunks(7)
            .into_iter()
            .map(|chunk| {
                chunk
                    .map(|(y, x)| match self.0.get(&Coord(x, y)) {
                        Some(rock) => match rock {
                            Rock::Rock => "#",
                            Rock::Gap => ".",
                        },
                        None => ".",
                    })
                    .collect::<String>()
            })
            .join("|\n|");
        write!(f, "|{}|\n+{}+", results, "-".repeat(7))
    }
}

fn main() {
    let input = fs::read_to_string("test_input.txt").unwrap();

    let result = function(input);

    println!("{}", result);
}

fn function(input: String) -> u32 {
    let (_, directions) = parse_input(&input).unwrap();
    let rock_formations = rock_formations();

    let mut directions = directions.iter().cycle();
    let mut falling_rocks = rock_formations.iter().cycle();

    let mut chamber = Chamber::new();
    let mut states: IndexMap<Chamber, BTreeMap<i32, i32>> = IndexMap::new();

    let mut stopped_rocks = 0;
    let mut tower_height = 0;
    let mut duplicate_state_index = 0;
    let mut state = BTreeMap::new();

    let rocks_to_be_stopped = 100_000;

    'outer: for _ in 0..rocks_to_be_stopped {
        let falling_rock = falling_rocks.next().unwrap();

        let mut current_rock_position =
            Coord(2, chamber.tower_height() + 3 + falling_rock.height());

        loop {
            let next_direction = directions.next().unwrap();

            // push left or right
            current_rock_position = move_rock(
                next_direction,
                current_rock_position,
                falling_rock,
                &chamber,
            );

            // keep falling
            let new_rock_position = move_rock(
                &Direction::Down,
                current_rock_position,
                falling_rock,
                &chamber,
            );

            if new_rock_position == current_rock_position {
                let height_before = chamber.tower_height();
                chamber.place_rock_formation(falling_rock, &current_rock_position);

                // keep track of state
                stopped_rocks += 1;
                tower_height += chamber.tower_height() - height_before;
                state.insert(stopped_rocks, tower_height as i32);

                let full_row_idx = chamber.full_row_idx();
                if full_row_idx > 0 {
                    println!("Full row found");
                    // remove every row smaller than the full row idx from the chamber
                    chamber.0.retain(|&k, _| k.1 >= full_row_idx);
                    chamber.0 = chamber
                        .0
                        .into_iter()
                        .map(|(k, v)| (Coord(k.0, k.1 - full_row_idx), v))
                        .collect();

                    if states.contains_key(&chamber) {
                        duplicate_state_index = states.get_index_of(&chamber).unwrap();
                        println!("Duplicate at index {}", duplicate_state_index);
                        // break 'outer;
                    }
                    states.insert(chamber.clone(), state.clone());
                    state = BTreeMap::new();
                }
                break;
            } else {
                current_rock_position = new_rock_position;
            }
        }
    }

    0
}

fn move_rock(
    next_direction: &Direction,
    current_rock_position: Coord,
    falling_rock: &RockFormation,
    chamber: &Chamber,
) -> Coord {
    match next_direction {
        Direction::Left => {
            if current_rock_position.0.checked_sub(1).is_some() {
                let desired_rock_position = current_rock_position.sub(&Coord(1, 0));

                if chamber.can_place_rock_formation(falling_rock, &desired_rock_position) {
                    desired_rock_position
                } else {
                    current_rock_position
                }
            } else {
                current_rock_position
            }
        }
        Direction::Right => {
            if current_rock_position.0 + falling_rock.width() < 7 {
                let next_rock_position = current_rock_position.add(&Coord(1, 0));

                if chamber.can_place_rock_formation(falling_rock, &next_rock_position) {
                    next_rock_position
                } else {
                    current_rock_position
                }
            } else {
                current_rock_position
            }
        }
        Direction::Down => {
            if current_rock_position.1.checked_sub(1).is_some() {
                let next_rock_position = current_rock_position.sub(&Coord(0, 1));

                if chamber.can_place_rock_formation(falling_rock, &next_rock_position) {
                    next_rock_position
                } else {
                    current_rock_position
                }
            } else {
                current_rock_position
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function() {
        // read test input file into a string
        let input = fs::read_to_string("test_input.txt").unwrap();

        let result = function(input);

        assert_eq!(result, 3137);
    }
}
