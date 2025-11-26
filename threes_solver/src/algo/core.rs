use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use threes_simulator::game_state::GameState;

use super::impls::{
    empties::empties,
    highs::{high_corners, high_walls},
    merges::{merges, nearly_merges},
    monotones::monotones,
    squeezes::squeezes,
};
use super::wrappers::MovesScaled;

pub struct AlgoConfig {
    pub base: bool,
    pub time_positive: bool,
    pub time_negative: bool,
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

pub trait Algo: std::fmt::Debug + Send + Sync {
    fn score(&self, game_state: &Option<GameState>) -> i8;
}

impl Algo for Algos {
    fn score(&self, game_state: &Option<GameState>) -> i8 {
        if let Some(game_state) = game_state {
            match self {
                Algos::Empties => empties(game_state) as i8,
                Algos::Merges => merges(game_state) as i8,
                Algos::NearlyMerges => nearly_merges(game_state) as i8,
                Algos::Squeezes => squeezes(game_state) as i8 * -1,
                Algos::HighWall => high_walls(game_state) as i8,
                Algos::HighCorner => high_corners(game_state) as i8,
                Algos::Monotones => monotones(game_state) as i8,
            }
        } else {
            0
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
            },
            Algos::Merges => AlgoConfig {
                base: true,
                time_positive: false,
                time_negative: false,
            },
            Algos::NearlyMerges => AlgoConfig {
                base: false,
                time_positive: true,
                time_negative: false,
            },
            Algos::Squeezes => AlgoConfig {
                base: true,
                time_positive: false,
                time_negative: false,
            },
            Algos::HighWall => AlgoConfig {
                base: false,
                time_positive: true,
                time_negative: false,
            },
            Algos::HighCorner => AlgoConfig {
                base: true,
                time_positive: false,
                time_negative: false,
            },
            Algos::Monotones => AlgoConfig {
                base: false,
                time_positive: false,
                time_negative: true,
            },
        }
    }
}

pub fn build_all_algos() -> Vec<Box<dyn Algo>> {
    let mut all_algos: Vec<Box<dyn Algo>> = Vec::new();

    for algo in Algos::iter() {
        let config = algo.default_config();

        if config.base {
            all_algos.push(Box::new(algo) as Box<dyn Algo>);
        }

        if config.time_positive {
            all_algos.push(Box::new(MovesScaled {
                wrapped: algo,
                positive: true,
            }) as Box<dyn Algo>);
        }

        if config.time_negative {
            all_algos.push(Box::new(MovesScaled {
                wrapped: algo,
                positive: false,
            }) as Box<dyn Algo>);
        }
    }

    all_algos
}
