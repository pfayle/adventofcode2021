use std::collections::HashMap;
use std::io;
use std::io::Read;

const DEBUG: bool = false;

fn main() {
    let mut buf: String = String::new();
    io::stdin().read_to_string(&mut buf).expect("Error reading stdin.");
    let mut cucumbers_right = vec![];
    let mut cucumbers_down = vec![];
    let mut locations: HashMap<(usize, usize), Contents> = HashMap::new();
    let (mut row, mut col) = (0, 0);
    for line in buf.trim().split('\n') {
        col = 0;
        for c in line.chars() {
            match c {
                '>' => {
                    cucumbers_right.push(Cucumber{
                        location: (row, col),
                        to_move: false,
                    });
                    locations.insert((row, col), Contents::RIGHT);
                },
                'v' => {
                    cucumbers_down.push(Cucumber{
                        location: (row, col),
                        to_move: false,
                    });
                    locations.insert((row, col), Contents::DOWN);
                },
                _ => {
                    locations.insert((row, col), Contents::EMPTY);
                }
            }
            col += 1;
        }
        row += 1;
    }
    if DEBUG {
        display(row, col, &locations);
    }
    let mut moves = 0;
    loop {
        moves += 1;
        let tick_moves = tick(
            &mut cucumbers_right,
            &mut cucumbers_down,
            &mut locations,
            (row, col)
        );
        if DEBUG {
            println!("{} moved this step", tick_moves);
        }
        if tick_moves == 0 {
            break;
        }
    }
    println!("No cucumbers moved after {} steps", moves);
}

fn tick<'a>(
    c_right: &'a mut Vec<Cucumber>,
    c_down: &'a mut Vec<Cucumber>,
    loc: &'a mut HashMap<(usize, usize), Contents>,
    dims: (usize, usize)
) -> u32 {
    let mut moves = 0;
    for c in c_right.iter_mut() {
        if let Contents::EMPTY = loc.get(&(c.location.0, (c.location.1+1)%dims.1)).unwrap() {
            c.to_move = true;
            moves += 1;
        }
    }
    for c in c_right {
        if c.to_move {
            loc.insert(c.location, Contents::EMPTY);
            c.location.1 = (c.location.1+1)%dims.1;
            loc.insert(c.location, Contents::RIGHT);
            c.to_move = false;
        } else {
        }
    }
    for c in c_down.iter_mut() {
        if let Contents::EMPTY = loc.get(&((c.location.0+1)%dims.0, c.location.1)).unwrap() {
            c.to_move = true;
            moves += 1;
        }
    }
    for c in c_down {
        if c.to_move {
            loc.insert(c.location, Contents::EMPTY);
            c.location.0 = (c.location.0+1)%dims.0;
            loc.insert(c.location, Contents::DOWN);
            c.to_move = false;
        } else {
        }
    }
    if DEBUG {
        display(dims.0, dims.1, loc);
    }
    moves
}

fn display(rows: usize, cols: usize, locs: &HashMap<(usize, usize), Contents>) {
    for r in 0..rows {
        for c in 0..cols {
            print!("{}",
                match *locs.get(&(r, c)).unwrap() {
                    Contents::EMPTY => '.',
                    Contents::DOWN => 'v',
                    Contents::RIGHT => '>',
                }
            );
        }
        print!("\n");
    }
    print!("\n");
}
struct Cucumber {
    location: (usize, usize),
    to_move: bool,
}

enum Contents {
    EMPTY,
    DOWN,
    RIGHT,
}
