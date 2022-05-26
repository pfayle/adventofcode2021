use std::io::{self};
use std::io::Read;
use std::fmt::{Display, Formatter, Error};

const DEBUG: bool = false;

fn main() {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).expect("Error reading stdin.");
    let v: Vec<&str> = buf.split("\n\n").collect();
    let mut image = Image::create(v[0], v[1]);
    println!("{}\n{} lit.", image, image.count_lit());
    for _ in 0..50 {
        image = image.transform();
        if DEBUG {
            println!("{}\n{} lit.", image, image.count_lit());
        }
    }
    println!("{} lit.", image.count_lit());
}

fn char_to_bool(c: char) -> bool {
    c == '#'
}

struct Image {
    default: bool,
    enhancement_algo: String,
    pixels: Vec<Vec<bool>>,
}

impl Display for Image {
    fn fmt(&self, _: &mut Formatter) -> Result<(), Error> {
        for _ in 0..3 {
            for _ in 0..self.pixels[0].len()+6 {
                print!("{}", if self.default {'#'} else {'.'});
            }
            print!("\n");
        }
        for row in &self.pixels {
            for _ in 0..3 {
                print!("{}", if self.default {'#'} else {'.'});
            }
            for &c in row {
                print!("{}", if c {'#'} else {'.'});
            }
            for _ in 0..3 {
                print!("{}", if self.default {'#'} else {'.'});
            }
            print!("\n");
        }
        for _ in 0..3 {
            for _ in 0..self.pixels[0].len()+6 {
                print!("{}", if self.default {'#'} else {'.'});
            }
            print!("\n");
        }
        Ok(())
    }
}

impl Image {
    fn dims(&self) -> (usize, usize) {
        (self.pixels.len(), self.pixels[0].len())
    }
    fn create(enhancement_algo: &str, input: &str) -> Self {
        let mut pixels = vec![];
        for line in input.trim().split('\n') {
            let cs = line.chars().map(|c| c == '#')
                .collect();
            pixels.push(cs);
        }
        Self { default: false, enhancement_algo: String::from(enhancement_algo), pixels }
    }
    fn get_with_border(&self, pos: (isize, isize)) -> bool {
        let dims = self.dims();
        if pos.0 < 0 || pos.1 < 0 {
            self.default
        } else if (pos.0 as usize) < dims.0 && (pos.1 as usize) < dims.1 {
            self.pixels[pos.0 as usize][pos.1 as usize]
        } else {
            self.default
        }
    }
    fn get_index(&self, pos: (isize, isize)) -> usize {
        let mut r = 0;
        for x in pos.0-1..=pos.0+1 {
            for y in pos.1-1..=pos.1+1 {
                r = 2*r + self.get_with_border((x, y)) as u8 as usize;
            }
        }
        r
    }
    fn next_default(&self) -> bool {
        let index = if self.default {511} else {0};
        char_to_bool(self.enhancement_algo.chars().nth(index).unwrap())
    }
    fn transform(&self) -> Self {
        let mut pixels: Vec<Vec<bool>> = vec![];
        let default = self.next_default();
        for r in -2..(self.pixels.len() as isize)+2 {
            let mut row = vec![];
            for c in -2..(self.pixels[0].len() as isize)+2 {
                let index = self.get_index((r, c));
                row.push(char_to_bool(self.enhancement_algo.chars().nth(index).unwrap()));
            }
            pixels.push(row);
        }
        Self{ default, enhancement_algo: self.enhancement_algo.clone(), pixels }
    }

    fn count_lit(&self) -> usize {
        self.pixels.iter().flat_map(|r| r.iter()).filter(|&&b| b).count()
    }
}
