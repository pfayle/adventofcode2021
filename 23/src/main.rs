use std::collections::HashMap;
use std::hash::Hash;
use std::{fmt::Display, vec};
use std::io;
use std::io::Read;
use clap::{arg, command};

const LETTERS: [char; 4] = ['A', 'B', 'C', 'D'];
const ENERGY_COST: [usize; 4] = [1, 10, 100, 1000];
const INSERTION: [&str; 2] = ["  #D#C#B#A#", "  #D#B#A#C#"];

fn main() {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).expect("Error reading stdin.");
    let matches = command!("")
    .arg(arg!(
        -i --insert ... "Insert lines for part two"
    ).takes_value(false))
    .arg(arg!(
        -d --debug ... "Debug"
    ).takes_value(false))
    .get_matches();

    let debug = matches.is_present("debug");

    let mut lines = buf.lines();
    let mut s = lines.next().unwrap().to_string() + "\n";
    s += lines.next().unwrap();
    s += "\n";
    s += lines.next().unwrap();
    s += "\n";
    if matches.is_present("insert") {
        for line in INSERTION {
            s += line;
            s += "\n";
        }
    }
    for line in lines {
        s += line;
        s += "\n";
    }
    if debug {
        println!("{}", s);
    }
    let mut f = Floor::from_string(&s);
    f.debug(debug);
    let mut energies: HashMap<Floor, Option<usize>> = HashMap::new();
    let r = f.min_energy(None, &mut 0, &mut energies);
    println!("Minimum energy: {:?}", r.unwrap());
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct Occupant(char);

impl Occupant {
    fn move_cost(&self) -> usize {
        ENERGY_COST[LETTERS.iter().position(|c| *c == self.0).unwrap()]
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct Floor {
    halls: Vec<Hall>,
    alleys: Vec<Alley>,
    debug: bool,
}

impl Floor {
    fn debug(&mut self, v: bool) {
        self.debug = v;
    }

    fn from_string(s: &str) -> Self {
        let sp = s.trim().split('\n').skip(2);
        let mut occupants: Vec<Vec<Occupant>> = vec![];
        for line in sp {
            let chars = line.chars().filter(|c| LETTERS.contains(c)).collect::<Vec<char>>();
            if chars.is_empty() { break; }
            occupants.push(chars.iter().map(|c| Occupant(*c)).collect::<Vec<Occupant>>());
        }
        let alley_count = occupants[0].len();
        let hall_count = alley_count + 3;
        let mut alleys = vec![];
        let halls = vec![Hall{
            occupant: None,
        }; hall_count];
        for i in 0..alley_count {
            let mut rooms = vec![];
            for row in &occupants {
                rooms.push(Room { letter: LETTERS[i], occupant: Some(row[i]) });
            }
            alleys.push(Alley{ rooms });
        }
        Floor { halls, alleys, debug: false }
    }

    fn move_valid(&self, m: Move) -> bool {
        let hall = self.halls.get(m.hall_index).unwrap();
        let alley = self.alleys.get(m.alley_index).unwrap();
        match m.direction {
            Direction::ToHall => {
                if hall.occupant.is_some() {
                    false
                } else if alley.next_occupant().is_some() {
                    self.clear_path(m.hall_index, m.alley_index)
                } else {
                    false
                }
            },
            Direction::ToAlley => {
                if let Some(o) = hall.occupant {
                    alley.is_ready() && alley.is_dest(o) && alley.open() && self.clear_path(m.hall_index, m.alley_index)
                } else {
                    false
                }
            },
        }
    }

    fn moves(&self) -> Vec<Move> {
        let mut r = vec![];
        for h in 0..self.halls.len() {
            if self.halls.get(h).unwrap().occupant.is_some() {
                for a in 0..self.alleys.len() {
                    let m = Move{
                        hall_index: h,
                        alley_index: a,
                        direction: Direction::ToAlley,
                    };
                    if self.move_valid(m) {
                        r.push(m);
                    }
                }
            } else {
                for a in 0..self.alleys.len() {
                    let m = Move{
                        hall_index: h,
                        alley_index: a,
                        direction: Direction::ToHall,
                    };
                    if self.move_valid(m) {
                        r.push(m);
                    }
                }
            }
        }
        r
    }

    fn move_length(&self, m: Move) -> usize {
        let al = self.alleys.get(m.alley_index).unwrap();
        let mut r = 1; // space in front of room
        // 2 spaces to reach each intermediate hall
        if m.hall_index > m.alley_index + 1 {
            r += (m.hall_index - m.alley_index - 2) * 2
        } else {
            r += (m.alley_index + 1 - m.hall_index) * 2
        }
        // minus double-counting for end rooms
        if m.hall_index == 0 || m.hall_index == self.halls.len() - 1 {
            r -= 1;
        }
        // 1 space for each unoccupied room
        r += al.rooms.iter().filter(|r| !r.occupied()).count();
        // move to hall
        if m.direction == Direction::ToHall {
            r += 1;
        }
        r
    }

    fn do_move(&self, m: Move) -> (Self, usize) {
        let mut energy = 0;
        let mut floor = self.clone();
        if !floor.move_valid(m) {
            panic!();
        }
        let mut h = floor.halls.get_mut(m.hall_index).unwrap();
        let al = floor.alleys.get_mut(m.alley_index).unwrap();
        match m.direction {
            Direction::ToHall => {
                if let Some(o) = al.next_occupant() {
                    h.occupant = Some(o);
                    energy += o.move_cost() * self.move_length(m);
                    al.remove_occupant();
                }
            },
            Direction::ToAlley => {
                if let Some(o) = h.occupant {
                    h.occupant = None;
                    energy += o.move_cost() * self.move_length(m);
                    al.add_occupant(o);
                }
            },
        }
        (floor, energy)
    }

    fn complete(&self) -> bool {
        self.alleys.iter().filter(|a| !a.is_complete()).count() == 0
    }

    fn min_energy(&self, mut limit: Option<usize>, counter: &mut usize, energies: &mut HashMap<Floor, Option<usize>>) -> Option<usize> {
        if energies.contains_key(self) {
            return *energies.get(self).unwrap();
        }
        if self.complete() {
            return Some(0);
        }
        let mut min: Option<usize> = None;
        let mut l: Option<usize> = limit;
        for m in self.moves() {
            let (f, e) = self.do_move(m);
            *counter += 1;
            if self.debug && *counter % 1000000 == 0 {
                println!("{} moves calculated; {} hashes", counter, energies.len());
            }
            if let Some(v) = limit {
                if e >= v {
                    continue;
                } else {
                    l = Some(v - e);
                }
            }
            let r = f.min_energy(l, counter, energies);
            energies.insert(f, r);
            if let Some(e2) = r {
                if let Some(x) = min {
                    if e+e2 < x {
                        min = Some(e+e2);
                        limit = min;
                    }
                } else {
                    min = Some(e+e2);
                    limit = min;
                }
            }
        }
        min
    }

    fn clear_path(&self, h_in: usize, a_in: usize) -> bool {
        for i in 0..self.halls.len() {
            if h_in <= a_in + 1 {
                if i > h_in && i <= a_in + 1 && self.halls.get(i).unwrap().occupant.is_some() {
                    return false;
                }
            } else if i < h_in && i > a_in + 1 && self.halls.get(i).unwrap().occupant.is_some() {
                return false;
            }
        }
        true
    }
}

impl Display for Floor {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..=(self.halls.len()+self.alleys.len()+1) {
            print!("#");
        }
        println!();
        print!("#{}", self.halls[0].occupant.unwrap_or(Occupant('.')).0);
        for h in &self.halls[1..self.halls.len()-2] {
            print!("{}.", h.occupant.unwrap_or(Occupant('.')).0);
        }
        print!("{}{}#",
            self.halls[self.halls.len()-2].occupant.unwrap_or(Occupant('.')).0,
            self.halls[self.halls.len()-1].occupant.unwrap_or(Occupant('.')).0
        );
        println!();
        print!("###");
        for a in &self.alleys {
            print!("{}#", a.rooms[0].occupant.unwrap_or(Occupant('.')).0);
        }
        println!("##");
        for i in 1..self.alleys[0].rooms.len() {
            print!("  #");
            for a in &self.alleys {
                print!("{}#", a.rooms[i].occupant.unwrap_or(Occupant('.')).0);
            }
            println!();
        }
        print!("  ");
        for _ in 0..2*self.alleys.len()+1 {
            print!("#");
        }
        println!();
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    ToHall,
    ToAlley,
}

#[derive(Clone, Copy, Debug)]
struct Move {
    hall_index: usize,
    alley_index: usize,
    direction: Direction,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct Room {
    letter: char,
    occupant: Option<Occupant>,
}

impl Room {
    fn occupied(&self) -> bool {
        self.occupant.is_some()
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct Hall {
    occupant: Option<Occupant>,
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct Alley {
    rooms: Vec<Room>,
}

impl Alley {
    fn is_dest(&self, o: Occupant) -> bool {
        self.letter() == o.0
    }
    fn letter(&self) -> char {
        self.rooms[0].letter
    }
    fn is_complete(&self) -> bool {
        self.rooms.iter().filter(|r| !r.occupied() || !self.is_dest(r.occupant.unwrap())).count() == 0
    }

    // if there are any non-dest occupants, return the next occupant
    fn next_occupant(&self) -> Option<Occupant> {
        if self.rooms.iter().any(|r| r.occupied() && r.occupant.unwrap().0 != r.letter) {
            self.rooms.iter().find_map(|r| r.occupant)
        } else {
            None
        }
    }
    fn open(&self) -> bool {
        self.rooms.iter().filter(|r| !r.occupied()).count() > 0
    }
    fn is_ready(&self) -> bool {
        self.rooms.iter().filter(|r| r.occupied()).filter(|r| !self.is_dest(r.occupant.unwrap())).count() == 0
    }
    fn add_occupant(&mut self, o: Occupant) {
        self.rooms.iter_mut().rev().find(|r| !r.occupied()).unwrap().occupant = Some(o);
    }
    fn remove_occupant(&mut self) {
        for r in self.rooms.iter_mut() {
            if r.occupant.is_some() {
                r.occupant = None;
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn swap() {
        let f = Floor{
            halls: vec![Hall{occupant: None}; 5],
            alleys: vec![
                Alley{
                    rooms: vec![
                        Room{
                            letter: 'A',
                            occupant: Some(Occupant('B')),
                        },
                        Room{
                            letter: 'A',
                            occupant: Some(Occupant('B')),
                        }
                    ],
                },
                Alley{
                    rooms: vec![
                        Room{
                            letter: 'B',
                            occupant: Some(Occupant('A')),
                        },
                        Room{
                            letter: 'B',
                            occupant: Some(Occupant('A')),
                        },
                    ],
                },
            ],
            debug: false,
        };
        let moves = f.moves();
        assert_eq!(moves.len(), 10);
        assert_eq!(f.do_move(moves[0]).1, 30);
        assert_eq!(f.do_move(moves[1]).1, 5);
        assert_eq!(f.do_move(moves[2]).1, 20);
        assert_eq!(f.do_move(moves[3]).1, 4);
        assert_eq!(f.do_move(moves[4]).1, 20);
        assert_eq!(f.do_move(moves[5]).1, 2);
        assert_eq!(f.do_move(moves[6]).1, 40);
        assert_eq!(f.do_move(moves[7]).1, 2);
        assert_eq!(f.do_move(moves[8]).1, 50);
        assert_eq!(f.do_move(moves[9]).1, 3);
    }
    
    #[test]
    fn clear_hallway() {
        let f = Floor{
            halls: vec![
                Hall{occupant: None},
                Hall{occupant: Some(Occupant('A'))},
                Hall{occupant: None},
                Hall{occupant: None},
                Hall{occupant: None},
            ],
            alleys: vec![
                Alley{
                    rooms: vec![
                        Room{
                            letter: 'A',
                            occupant: None,
                        },
                        Room{
                            letter: 'A',
                            occupant: Some(Occupant('B')),
                        }
                    ],
                },
                Alley{
                    rooms: vec![
                        Room{
                            letter: 'B',
                            occupant: None,
                        },
                        Room{
                            letter: 'B',
                            occupant: None,
                        },
                    ],
                },
            ],
            debug: false,
        };
        assert!(f.clear_path(3, 0));
    }

    #[test]
    fn blocked_hallway() {
        let f = Floor{
            halls: vec![
                Hall{occupant: None},
                Hall{occupant: Some(Occupant('A'))},
                Hall{occupant: Some(Occupant('B'))},
                Hall{occupant: None},
                Hall{occupant: None},
            ],
            alleys: vec![
                Alley{
                    rooms: vec![
                        Room{
                            letter: 'A',
                            occupant: None,
                        },
                        Room{
                            letter: 'A',
                            occupant: Some(Occupant('B')),
                        }
                    ],
                },
                Alley{
                    rooms: vec![
                        Room{
                            letter: 'B',
                            occupant: None,
                        },
                        Room{
                            letter: 'B',
                            occupant: None,
                        },
                    ],
                },
            ],
            debug: false,
        };
        assert!(!f.clear_path(3, 0));
        assert!(!f.clear_path(0, 3));
    }
    #[test]
    fn full_swap() {
        let f = Floor{
            halls: vec![
                Hall{occupant: None},
                Hall{occupant: None},
                Hall{occupant: None},
                Hall{occupant: None},
                Hall{occupant: None},
            ],
            alleys: vec![
                Alley{
                    rooms: vec![
                        Room{
                            letter: 'A',
                            occupant: Some(Occupant('B')),
                        },
                        Room{
                            letter: 'A',
                            occupant: Some(Occupant('A')),
                        }
                    ],
                },
                Alley{
                    rooms: vec![
                        Room{
                            letter: 'B',
                            occupant: Some(Occupant('A')),
                        },
                        Room{
                            letter: 'B',
                            occupant: Some(Occupant('B')),
                        },
                    ],
                },
            ],
            debug: false,
        };
        assert_eq!(f.min_energy(None, &mut 0, &mut HashMap::new()), Some(46));
    }

    #[test]
    fn from_string() {
        let s = "\
            #########\n\
            #.......#\n\
            ###B#A###\n\
              #A#B#  \n\
              #####  \n";
        let f = Floor::from_string(s);
        assert_eq!(f.halls.len(), 5);
        assert_eq!(f.alleys.len(), 2);
        assert_eq!(f.alleys[0].letter(), 'A');
        assert_eq!(f.alleys[1].letter(), 'B');
        assert_eq!(f.min_energy(None, &mut 0, &mut HashMap::new()), Some(46));
    }

    #[test]
    fn part1_example() {
        let s = fs::read_to_string("example.txt").unwrap();
        let f = Floor::from_string(&s);
        let r = f.min_energy(None, &mut 0, &mut HashMap::new());
        assert_eq!(r, Some(12521));
    }

    #[test]
    fn four_deep_room_string() {
        let s = "\
            #########\n\
            #.......#\n\
            ###B#A###\n\
              #B#A#  \n\
              #B#A#  \n\
              #A#B#  \n\
              #####  \n";
        let f = Floor::from_string(s);
        assert_eq!(f.halls.len(), 5);
        assert_eq!(f.alleys.len(), 2);
        assert_eq!(f.alleys[0].rooms.len(), 4);
        assert_eq!(f.alleys[0].letter(), 'A');
        assert_eq!(f.alleys[1].letter(), 'B');
        assert_eq!(f.min_energy(None, &mut 0, &mut HashMap::new()), Some(206));
    }
}
