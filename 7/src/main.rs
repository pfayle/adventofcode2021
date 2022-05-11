use std::io;
use std::env;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let cost_fn = &args[1];

    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let positions: Vec<i32> = buf.trim().split(",").map(|x| x.parse::<i32>().unwrap()).collect();

    let (&max, &min) = (positions.iter().max().unwrap(), positions.iter().min().unwrap());
    let mut best_cost: Option<i32> = None;
    let mut best_position: Option<i32> = None;
    for n in min..=max {
        let cost = cost(&positions, n, cost_fn);
        if let Some(x) = best_cost {
            if x < cost {
                continue;
            }
        }
        best_cost = Some(cost);
        best_position = Some(n);
    }
    println!("Best position: {}; fuel cost: {}", best_position.unwrap(), best_cost.unwrap());
    Ok(())
}

fn cost(positions: &Vec<i32>, align_to: i32, cost_fn: &str) -> i32 {
    if cost_fn == "one" {
        positions.iter().map(|x| i32::abs(x-align_to)).sum()
    } else {
        positions.iter().map(|x| tri(i32::abs(x-align_to))).sum()
    }
}

fn tri(n: i32) -> i32 {
    n*(n+1)/2
}
