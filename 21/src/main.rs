use std::{fmt::{Display}, collections::HashMap, ops::Add, iter::Sum};
use std::{io, io::{Read}};

const PART1_GAME_LIMIT: usize = 1_000;
const PART2_GAME_LIMIT: usize = 21;
const BOARD_LIMITS: [usize; 2] = [1, 11];
const ROLLS_PER_TURN: usize = 3;
const DEBUG: bool = false;

fn main() {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).expect("Error reading input");
    let starts: Vec<usize> = buf.trim().lines().map(|line| line.chars().last().unwrap().to_digit(10).unwrap() as usize).collect();

    let mut game = Game::create(&[starts[0], starts[1]]);
    game.main_loop();

    let WinCount(wins1, wins2) = wins(GameState {
        p1_score: 0,
        p2_score: 0,
        p1_rolls: ROLLS_PER_TURN,
        p1_position: starts[0],
        p2_position: starts[1]
    }, &mut HashMap::new());
    println!("Player 1 wins: {}; player 2 wins: {}", wins1, wins2);
    println!("Player {} wins in more universes", if wins1 > wins2 {"1"} else {"2"});
}

#[derive(Clone)]
struct WinCount(u64, u64);

impl Add for WinCount {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0+rhs.0, self.1+rhs.1)
    }
}

impl Sum for WinCount {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|a, wc| a+wc).unwrap()
    }
}

impl Display for WinCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0, self.1)
    }
}

impl WinCount {
    fn reverse(&self) -> Self {
        Self(self.1, self.0)
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct GameState {
    p1_score: usize,
    p2_score: usize,
    p1_rolls: usize,
    p1_position: usize,
    p2_position: usize,
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "{{scores:[{},{}]; rolls left: {}; positions: [{},{}]}}",
            self.p1_score, self.p2_score, self.p1_rolls, self.p1_position, self.p2_position
        )
    }
}

fn new_position(start: usize, mov: usize) -> usize {
    (start - BOARD_LIMITS[0] + mov) % (BOARD_LIMITS[1] - BOARD_LIMITS[0]) + BOARD_LIMITS[0]
}

fn wins(state: GameState, cache: &mut HashMap<GameState, WinCount>) -> WinCount {
    if let Some(x) = cache.get(&state) {
        x.clone()
    } else if state.p1_rolls == 0 {
        wins(GameState {
            p1_score: state.p2_score,
            p2_score: state.p1_score + state.p1_position,
            p1_rolls: ROLLS_PER_TURN,
            p1_position: state.p2_position,
            p2_position: state.p1_position
        }, cache).reverse()
    } else {
        if DEBUG {
            println!("Calculating wins for game state: {}", state);
        }
        let ret = match (state.p1_score, state.p2_score) {
            (PART2_GAME_LIMIT..,_) => WinCount(1, 0),
            (_,PART2_GAME_LIMIT..) => WinCount(0, 1),
            (_, _) => {
                (1..=3_usize).map(
                    |roll|
                    wins(GameState {
                        p1_score: state.p1_score,
                        p2_score: state.p2_score,
                        p1_rolls: state.p1_rolls - 1,
                        p1_position: new_position(state.p1_position, roll),
                        p2_position: state.p2_position
                    }, cache)
                ).sum()
            }
        };
        cache.insert(state.clone(), ret.clone());
        if DEBUG {
            println!("Wins for game state: {}: {}", state, &ret);
        }
        ret
    }
}

trait Die {
    fn roll(&mut self) -> usize;
    fn rolls(&self) -> usize;
}

struct DeterministicDie {
    next: usize,
    limit: usize,
    rolls: usize,
}

impl DeterministicDie {
    fn create(start: usize, limit: usize) -> Self {
        Self{
            next: start,
            limit,
            rolls: 0,
        }
    }
}

impl Die for DeterministicDie {
    fn roll(&mut self) -> usize {
        let ret = self.next;
        self.next =  (self.next + 1) % self.limit;
        self.rolls += 1;
        ret
    }
    fn rolls(&self) -> usize {
        self.rolls
    }
}

struct Player {
    id: usize,
    position: usize,
    score: usize,
}

impl Player {
    fn play(&mut self, die: &mut Box<dyn Die>) {
        let rolls: Vec<usize> = (0..ROLLS_PER_TURN).map(|_| die.roll()).collect();
        let result: usize = rolls.iter().sum();
        self.position = new_position(self.position, result);
        self.score += self.position;
        if DEBUG {
            println!("Player {} rolled {} and moved to space {} for a total score of {}",
                self.id, rolls.iter().map(|n| n.to_string()).reduce(|a, s| format!("{}+{}", a, s)).unwrap(),
                self.position, self.score
            );
        }
    }
}

struct Game {
    players: Vec<Player>,
    turn_player: usize,
    die: Box<dyn Die>,
    game_limit: usize,
}

impl Game {
    fn create(starts: &[usize; 2]) -> Self {
        Self {
            players: vec![
                Player{id:1, score: 0, position: starts[0]},
                Player{id:2, score: 0, position: starts[1]},
            ],
            turn_player: 0,
            die: Box::new(DeterministicDie::create(1, 100)),
            game_limit: PART1_GAME_LIMIT,
        }
    }
    fn main_loop(&mut self) {
        while !self.players.iter().any(|p| p.score >= self.game_limit) {
            self.players[self.turn_player].play(&mut self.die);
            self.turn_player = (self.turn_player + 1) % self.players.len();
        }
        println!("Game over! Game ended after {} die rolls", self.die.rolls());
        for player in &self.players {
            println!("Player {} has {}", player.id, player.score);
            if player.score < self.game_limit {
                println!("Player result is {}", player.score * self.die.rolls());
            }
        }
    }
}
