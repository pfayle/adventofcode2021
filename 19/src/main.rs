use std::cmp::Ordering;
use std::fmt::Display;
use std::io::{self, Read};
use geomath::prelude::Algebra;
use geomath::vector::Vector3;
use geomath::matrix::Matrix3;
use geomath::vector::consts::ZEROS_3;
use geomath::matrix::consts::EYE_3;

const OVERLAP_GUARANTEE: usize = 12;

fn main() {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).expect("Error reading stdin.");

    let scs: Vec<Scanner> = buf.split("\n\n").map(|s| Scanner::from(s.to_string())).collect();
    let (beacons, scanners) = collect_all_beacons_and_scanners(scs, OVERLAP_GUARANTEE);
    println!("{} beacons; max scanner distance {}", beacons.len(), max_distance(scanners));
}

// Find all 24 rotations of the cube:
// consider the face at the front of the scanner,
// rotate this to each of 6 faces, then rotate 3 times about the axis in the middle of that face.
fn rotations() -> Vec<Matrix3> {
    const X: Matrix3 = Matrix3 {
        xx: 1.0, xy: 0.0, xz: 0.0,
        yx: 0.0, yy: 0.0, yz: -1.0,
        zx: 0.0, zy: 1.0, zz: 0.0
    };
    const Y: Matrix3 = Matrix3 {
        xx: 0.0, xy: 0.0, xz: -1.0,
        yx: 0.0, yy: 1.0, yz: 0.0,
        zx: 1.0, zy: 0.0, zz: 0.0
    };
    const Z: Matrix3 = Matrix3 {
        xx: 0.0, xy: -1.0, xz: 0.0,
        yx: 1.0, yy: 0.0, yz: 0.0,
        zx: 0.0, zy: 0.0, zz: 1.0
    };
    vec![
        EYE_3,
        X, X*X, X*X*X,
        Z, Z*Y, Z*Y*Y, Z*Y*Y*Y,
        Z*Z, Z*Z*X, Z*Z*X*X, Z*Z*X*X*X,
        Z*Z*Z, Z*Z*Z*Y, Z*Z*Z*Y*Y, Z*Z*Z*Y*Y*Y,
        Y, Y*Z, Y*Z*Z, Y*Z*Z*Z,
        Y*Y*Y, Y*Y*Y*Z, Y*Y*Y*Z*Z, Y*Y*Y*Z*Z*Z,
    ]
}

// convenience wrapper for sorting vectors, allowing easy deduplication later.
#[derive(PartialEq, Debug)]
struct SortableVec3(Vector3);

impl SortableVec3 {
    fn unwrap(&self) -> Vector3 {
        self.0
    }

    fn distance(&self, other: &Self) -> f64 {
        (other.unwrap().x - self.unwrap().x).abs()
        + (other.unwrap().y - self.unwrap().y).abs()
        + (other.unwrap().z - self.unwrap().z).abs()
    }
}

impl Ord for SortableVec3 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for SortableVec3 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.x.partial_cmp(&other.0.x).unwrap()
        .then(self.0.y.partial_cmp(&other.0.y).unwrap())
        .then(self.0.z.partial_cmp(&other.0.z).unwrap()))
    }
}

impl Eq for SortableVec3 {}

impl Display for SortableVec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}, {}]", self.0.x, self.0.y, self.0.z)
    }
}

#[derive(PartialEq, Debug, Clone)]
struct Beacon {
    pos: Vector3,
}

impl Beacon {
    fn _transform(&self, matrix: &Matrix3, translation: &Vector3) -> Beacon {
        Beacon { pos: *matrix * self.pos + *translation }
    }
}

impl From<String> for Beacon {
    fn from(s: String) -> Self {
        let mut it = s.split(',').map(|coord| coord.parse::<f64>().unwrap());
        Beacon { pos: Vector3{
            x: it.next().unwrap(),
            y: it.next().unwrap(),
            z: it.next().unwrap()
        }}
    }
}

#[derive(PartialEq)]
struct Scanner {
    beacons: Vec<Beacon>,
}

impl Scanner {
    fn _transform(&self, matrix: &Matrix3, translation: &Vector3) -> Self {
        let beacons = self.beacons.iter().map(|b| b._transform(matrix, translation)).collect::<Vec<Beacon>>();
        Self{beacons}
    }
    fn _rotations(&self) -> Vec<Scanner> {
        let mut r = vec![];
        for rot in rotations() {
            r.push(self._transform(&rot, &ZEROS_3));
        }
        r
    }

    // TODO: this function contributes the bulk of the runtime;
    // look for a better method for finding overlaps.
    fn overlap(&self, other: &Self, number: usize) -> Option<Overlap> {
        for b1 in &self.beacons {
            for b2 in &other.beacons {
                for m in rotations() {
                    let translation = b1.pos - m * b2.pos;
                    let overlap = other.beacons.iter().map(|b| Beacon{pos: m * b.pos + translation})
                        .filter(|b| self.beacons.contains(b)).count();
                    if overlap >= number {
                        return Some(Overlap{translation, rotation: m});
                    }
                }
            }
        }
        None
    }

    fn _collect_beacons(&self, other: &Self, number: usize) -> Vec<SortableVec3> {
        let mut points: Vec<SortableVec3> = self.beacons.iter().map(|b| SortableVec3(b.pos)).collect();
        if let Some(ov) = self.overlap(other, number) {
            let mut pts = other.beacons.iter().map(|b| SortableVec3(ov.transform(&b.pos))).collect::<Vec<SortableVec3>>();
            points.append(&mut pts);
        }
        points.sort();
        points.dedup();
        points
    }
}

impl From<String> for Scanner {
    fn from(s: String) -> Self {
        let beacons = s.trim().split('\n').skip(1).map(|line| Beacon::from(line.to_string())).collect();
        Self{beacons}
    }
}

struct Overlap {
    translation: Vector3,
    rotation: Matrix3,
}

impl Overlap {
    fn chain(&self, other: &Self) -> Self {
        let translation = self.translation + self.rotation * other.translation;
        let rotation = self.rotation * other.rotation;
        Self{translation, rotation}
    }

    fn transform(&self, point: &Vector3) -> Vector3 {
        self.rotation * *point + self.translation
    }

    fn _inverse_transform(&self, point: &Vector3) -> Vector3 {
        self.rotation.inverse() * (*point - self.translation)
    }
}

fn collect_all_beacons_and_scanners(scs: Vec<Scanner>, number: usize) -> (Vec<SortableVec3>, Vec<SortableVec3>) {
    let mut scanner_locs: Vec<SortableVec3> = vec![];
    let mut points: Vec<SortableVec3> = vec![];
    let mut routes: Vec<(&Scanner, Overlap)> = vec![(&scs[0], scs[0].overlap(&scs[0], number).unwrap())];
    points.append(&mut scs[0].beacons.iter().map(|b| SortableVec3(b.pos)).collect::<Vec<SortableVec3>>());
    scanner_locs.push(SortableVec3(routes[0].1.translation));
    points.sort();
    while scs.len() > routes.len() {
        for sc in &scs {
            if routes.iter().map(|(s, _)| *s).any(|s| *s == *sc) {
                continue;
            }
            let mut new_routes = vec![];
            for (link, overlap) in &routes {
                if let Some(ov) = link.overlap(sc, number) {
                    let combined = overlap.chain(&ov);
                    scanner_locs.push(SortableVec3(combined.translation));
                    let mut origpts: Vec<SortableVec3> = sc.beacons.iter().map(|b| SortableVec3(combined.transform(&b.pos))).collect();
                    origpts.sort();
                    points.append(&mut origpts);
                    new_routes.push((sc, combined));
                    break;
                }
            }
            routes.append(&mut new_routes);
        }
    }
    points.sort();
    points.dedup();
    (points, scanner_locs)
}

fn max_distance(points: Vec<SortableVec3>) -> f64 {
    let mut max: Option<f64> = None;
    for m in 0..(points.len()-1) {
        for n in (m+1)..points.len() {
            let d = points[m].distance(&points[n]);
            if max.is_none() || d > max.unwrap() {
                max = Some(d);
            }
        }
    }
    max.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beacon_from_string() {
        let s = "1,-1,1".to_string();
        let b = Beacon::from(s);
        assert_eq!(b, Beacon{ pos: Vector3 { x: 1.0, y: -1.0, z: 1.0 }});
    }

    #[test]
    fn scanner_from_string() {
        let s = "--- scanner 0 ---\n\
            9,5,4\n\
            8,-1,3\n".to_string();
        let sc = Scanner::from(s);
        assert_eq!(sc.beacons, vec![
            Beacon{pos: Vector3 { x: 9.0, y: 5.0, z: 4.0 }},
            Beacon{pos: Vector3 { x: 8.0, y: -1.0, z: 3.0 }},
        ]);
    }

    #[test]
    fn check_rotations() {
        let v1 = Vector3{x:1.0, y:2.0, z:3.0};
        let mut rots1: Vec<SortableVec3> = rotations().iter().map(|r| SortableVec3(*r*v1)).collect();
        rots1.sort();
        rots1.dedup();
        assert_eq!(rots1.len(), 24);
        let v2 = Vector3{x:0.0, y:0.0, z:0.0};
        let mut rots2: Vec<SortableVec3> = rotations().iter().map(|r| SortableVec3(*r*v2)).collect();
        rots2.sort();
        rots2.dedup();
        assert_eq!(rots2.len(), 1);

        let sc = Scanner{beacons: vec![
            Beacon{pos: v1},
            Beacon{pos: v2},
        ]};
        let srots = sc._rotations();
        assert_eq!(srots.len(), 24);
        let target = Scanner{beacons: vec![
            Beacon{pos: Vector3 { x: -3.0, y: 2.0, z: 1.0 }},
            Beacon{pos: Vector3 { x: 0.0, y: 0.0, z: 0.0 }},
        ]};
        assert!(srots.contains(&target));
    }

    #[test]
    fn simple_scanners() {
        let sc1 = Scanner{ beacons: vec![
            Beacon::from("0,2,0".to_string()),
            Beacon::from("4,1,0".to_string()),
            Beacon::from("3,3,0".to_string()),
        ]};
        let sc2 = Scanner{ beacons: vec![
            Beacon::from("-1,-1,0".to_string()),
            Beacon::from("-5,0,0".to_string()),
            Beacon::from("-2,1,0".to_string()),
        ]};
        let ov = sc1.overlap(&sc2, 3);
        assert!(ov.is_some());
        assert_eq!(ov.unwrap().translation, Vector3{x:5.0, y:2.0, z:0.0});
        let ov2 = sc1.overlap(&sc2, 4);
        assert!(ov2.is_none());
    }

    // ^
    // 1> B B
    // .  . ^ B
    // .  < 2 .
    // .  . . .
    // .  . < 3 B
    // .  . X v
    fn three_scanners() -> Vec<Scanner> {
        let sc1 = Scanner{ beacons: vec![
            Beacon::from("1,0,0".to_string()),
            Beacon::from("2,0,0".to_string()),
            Beacon::from("3,-1,0".to_string()),
        ]};
        let sc2 = Scanner{ beacons: vec![
            Beacon::from("2,0,0".to_string()),
            Beacon::from("1,-1,0".to_string()),
            Beacon::from("-2,-2,0".to_string()),
            Beacon::from("2,1,0".to_string()),
        ]};
        let sc3 = Scanner{ beacons: vec![
            Beacon::from("0,-3,0".to_string()),
            Beacon::from("-1,0,0".to_string()),
            Beacon::from("1,-4,0".to_string()),
        ]};
        vec![sc1, sc2, sc3]
    }

    #[test]
    fn distant_scanners() {
        let sc = three_scanners();
        let ov2 = sc[0].overlap(&sc[1], 3);
        let ov3 = sc[0].overlap(&sc[2], 3);
        assert!(ov3.is_none());
        assert!(ov2.is_some());
        let ov23 = sc[1].overlap(&sc[2], 3);
        assert!(ov23.is_some());
        let ov2u = ov2.unwrap();
        let ov23u = ov23.unwrap();
        let big_trans = ov2u.translation + ov2u.rotation * ov23u.translation;
        assert_eq!(big_trans, Vector3{x: 3.0, y: -4.0, z: 0.0});
        let big_rot = ov2u.rotation * ov23u.rotation;
        assert_eq!(big_rot, Matrix3{
            xx: -1.0, xy: 0.0, xz: 0.0,
            yx: 0.0, yy: -1.0, yz: 0.0,
            zx: 0.0, zy: 0.0, zz: 1.0
        });
        let chained_overlap = ov2u.chain(&ov23u);
        assert_eq!(chained_overlap.translation, big_trans);
        assert_eq!(chained_overlap.rotation, big_rot);
        let point = Vector3{x: 2.0, y: -5.0, z: 0.0};
        let transformed = chained_overlap.transform(&point);
        assert_eq!(transformed, Vector3{x: 1.0, y: 1.0, z: 0.0});
        let point = Vector3{x: 1.0, y: 1.0, z: 0.0};
        let transformed = chained_overlap._inverse_transform(&point);
        assert_eq!(transformed, Vector3{x: 2.0, y: -5.0, z: 0.0});
    }

    #[test]
    fn collect_beacons() {
        let scs = three_scanners();
        let points = scs[0]._collect_beacons(&scs[1], 3);
        assert_eq!(points.len(), 4);
    }

    #[test]
    fn recursive_collect_beacons() {
        let scs = three_scanners();
        let (points, _) = collect_all_beacons_and_scanners(scs, 3);
        assert_eq!(points.len(), 4);
    }

    #[test]
    fn recursive_collect_scanners() {
        let scs = three_scanners();
        let (_, scanners) = collect_all_beacons_and_scanners(scs, 3);
        assert_eq!(scanners.len(), 3);
        assert_eq!(scanners, vec![
            SortableVec3(Vector3{x: 0.0, y: 0.0, z: 0.0}),
            SortableVec3(Vector3{x: 2.0, y: -2.0, z: 0.0}),
            SortableVec3(Vector3{x: 3.0, y: -4.0, z: 0.0}),
        ]);
    }

    #[test]
    fn manhattan_distance() {
        let (_, scanners) = collect_all_beacons_and_scanners(three_scanners(), 3);
        let distance = scanners[0].distance(&scanners[1]);
        assert_eq!(distance, 4.0);
        let max_distance = max_distance(scanners);
        assert_eq!(max_distance, 7.0);
    }
}
