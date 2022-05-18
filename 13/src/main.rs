/*
    convert fold x=a
    (all folds exactly in half)
    - y coords fixed
    - x coords x->2a-x if x>a
*/
use regex::Regex;
use std::io;

fn main() {
    let mut points: Vec<(i32, i32)> = vec![];
    let mut folds: Vec<(char, i32)> = vec![];
    let point_re: Regex = Regex::new(r"(?m)^(\d+),(\d+)$").unwrap();
    let fold_re: Regex = Regex::new(r"(?m)^fold along (x|y)=(\d+)$").unwrap();
    loop {
        let mut buf = String::new();
        let n = io::stdin().read_line(&mut buf);
        match n {
            Ok(0) => { break },
            Ok(_) => { parse_line(&buf, &point_re, &fold_re, &mut points, &mut folds); },
            Err(_) => { break; },
        }
    }

    let mut r = vec![];
    for fold in vec![folds[0]] {
        for point in &points {
            let p = point_transform(*point, fold);
            if !r.contains(&p) {
                r.push(p);
            }
        }
    }

    println!("Number of points: {}", r.len());

    for fold in folds {
        let mut next_points = vec![];
        for point in &points {
            let p = point_transform(*point, fold);
            if !next_points.contains(&p) {
                next_points.push(p);
            }
        }
        points = next_points;
    }

    display(points);
}

fn parse_line(line: &str, point_re: &Regex, fold_re: &Regex, points: &mut Vec<(i32, i32)>, folds: &mut Vec<(char, i32)>) {
    if let Some(caps) = point_re.captures(line) {
        points.push((caps[1].parse::<i32>().unwrap(), caps[2].parse::<i32>().unwrap()));
    } else if let Some(caps) = fold_re.captures(line) {
        folds.push((caps[1].chars().nth(0).unwrap(), caps[2].parse::<i32>().unwrap()));
    } else {
        //println!("Doesn't match");
    }
}

fn point_transform(point: (i32, i32), fold: (char, i32)) -> (i32, i32) {
    let mut r = point;
    if fold.0 == 'x' && point.0 > fold.1 {
        r.0 = 2*fold.1 - point.0;
    }
    if fold.0 == 'y' && point.1 > fold.1 {
        r.1 = 2*fold.1 - point.1;
    }
    r
}

fn display(points: Vec<(i32, i32)>) {
    let mut r = String::new();
    for y in 0..=points.iter().map(|(_, y1)| *y1).max().unwrap() {
        for x in 0..=points.iter().map(|(x1, _)| *x1).max().unwrap() {
            if points.contains(&(x, y)) {
                r.push('#');
            } else {
                r.push(' ');
            }
        }
        r.push('\n');
    }
    println!("{}", r);
}
