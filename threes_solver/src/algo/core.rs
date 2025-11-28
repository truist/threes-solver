use std::fmt;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use threes_simulator::game_state::{Card, GameState};

use super::wrappers::{MovesScaler, ValueScaler};

pub struct AlgoConfig {
    pub base: bool,
    pub time_positive: bool,
    pub time_negative: bool,

    pub boost_12: bool,
    pub time_positive_boost_12: bool,
    pub time_negative_boost_12: bool,

    pub boost_high: bool,
    //no need for time-based boost_high; high values only show up later
}

pub trait AlgoScaler: std::fmt::Debug + std::fmt::Display {
    fn scale_for(&self, game_state: &GameState, values: &[Card]) -> f64;
}

#[derive(Debug)]
pub struct AlgoScalers<'a> {
    // TODO does this need to be a Vec?
    pub scalers: Vec<&'a dyn AlgoScaler>,
}
impl<'a> AlgoScalers<'a> {
    pub fn scale_score(&self, mut score: f64, game_state: &GameState, values: &[Card]) -> f64 {
        for scaler in self.scalers.iter() {
            score *= scaler.scale_for(game_state, values);
        }
        score
    }
}

pub trait Algo: std::fmt::Debug + std::fmt::Display + Send + Sync {
    fn score(&self, game_state: &Option<GameState>, scalers: &AlgoScalers) -> f64;
}

#[derive(Clone, Copy, Debug, EnumIter)]
pub enum Algos {
    Empties,
    Merges,
    NearlyMerges,
    Squeezes,
    HighWall,
    HighCorner,
    Monotones,
}

impl Algo for Algos {
    fn score(&self, game_state: &Option<GameState>, scalers: &AlgoScalers) -> f64 {
        if let Some(game_state) = game_state {
            match self {
                Algos::Empties => self.empties(game_state, scalers),
                Algos::Merges => self.merges(game_state, scalers),
                Algos::NearlyMerges => self.nearly_merges(game_state, scalers),
                Algos::Squeezes => self.squeezes(game_state, scalers) * -1.0,
                Algos::HighWall => self.high_walls(game_state, scalers),
                Algos::HighCorner => self.high_corners(game_state, scalers),
                Algos::Monotones => self.monotones(game_state, scalers),
            }
        } else {
            0.0
        }
    }
}

impl Algos {
    // This gives us compiler guarantees that we haven't missed any cases,
    // and an easy way to toggle cases on and off.
    pub fn default_config(&self) -> AlgoConfig {
        match self {
            Algos::Empties => AlgoConfig {
                base: false,
                time_positive: true,
                time_negative: false,

                boost_12: false,
                time_positive_boost_12: false,
                time_negative_boost_12: false,

                boost_high: false,
            },
            Algos::Merges => AlgoConfig {
                base: true,
                time_positive: false,
                time_negative: false,

                boost_12: true,
                time_positive_boost_12: false,
                time_negative_boost_12: false,

                boost_high: true,
            },
            Algos::NearlyMerges => AlgoConfig {
                base: false,
                time_positive: true,
                time_negative: false,

                boost_12: false,
                time_positive_boost_12: false,
                time_negative_boost_12: false,

                boost_high: true,
            },
            Algos::Squeezes => AlgoConfig {
                base: true,
                time_positive: false,
                time_negative: false,

                boost_12: false,
                time_positive_boost_12: false,
                time_negative_boost_12: true,

                boost_high: false,
            },
            Algos::HighWall => AlgoConfig {
                base: false,
                time_positive: true,
                time_negative: false,

                boost_12: false,
                time_positive_boost_12: false,
                time_negative_boost_12: false,

                boost_high: true,
            },
            Algos::HighCorner => AlgoConfig {
                base: true,
                time_positive: false,
                time_negative: false,

                boost_12: false,
                time_positive_boost_12: false,
                time_negative_boost_12: false,

                boost_high: false,
            },
            Algos::Monotones => AlgoConfig {
                base: false,
                time_positive: false,
                time_negative: true,

                boost_12: false,
                time_positive_boost_12: false,
                time_negative_boost_12: false,

                boost_high: false,
            },
        }
    }
}

impl fmt::Display for Algos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn build_all_algos() -> Vec<Box<dyn Algo>> {
    let mut all_algos: Vec<Box<dyn Algo>> = Vec::new();

    for algo in Algos::iter() {
        let config = algo.default_config();

        if config.base {
            all_algos.push(algo_box(algo));
        }
        if config.time_positive {
            all_algos.push(algo_box(moves_scaler(algo, true)));
        }
        if config.time_negative {
            all_algos.push(algo_box(moves_scaler(algo, false)));
        }

        if config.boost_12 {
            all_algos.push(algo_box(value_scaler(algo, 1, 2, 2.0)));
        }
        if config.time_positive_boost_12 {
            all_algos.push(algo_box(moves_scaler(value_scaler(algo, 1, 2, 2.0), true)));
        }
        if config.time_negative_boost_12 {
            all_algos.push(algo_box(moves_scaler(value_scaler(algo, 1, 2, 2.0), false)));
        }

        if config.boost_high {
            all_algos.push(algo_box(value_scaler(algo, 96, 6144, 2.0)));
        }
    }

    all_algos
}

fn algo_box<A: Algo + 'static>(algo: A) -> Box<dyn Algo> {
    Box::new(algo) as Box<dyn Algo>
}
fn moves_scaler<A: Algo>(wrapped: A, positive: bool) -> MovesScaler {
    MovesScaler { positive }
}
fn value_scaler<A: Algo>(
    wrapped: A,
    min_value_to_scale: Card,
    max_value_to_scale: Card,
    scale: f64,
) -> ValueScaler {
    ValueScaler {
        min_value_to_scale,
        max_value_to_scale,
        scale,
    }
}
