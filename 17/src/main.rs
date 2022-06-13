use std::{io, io::Read};
use regex::Regex;

const PATTERN: &str = r"(?m)^target area: x=(-?\d+)\.\.(-?\d+), y=(-?\d+)\.\.(-?\d+)$";
fn main() {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).expect("Problem reading input");
    let target = Target::from(buf.trim());

    let mut max_y = 0;
    let mut velocities: Vec<(isize, isize)> = vec![];
    if target.x1 > 0 {
        for init_y in target.y0..=-target.y0 {
            for init_x in (1..=target.x1).rev() {
                let probe = Probe {
                    position: (0, 0),
                    x_velocity: init_x,
                    y_velocity: init_y,
                };
                let y = probe.highest_y(&target, 0);
                if let Some(v) = y {
                    velocities.push((init_x, init_y));
                    max_y = max_y.max(v);
                }
            }
        }
    }
    println!("Highest y position: {}", max_y);
    println!("{} distinct initial velocity values", velocities.len());
}

struct Target {
    x0: isize, x1: isize, y0: isize, y1: isize,
}

impl From<&str> for Target {
    fn from(s: &str) -> Self {
        let r: Regex = Regex::new(PATTERN).unwrap();
        if let Some(caps) = r.captures(s) {
            Self{
                x0: caps[1].parse::<isize>().unwrap(),
                x1: caps[2].parse::<isize>().unwrap(),
                y0: caps[3].parse::<isize>().unwrap(),
                y1: caps[4].parse::<isize>().unwrap(),
            }
        } else {
            Self{
                x0: 0, x1: 0, y0: 0, y1: 0,
            }
        }
    }
}

struct Probe {
    position: (isize, isize),
    x_velocity: isize,
    y_velocity: isize,
}

impl Probe {
    fn step(&self) -> Self {
        Self{
            position: ( self.position.0 + self.x_velocity, self.position.1 + self.y_velocity),
            x_velocity: match self.x_velocity {
                1.. => self.x_velocity - 1,
                0 => 0,
                _ => self.x_velocity + 1,
            },
            y_velocity: self.y_velocity - 1,
        }
    }

    fn intersecting(&self, target: &Target) -> bool {
        self.position.0.clamp(target.x0, target.x1) == self.position.0 
        && self.position.1.clamp(target.y0, target.y1) == self.position.1
    }

    fn doomed(&self, target: &Target) -> bool {
        !self.intersecting(target) &&
        (
            (self.position.0 < target.x0 && self.x_velocity <= 0)
            || (self.position.0 > target.x1 && self.x_velocity >= 0)
            || self.position.1 < target.y0
        )
    }

    fn highest_y(&self, target: &Target, current_max: isize) -> Option<isize> {
        if self.intersecting(target) {
            Some(current_max.max(self.position.1))
        } else if !self.doomed(target) {
            self.step().highest_y(target, current_max.max(self.position.1))
        } else {
            None
        }
    }
}
