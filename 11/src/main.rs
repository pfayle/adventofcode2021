use std::collections::HashMap;
use std::io;

const WORLD_SIZE: (usize, usize) = (10, 10);
const ENERGY_LIMIT: i32 = 9;
const MAX_TRIES: i32 = 1000;
const DEBUG: bool = false;

fn main() {
    let mut world = World{
        size: WORLD_SIZE,
        octopodes: HashMap::new(),
    };
    let mut lines: Vec<String> = vec![];
    loop {
        let mut buf = String::new();
        let n = io::stdin().read_line(&mut buf);
        match n {
            Ok(0) => { break },
            Ok(_) => { lines.push(String::from(buf.trim())) },
            Err(_) => { break; },
        }
    }
    for m in 0..lines.len() {
        for n in 0..lines[0].len() {
            world.octopodes.insert(
                (m, n),
                Octopus {
                    location: (m, n),
                    energy: lines[m].chars().nth(n).unwrap().to_digit(10).unwrap() as i32,
                    flashed: false
                }
            );
        }
    }
    if DEBUG {
        world.draw();
    }
    let mut count = 0;
    let mut sync = 0;
    let mut count100 = 0;
    for n in 1..=MAX_TRIES {
        let k = world.tick();
        count += k;
        if sync == 0 && k == world.size.0 * world.size.1 {
            sync = n;
        }
        if n == 100 {
            count100 = count;
        }
        if DEBUG {
            println!("Tick {}", n);
            println!("{} flashes this tick.", k);
            world.draw();
        }
    }
    println!("{} flashes in first 100 steps", count100);
    println!("First synchronisation at step {}", sync);
}

#[derive(Debug)]
struct Octopus {
    location: (usize, usize),
    energy: i32,
    flashed: bool,
}

#[derive(Debug)]
struct World {
    size: (usize, usize),
    octopodes: HashMap<(usize, usize), Octopus>,
}

impl World {
    fn tick(self: &mut Self) -> usize {
        let mut flash_locations: Vec<(usize, usize)> = vec![];
        for octo in self.octopodes.values_mut() {
            octo.flashed = false;
            octo.increase();
            if octo.flashed {
                flash_locations.push(octo.location);
            }
        }
        while flash_locations.len() > 0 {
            let mut new_locations: Vec<(usize, usize)> = vec![];
            for loc in flash_locations {
                for pos in self.neighbour_locations(loc) {
                    let octo = self.octopodes.get_mut(&pos).unwrap();
                    if octo.flashed {
                        continue;
                    }
                    octo.increase();
                    if octo.flashed {
                        new_locations.push(octo.location);
                    }
                }
            }
            flash_locations = new_locations;
        }
        self.octopodes.values().filter(|&o| o.flashed).count()
    }

    fn neighbour_locations(self: &Self, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let mut r = vec![];
        if pos.0 > 0 {
            r.push((pos.0-1,pos.1));
            if pos.1 > 0 {
                r.push((pos.0-1,pos.1-1));
            }
            if pos.1+1 < self.size.1 {
                r.push((pos.0-1,pos.1+1));
            }
        }
        if pos.1 > 0 {
            r.push((pos.0,pos.1-1));
        }
        if pos.0+1 < self.size.0 {
            r.push((pos.0+1,pos.1));
            if pos.1 > 0 {
                r.push((pos.0+1,pos.1-1));
            }
            if pos.1+1 < self.size.1 {
                r.push((pos.0+1,pos.1+1));
            }
        }
        if pos.1+1 < self.size.1 {
            r.push((pos.0,pos.1+1));
        }
        r
    }

    fn draw(self: &Self) {
        for x in 0..self.size.0 {
            for y in 0..self.size.1 {
                print!("{}", self.octopodes.get(&(x, y)).unwrap().energy);
            }
            print!("\n");
        }
    }
}

impl Octopus {
    fn increase(self: &mut Self) {
        if !self.flashed {
            self.energy += 1;
            if self.energy > ENERGY_LIMIT {
                self.flashed = true;
                self.energy = 0;
            }
        }
    }
}
