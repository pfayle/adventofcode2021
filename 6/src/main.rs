use std::io;
use std::env;

const STARTING_TIMER: usize = 8;
const REFRESH_TIMER: usize = 6;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let days = args[1].parse::<usize>().unwrap();

    let mut fish: [i64; 9] = [0; STARTING_TIMER + 1];

    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    for n in buf.trim().split(",") {
        fish[n.parse::<usize>().unwrap()] += 1;
    }
    let mut today = 0;
    while today < days {
        iterate(&mut fish);
        today += 1;
    }
    println!("Number of fish: {}", fish.iter().sum::<i64>());
    Ok(())
}

fn iterate(fish: &mut [i64; STARTING_TIMER+1]) {
    let mut newfish: [i64; 9] = [0; STARTING_TIMER+1];
    for n in 0..REFRESH_TIMER {
        newfish[n] = fish[n+1];
    }
    newfish[REFRESH_TIMER] = fish[REFRESH_TIMER+1] + fish[0];
    for n in REFRESH_TIMER+1..STARTING_TIMER {
        newfish[n] = fish[n+1];
    }
    newfish[STARTING_TIMER] = fish[0];
    *fish = newfish;
}
