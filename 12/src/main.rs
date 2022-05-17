/*
    Assume:
    1. Big caves are never connected to other big caves.
    2. start and end may each only be visited once.

    Approach:
    * start at start node
    * loop through possible next steps, constructing paths incrementally and exhaustively
    * for each path, mark small caves when visited
*/
use std::io;
use std::collections::HashMap;
use std::env;

const DEBUG: bool = true;

fn main() {
    let args: Vec<String> = env::args().collect();
    let extra_visits = &args[1];

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
    let mut nodes = HashMap::new();
    let mut paths = vec![];
    for line in lines {
        let mut split = line.trim().split("-");
        let (n1, n2) = (split.next().unwrap().to_string(), split.next().unwrap().to_string());
        paths.push(Path{
            nodes: [n1.clone(), n2.clone()],
        });
        nodes.insert(n1.clone(), Node{
            id: n1.clone(),
            kind: Node::kind(n1.clone()),
            visits: 0,
        });
        nodes.insert(n2.clone(), Node{
            id: n2.clone(),
            kind: Node::kind(n2.clone()),
            visits: 0,
        });
    }
    let start = nodes.values().filter(|&n| {
        match n.kind {
            NodeType::Start => true,
            _ => false,
        }
    }).next().unwrap();
    // each path: length so far, current node, node states
    // current node
    // get unvisited neighbours of current node
    // if end, add 1 to total
    // if not end, mark as visited, make neighbour current node, and call current function recursively
    let pathcount = start.paths(&nodes, &paths, "start", extra_visits.parse::<i32>().unwrap());
    println!("Number of paths: {}", pathcount);
}

#[derive(Debug,Clone,Copy)]
enum NodeType {
    BigCave,
    SmallCave,
    Start,
    End,
}

struct Path {
    nodes: [String; 2],
}

#[derive(Debug)]
struct Node {
    id: String,
    kind: NodeType,
    visits: i32,
}

impl Node {
    fn paths(self: &Self, nodes: &HashMap<String, Node>, paths: &Vec<Path>, path: &str, extra_visits: i32) -> i32 {
        let mut r = 0;
        for (_, neighbour) in self.neighbours(&nodes, paths) {
            if let NodeType::SmallCave = neighbour.kind {
                if neighbour.visits > 0 && extra_visits == 0 {
                    continue;
                }
            }
            let mut next_nodes = HashMap::new();
            for (k, v) in nodes {
                if *k != neighbour.id.clone() {
                    next_nodes.insert(k.clone(), Node{
                        id: v.id.clone(),
                        kind: v.kind,
                        visits: v.visits,
                    });
                }
            }
            match neighbour.kind {
                NodeType::Start => {
                    continue;
                },
                NodeType::End => {
                    if DEBUG {
                        println!("{}-end", path);
                    }
                    r += 1;
                    continue;
                },
                NodeType::BigCave => {
                    next_nodes.insert(neighbour.id.clone(), Node {
                        id: neighbour.id.clone(),
                        kind: neighbour.kind,
                        visits: neighbour.visits + 1,
                    });
                    r += neighbour.paths(&next_nodes, paths, format!("{}-{}", path, neighbour.id).as_str(), extra_visits);
                },
                NodeType::SmallCave => {
                    next_nodes.insert(neighbour.id.clone(), Node {
                        id: neighbour.id.clone(),
                        kind: neighbour.kind,
                        visits: neighbour.visits + 1,
                    });
                    r += neighbour.paths(&next_nodes, paths, format!("{}-{}", path, neighbour.id).as_str(),
                        extra_visits - if neighbour.visits > 0 { 1 } else { 0 }
                    );
                },
            }
        }
        r
    }
    fn connected_to(self: &Self, other: &Self, paths: &Vec<Path>) -> bool {
        for path in paths {
            if path.nodes.contains(&self.id) && path.nodes.contains(&other.id) {
                return true;
            }
        }
        false
    }
    fn neighbours<'a>(self: &'a Self, nodes: &HashMap<String, Node>, paths: &Vec<Path>) -> HashMap<String, Node> {
        let mut r: HashMap<String, Node> = HashMap::new();
        for node in nodes.values() {
            if self.id == node.id {
                continue;
            }
            if self.connected_to(&node, paths) {
                r.insert(node.id.clone(), Node{
                    id: node.id.clone(),
                    kind: node.kind,
                    visits: node.visits,
                });
            }
        }
        r
    }
    fn kind(id: String) -> NodeType {
        if id == "start" {
            NodeType::Start
        } else if id == "end" {
            NodeType::End
        } else if id == id.to_uppercase() {
            NodeType::BigCave
        } else {
            NodeType::SmallCave
        }
    }
}
