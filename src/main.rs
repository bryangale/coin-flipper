use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use rayon::prelude::*;

#[derive(Clone, Copy, Debug)]
struct Result {
    alice_wins: u32,
    bob_wins: u32,
    ties: u32,
}

#[derive(Clone, Copy, Debug)]
enum GameResult {
    Empty,
    Single(FlipResult),
    Sequence(Sequence),
}

#[derive(Clone, Copy, Debug)]
struct Sequence {
    length: i32,
    first: FlipResult,
    last: FlipResult,
    alice_points: i32,
    bob_points: i32,
}

#[derive(Clone, Copy, Debug)]
enum FlipResult {
    Heads,
    Tails,
}

impl Distribution<FlipResult> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> FlipResult {
        match rng.gen::<bool>() {
            true => FlipResult::Heads,
            false => FlipResult::Tails,
        }
    }
}

fn main() {
    let game_count: u32 = 10000;
    let flip_count: u64 = 10000;

    let result = (0..game_count)
        .into_par_iter()
        .map(|_| {
            let result = game_result(flip_count);

            match result {
                GameResult::Empty => Result {
                    alice_wins: 0,
                    bob_wins: 0,
                    ties: 0,
                },
                GameResult::Single(_) => Result {
                    alice_wins: 0,
                    bob_wins: 0,
                    ties: 0,
                },
                GameResult::Sequence(sequence) => Result {
                    alice_wins: if sequence.alice_points > sequence.bob_points {
                        1
                    } else {
                        0
                    },
                    bob_wins: if sequence.alice_points < sequence.bob_points {
                        1
                    } else {
                        0
                    },
                    ties: if sequence.alice_points == sequence.bob_points {
                        1
                    } else {
                        0
                    },
                },
            }
        })
        .reduce(
            || Result {
                alice_wins: 0,
                bob_wins: 0,
                ties: 0,
            },
            |a, b| Result {
                alice_wins: a.alice_wins + b.alice_wins,
                bob_wins: a.bob_wins + b.bob_wins,
                ties: a.ties + b.ties,
            },
        );

    println!("{:#?}", result)
}

fn game_result(flip_count: u64) -> GameResult {
    (0..flip_count)
        .into_par_iter()
        .map_init(rand::thread_rng, |rng, _| {
            GameResult::Single(rng.gen::<FlipResult>())
        })
        .reduce(
            || GameResult::Empty,
            |a, b| match (a, b) {
                (GameResult::Empty, b) => b,
                (a, GameResult::Empty) => a,
                (GameResult::Single(a), GameResult::Single(b)) => {
                    GameResult::Sequence(combine_single(a, b))
                }
                (GameResult::Single(a), GameResult::Sequence(b)) => {
                    GameResult::Sequence(prepend_sequence(a, b))
                }
                (GameResult::Sequence(a), GameResult::Single(b)) => {
                    GameResult::Sequence(postpend_sequence(a, b))
                }
                (GameResult::Sequence(a), GameResult::Sequence(b)) => {
                    GameResult::Sequence(combine_sequence(a, b))
                }
            },
        )
}

fn combine_single(a: FlipResult, b: FlipResult) -> Sequence {
    Sequence {
        length: 2,
        first: a,
        last: b,
        alice_points: alice_wins(a, b),
        bob_points: bob_wins(a, b),
    }
}

fn prepend_sequence(a: FlipResult, b: Sequence) -> Sequence {
    Sequence {
        length: 1 + b.length,
        first: a,
        last: b.last,
        alice_points: alice_wins(a, b.first) + b.alice_points,
        bob_points: bob_wins(a, b.first) + b.bob_points,
    }
}

fn postpend_sequence(a: Sequence, b: FlipResult) -> Sequence {
    Sequence {
        length: a.length + 1,
        first: a.first,
        last: b,
        alice_points: a.alice_points + alice_wins(a.last, b),
        bob_points: a.bob_points + bob_wins(a.last, b),
    }
}

fn combine_sequence(a: Sequence, b: Sequence) -> Sequence {
    Sequence {
        length: a.length + b.length,
        first: a.first,
        last: b.last,
        alice_points: a.alice_points + b.alice_points + alice_wins(a.last, b.first),
        bob_points: a.bob_points + b.bob_points + bob_wins(a.last, b.first),
    }
}

fn alice_wins(a: FlipResult, b: FlipResult) -> i32 {
    match (a, b) {
        (FlipResult::Heads, FlipResult::Heads) => 1,
        _ => 0,
    }
}

fn bob_wins(a: FlipResult, b: FlipResult) -> i32 {
    match (a, b) {
        (FlipResult::Heads, FlipResult::Tails) => 1,
        _ => 0,
    }
}
