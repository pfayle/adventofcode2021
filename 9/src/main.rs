use std::io;

fn main() -> io::Result<()> {
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
    let mut locs: Vec<Vec<i32>> = vec![];
    for line in lines {
        let mut loc: Vec<i32> = vec![];
        for c in line.trim().chars() {
            loc.push(c.to_digit(10).unwrap().try_into().unwrap());
        }
        locs.push(loc);
    }
    println!("Sum of low point risks: {}", low_sum(&locs));

    let mut world = World::init(&locs);
    world.visit_all_basins();
    let mut basins = world.basins;
    basins.sort();
    basins.reverse();
    println!("Product of largest 3 basins: {}", basins[0]*basins[1]*basins[2]);
    Ok(())
}

fn low_sum(locs: &Vec<Vec<i32>>) -> i32 {
    let mut r = 0;
    let (rows, cols) = (locs.len(), locs[0].len());
    for rn in 0..rows {
        for cn in 0..cols {
            let l = locs[rn][cn];
            if (rn == 0 || l < locs[rn-1][cn])
                && (rn == rows - 1 || l < locs[rn+1][cn])
                && (cn == 0 || l < locs[rn][cn-1])
                && (cn == cols - 1 || l < locs[rn][cn+1]) {
                r += l + 1;
            }
        }
    }
    r
}

struct World {
    size: (usize, usize),
    nodes: Vec<Vec<Node>>,
    basins: Vec<i32>,
}

impl World {
    fn init(map: &Vec<Vec<i32>>) -> Self {
        let rows = map.len();
        let cols = map[0].len();
        let mut nodes: Vec<Vec<Node>> = vec![];
        for rn in 0..rows {
            let mut row = vec![];
            for cn in 0..cols {
                let node = Node{
                    weight: map[rn][cn],
                    visited: false,
                };
                row.push(node);
            }
            nodes.push(row);
        }
        World{
            size: (rows, cols),
            nodes,
            basins: vec![],
        }
    }

    // visitor algorithm for basins
    // store all nodes and their neighbours
    // start at first node
    // each neighbour, if it's not already visited and not 9, add to basin; mark as visited regardless
    // repeat for each neighbour
    // no need to check further for end
    // move to next node
    // if visited/9, move to next node; if not, start new basin.
    // after moving through all nodes, check list of basins, find biggest 3, multiply together.
    fn visit_all_basins(self: &mut Self) {
        for rn in 0..self.size.0 {
            for cn in 0..self.size.1 {
                let basin = self.visit_basin((rn, cn));
                self.basins.push(basin);
            }
        }
    }

    fn visit_basin(self: &mut Self, location: (usize, usize)) -> i32 {
        let (rn, cn) = location;
        let mut node = &mut self.nodes[rn][cn];
        if node.visited || node.weight == 9 {
            return 0;
        }
        node.visited = true;
        let mut r = 1;
        for n_location in self.neighbours(rn, cn) {
            r += self.visit_basin(n_location);
        }
        r
    }

    fn neighbours(self: &Self, rn: usize, cn: usize) -> Vec<(usize, usize)> {
        let mut r = vec![];
        if rn > 0 {
            r.push((rn-1,cn));
        }
        if cn > 0 {
            r.push((rn,cn-1));
        }
        if rn+1 < self.size.0 {
            r.push((rn+1,cn));
        }
        if cn+1 < self.size.1 {
            r.push((rn,cn+1));
        }
        r
    }
}

struct Node {
    weight: i32,
    visited: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inner() {
        let mut locations = vec![vec![2; 10]; 5];
        locations[3][2] = 1;
        assert_eq!(low_sum(&locations), 2);
    }

    #[test]
    fn test_outer() {
        let mut locations = vec![vec![2; 10]; 5];
        locations[0][2] = 1;
        locations[4][2] = 1;
        locations[2][0] = 1;
        locations[2][9] = 1;
        assert_eq!(low_sum(&locations), 8);
    }

    #[test]
    fn test_corner() {
        let mut locations = vec![vec![2; 10]; 5];
        locations[0][0] = 1;
        locations[4][0] = 1;
        locations[0][9] = 1;
        locations[4][9] = 1;
        assert_eq!(low_sum(&locations), 8);
    }
}
