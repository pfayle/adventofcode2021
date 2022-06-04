use std::isize;
use std::{fmt::Display};
use itertools::{iproduct};
use std::{io, io::Read};
use std::env;
use std::ops::Range;

use regex::Regex;

const REGEX: &str = r"(?m)^(on|off) x=(-?\d+)\.\.(-?\d+),y=(-?\d+)\.\.(-?\d+),z=(-?\d+)\.\.(-?\d+)$";
const DEBUG: bool = true;
const SOLUTION_ONE: bool = false;

fn main() {
    let bounded = env::args().count() == 1;

    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).expect("Error reading stdin.");

    let ins: Vec<Instruction> = buf.trim().lines().into_iter()
        .map(|line| Instruction::from(line.to_string())).collect();

    if SOLUTION_ONE {
        solution_one(&ins, &bounded);
    } else {
        solution_two(&ins[..], &bounded);
    }
}

fn solution_two(ins: &[Instruction], &bounded: &bool) {
    let iter = ins.iter();
    let mut cuboids: Vec<WeightedCuboid> = vec![];
    for ins in iter {
        let insc = ins.cuboid(bounded);
        let mut v = vec![];
        for c in &cuboids {
            if let Some(int) = insc.intersection(&c.cuboid) {
                // flip the type with each intersection to remove double-counting
                let kind = match c.state {
                    CuboidWeight::Positive => CuboidWeight::Negative,
                    _ => CuboidWeight::Positive,
                };
                v.push(WeightedCuboid{
                    cuboid: int,
                    state: kind,
                });
            }
        }
        if ins.direction == "on" {
            v.push(WeightedCuboid{
                cuboid: insc,
                state: CuboidWeight::Positive,
            });
        }
        cuboids.append(&mut v);
    }
    let count: isize = cuboids.iter().map(|c| c.count()).sum();
    println!("Total: {}", count);
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CuboidWeight {
    Positive,
    Negative,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct WeightedCuboid {
    cuboid: Cuboid,
    state: CuboidWeight,
}

impl WeightedCuboid {
    fn count(&self) -> isize {
        self.cuboid.volume() as isize * match self.state {
            CuboidWeight::Positive => {
                1
            },
            CuboidWeight::Negative => {
                -1
            },
        }
    }
}

fn solution_one(ins: &Vec<Instruction>, &bounded: &bool) {
    let cuboids: Vec<Cuboid> = ins.iter().map(|i| i.cuboid(bounded)).collect();
    let refs: Vec<&Cuboid> = cuboids.iter().collect();
    let big_partition = Cuboid::partition_connected_cuboids(&refs[..]);
    if DEBUG {
        println!("{} big partitions", big_partition.len());
    }
    let mut cuboids: usize = 0;
    let mut total: usize = 0;
    for big_part in big_partition {
        if DEBUG {
            println!("Big partition has {} cuboids before partitioning", big_part.len());
        }
        let small_partition = Cuboid::disjoint_cover(&big_part);
        if DEBUG {
            println!("Big partition has {} cuboids", small_partition.len());
        }
        cuboids += small_partition.len();
        let add: usize = small_partition.iter().filter(|c| Instruction::process(c, ins)).map(|c| c.volume()).sum();
        if DEBUG {
            println!("Big partition has volume {}", add);
        }
        total += add;
    }
    if DEBUG {
        println!("{} cuboids", cuboids);
    }
    println!("{} cubes on", total);
}

struct Instruction {
    direction: String,
    x0: isize,
    x1: isize,
    y0: isize,
    y1: isize,
    z0: isize,
    z1: isize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Cuboid{
    x0: isize,
    x1: isize,
    y0: isize,
    y1: isize,
    z0: isize,
    z1: isize,
}

impl Cuboid {
    fn new(t: [isize; 6]) -> Self {
        Self { x0: t[0], x1: t[1], y0: t[2], y1: t[3], z0: t[4], z1: t[5] }
    }

    fn intersects(&self, other: &Self) -> bool {
        ((self.x0..self.x1).contains(&other.x0) || (other.x0..other.x1).contains(&self.x0))
        && ((self.y0..self.y1).contains(&other.y0) || (other.y0..other.y1).contains(&self.y0))
        && ((self.z0..self.z1).contains(&other.z0) || (other.z0..other.z1).contains(&self.z0))
    }

    fn get(&self, dim: u8, index: u8) -> isize {
        match dim {
            0 => if index == 0 { self.x0 } else { self.x1 },
            1 => if index == 0 { self.y0 } else { self.y1 },
            2 => if index == 0 { self.z0 } else { self.z1 },
            _ => {unreachable!()},
        }
    }

    fn range(&self, dim: u8) -> Range<isize> {
        self.get(dim, 0)..self.get(dim, 1)
    }

    fn intersection(&self, other: &Self) -> Option<Self> {
        if self.intersects(other) {
            let mut coords = [0; 6];
            for d in 0..3 {
                let range = other.range(d);
                if range.contains(&self.get(d, 0)) {
                    coords[(2*d) as usize] = self.get(d, 0);
                } else {
                    coords[(2*d) as usize] = other.get(d, 0);
                }
                if range.contains(&self.get(d, 1)) {
                    coords[(2*d+1) as usize] = self.get(d, 1);
                } else {
                    coords[(2*d+1) as usize] = other.get(d, 1);
                }
            }
            Some(Cuboid::new(coords))
        } else {
            None
        }
    }

    fn volume(&self) -> usize {
        ((self.x1-self.x0)*(self.y1-self.y0)*(self.z1-self.z0)) as usize
    }

    fn get_bounds(cs: &Vec<&Self>) -> (Vec<isize>, Vec<isize>, Vec<isize>) {
        let mut vx = vec![];
        let mut vy = vec![];
        let mut vz = vec![];
        for c in cs {
            vx.push(c.x0);
            vx.push(c.x1);
            vy.push(c.y0);
            vy.push(c.y1);
            vz.push(c.z0);
            vz.push(c.z1);
        }
        vx.sort_unstable();
        vx.dedup();
        vy.sort_unstable();
        vy.dedup();
        vz.sort_unstable();
        vz.dedup();
        (vx, vy, vz)
    }

    fn contains(&self, p: &(isize, isize, isize)) -> bool {
        (self.x0..self.x1).contains(&p.0)
        && (self.y0..self.y1).contains(&p.1)
        && (self.z0..self.z1).contains(&p.2)
    }

    fn disjoint_cover(cs: &Vec<&Self>) -> Vec<Self> {
        let mut v = vec![];
        let bounds = Self::get_bounds(cs);
        let iter = iproduct!(bounds.0.windows(2), bounds.1.windows(2), bounds.2.windows(2));
        for cds in iter {
            let corner = (cds.0[0], cds.1[0], cds.2[0]);
            if cs.iter().any(|c| c.contains(&corner)) {
                v.push(Cuboid::new([
                    cds.0[0], cds.0[1],
                    cds.1[0], cds.1[1],
                    cds.2[0], cds.2[1],
                ]));
            }
        }
        v
    }

    fn get_point(&self) -> (isize, isize, isize) {
        (self.x0, self.y0, self.z0)
    }

    fn intersects_part(&self, part: &[&Self]) -> bool {
        part.iter().any(|c| self.intersects(c))
    }

    fn partition_connected_cuboids<'a>(cs: &[&'a Self]) -> Vec<Vec<&'a Self>> {
        let mut r: Vec<Vec<&Cuboid>> = vec![];
        let iter = cs.iter();
        for c in iter {
            let mut members = r.iter_mut().filter(|p| c.intersects_part(p));
            if let Some(part) = members.next() {
                part.push(*c);
                for extra_part in members {
                    part.append(extra_part);
                }
            } else {
                r.push(vec![*c]);
            }
        }
        r
    }
}

impl Display for Cuboid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{},{},{},{},{}]", self.x0, self.x1, self.y0, self.y1, self.z0, self.z1)
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} x={}..{},y={}..{},z={}..{}",
            self.direction,
            self.x0, self.x1-1,
            self.y0, self.y1-1,
            self.z0, self.z1-1,
        )
    }
}

impl From<String> for Instruction {
    fn from(s: String) -> Self {
        let re: Regex = Regex::new(REGEX).unwrap();
        let caps = re.captures(&s).unwrap();
        Instruction{
            direction: caps[1].to_string(),
            x0: caps[2].parse::<isize>().unwrap(),
            x1: caps[3].parse::<isize>().unwrap()+1,
            y0: caps[4].parse::<isize>().unwrap(),
            y1: caps[5].parse::<isize>().unwrap()+1,
            z0: caps[6].parse::<isize>().unwrap(),
            z1: caps[7].parse::<isize>().unwrap()+1,
        }
    }
}

impl Instruction {
    fn cuboid(&self, bounded: bool) -> Cuboid {
        if bounded {
            self.cuboid_bounded()
        } else {
            self.cuboid_unbounded()
        }
    }

    fn cuboid_unbounded(&self) -> Cuboid {
        Cuboid::new([
            self.x0, self.x1,
            self.y0, self.y1,
            self.z0, self.z1,
        ])
    }

    fn cuboid_bounded(&self) -> Cuboid {
        Cuboid::new([
            self.x0.max(-50).min(51), self.x1.max(-50).min(51),
            self.y0.max(-50).min(51), self.y1.max(-50).min(51),
            self.z0.max(-50).min(51), self.z1.max(-50).min(51),
        ])
    }

    fn on(&self, pos: (isize, isize, isize), current: bool) -> bool {
        if (self.x0..self.x1).contains(&pos.0)
        && (self.y0..self.y1).contains(&pos.1)
        && (self.z0..self.z1).contains(&pos.2) {
            self.direction == "on"
        } else {
            current
        }
    }

    fn process(c: &Cuboid, ins: &Vec<Instruction>) -> bool {
        let mut on = false;
        let point = c.get_point();
        for i in ins {
            on = i.on(point, on);
        }
        on
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        let s = "on x=1..3,y=-20..40,z=-5..-3";
        let i = Instruction{
            direction: "on".to_string(),
            x0: 1, x1: 4,
            y0: -20, y1: 41,
            z0: -5, z1: -2,
        };
        assert_eq!(i.to_string(), s);
    }
    #[test]
    fn parse_instruction() {
        let s = "on x=1..3,y=-20..40,z=-5..-3".to_string();
        let i = Instruction::from(s);
        assert_eq!(i.direction, "on");
        assert_eq!(i.x0, 1);
        assert_eq!(i.x1, 4);
        assert_eq!(i.y0, -20);
        assert_eq!(i.y1, 41);
        assert_eq!(i.z0, -5);
        assert_eq!(i.z1, -2);
    }

    #[test]
    fn cuboid_volume() {
        let c = Cuboid::new([1, 4, -20, 41, -5, -2]);
        assert_eq!(c.volume(), 9*61);
    }

    #[test]
    fn cuboids_intersect() {
        let c1 = Cuboid::new([0, 1, 0, 1, 0, 1]);
        let c2 = Cuboid::new([0, 1, 2, 3, 0, 1]);
        assert!(!c1.intersects(&c2));
        assert!(!c2.intersects(&c1));
        let c3 = Cuboid::new([0, 1, 0, 2, 0, 1]);
        assert!(c1.intersects(&c3));
        assert!(c3.intersects(&c1));
        let c4 = Cuboid::new([-46, 8, -6, 47, -50, 1]);
        let c5 = Cuboid::new([-48, -31, 26, 42, -47, -36]);
        assert!(c5.intersects(&c4));
        assert!(c4.intersects(&c5));
        let c6 = Cuboid::new([0,4,2,6,2,6]);
        let c7 = Cuboid::new([0,4,0,4,0,4]);
        assert!(c6.intersects(&c7));
        assert!(c7.intersects(&c6));
    }

    #[test]
    fn cuboid_bounds() {
        {
            let c1 = Instruction::from("on x=0..2,y=0..2,z=0..2".to_string()).cuboid(true);
            let c2 = Instruction::from("off x=1..3,y=1..3,z=1..3".to_string()).cuboid(true);
            let v: (Vec<isize>, Vec<isize>, Vec<isize>) = Cuboid::get_bounds(&vec![&c1, &c2]);
            assert_eq!(v, (vec![0,1,3,4,], vec![0,1,3,4,], vec![0,1,3,4,],));
        }
        {
            let c1 = Instruction::from("on x=0..2,y=0..2,z=0..3".to_string()).cuboid(true);
            let c2 = Instruction::from("off x=1..3,y=1..3,z=1..3".to_string()).cuboid(true);
            let v: (Vec<isize>, Vec<isize>, Vec<isize>) = Cuboid::get_bounds(&vec![&c1, &c2]);
            assert_eq!(v, (vec![0,1,3,4,], vec![0,1,3,4,], vec![0,1,4,],));
        }
    }

    #[test]
    fn on() {
        let i1 = Instruction::from("on x=0..3,y=0..3,z=0..3".to_string());
        let i2 = Instruction::from("off x=2..5,y=2..5,z=2..5".to_string());
        assert!(i1.on((0,0,0), true));
        assert!(i1.on((0,0,0), false));
        assert!(i1.on((4,4,4), true));
        assert!(!i1.on((4,4,4), false));
        assert!(i2.on((0,0,0), true));
        assert!(!i2.on((0,0,0), false));
        assert!(!i2.on((4,4,4), true));
        assert!(!i2.on((4,4,4), false));
        let i3 = Instruction::from("on x=-14..36,y=-6..44,z=-16..29".to_string());
        let point: (isize, isize, isize) = (26,40,-2);
        assert!(i3.on(point, false));
    }

    #[test]
    fn cuboid_partition() {
        let c1 = Instruction::from("on x=0..3,y=0..3,z=0..3".to_string()).cuboid(true);
        let c2 = Instruction::from("off x=0..3,y=2..5,z=2..5".to_string()).cuboid(true);
        let c3 = Instruction::from("off x=6..9,y=2..5,z=2..5".to_string()).cuboid(true);
        let v: Vec<Cuboid> = Cuboid::disjoint_cover(&vec![&c1, &c2, &c3]);
        assert_eq!(v.len(), 11);
    }

    #[test]
    fn part_intersect() {
        {
            let i = Instruction::from("on x=0..3,y=0..3,z=0..3".to_string()).cuboid(true);
            let part = vec![
                &i,
            ];
            let c = Instruction::from("off x=0..3,y=2..5,z=2..5".to_string()).cuboid(true);
            assert!(i.intersects_part(&vec![&c]));
            assert!(c.intersects_part(&part));
        }
    }

    #[test]
    fn instruction_process() {
        {
            let i = vec![
                Instruction::from("on x=-14..36,y=-6..44,z=-16..29".to_string()),
            ];
            let c: Cuboid = Cuboid::new([26,40,40,51,-2,12]);
            assert!(Instruction::process(&c, &i));
        }
    }

    #[test]
    fn cuboid_partition2() {
        {
            let c1 = Instruction::from("on x=0..1,y=0..1,z=0..1".to_string()).cuboid(true);
            let c2 = Instruction::from("off x=2..3,y=2..3,z=2..3".to_string()).cuboid(true);
            let c3 = Instruction::from("off x=4..5,y=4..5,z=4..5".to_string()).cuboid(true);
            let v: Vec<Vec<&Cuboid>> = Cuboid::partition_connected_cuboids(&[&c1, &c2, &c3]);
            assert_eq!(v.len(), 3);
        }
        {
            let c1 = Instruction::from("on x=0..3,y=0..3,z=0..3".to_string()).cuboid(true);
            let c2 = Instruction::from("off x=0..3,y=2..5,z=2..5".to_string()).cuboid(true);
            let c3 = Instruction::from("off x=6..9,y=2..5,z=2..5".to_string()).cuboid(true);
            let v: Vec<Vec<&Cuboid>> = Cuboid::partition_connected_cuboids(&[&c1, &c2, &c3]);
            for p in &v {
                println!("Partition");
                for c in p {
                    println!("{}", c);
                }
            }
            assert_eq!(v.len(), 2);
        }
        {
            let c1 = Instruction::from("on x=0..3,y=0..3,z=0..3".to_string()).cuboid(true);
            let c2 = Instruction::from("off x=0..3,y=2..5,z=2..5".to_string()).cuboid(true);
            let c3 = Instruction::from("off x=3..5,y=2..5,z=2..5".to_string()).cuboid(true);
            let v: Vec<Vec<&Cuboid>> = Cuboid::partition_connected_cuboids(&[&c1, &c2, &c3]);
            assert_eq!(v.len(), 1);
        }
        {
            let c1 = Instruction::from("off x=26..39,y=40..50,z=-2..11".to_string()).cuboid(true);
            let c2 = Instruction::from("on x=-14..36,y=-6..44,z=-16..29".to_string()).cuboid(true);
            let v: Vec<Vec<&Cuboid>> = Cuboid::partition_connected_cuboids(&[&c1, &c2]);
            assert_eq!(v.len(), 1);
        }
    }

    #[test]
    fn big_on() {
        {
            let i1 = Instruction::from("on x=0..3,y=0..3,z=0..3".to_string());
            let i2 = Instruction::from("off x=0..3,y=2..5,z=2..5".to_string());
            let i3 = Instruction::from("off x=6..9,y=2..5,z=2..5".to_string());
            let v: Vec<Cuboid> = Cuboid::disjoint_cover(&vec![&i1.cuboid(true), &i2.cuboid(true), &i3.cuboid(true)]);
            assert!(Instruction::process(&v[0], &vec![i1, i2, i3]));
        }
        {
            let i1 = Instruction::from("on x=0..3,y=0..3,z=0..3".to_string());
            let i2 = Instruction::from("off x=0..3,y=2..5,z=2..5".to_string());
            let i3 = Instruction::from("off x=6..9,y=2..5,z=2..5".to_string());
            let cuboids = vec![i1.cuboid(true), i2.cuboid(true), i3.cuboid(true)];
            let ins = vec![i1, i2, i3];
            let v: Vec<Vec<&Cuboid>> = Cuboid::partition_connected_cuboids(&[&cuboids[0], &cuboids[1], &cuboids[2]]);
            assert!(Instruction::process(&v[0][0], &ins));
            assert!(!Instruction::process(&v[1][0], &ins));
        }
    }

    #[test]
    fn intersection() {
        let c1 = Cuboid::new([0,3,0,3,0,3]);
        let c2 = Cuboid::new([2,4,2,4,2,4]);
        let c3 = Cuboid::new([0,3,0,3,3,5]);
        let int1 = c1.intersection(&c2);
        let int2 = c1.intersection(&c3);
        assert_eq!(int1, Some(Cuboid::new([2,3,2,3,2,3])));
        assert_eq!(int2, None);
    }

}
