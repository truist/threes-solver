use std::fmt;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use threes_simulator::game_state::{Card, GameState};

use super::wrappers::{
    few_empties_scaler::FewEmptiesScaler, moves_scaler::MovesScaler,
    value_booster_wrapper::ValueBoosterWrapper,
};

pub struct AlgoConfig {
    pub base: bool,
    pub time_positive: bool,
    pub time_negative: bool,

    pub boost_12: bool,
    pub time_positive_boost_12: bool,
    pub time_negative_boost_12: bool,

    pub boost_high: bool, // no need for time-based boost_high; high values only show up later

    pub boost_few_empties: bool, // "few empties" is essentially time-based
    pub boost_few_empties_12: bool,
}

pub trait ValueBooster: std::fmt::Debug + std::fmt::Display {
    fn boost_score_for(&self, score: f64, values: &[Card]) -> f64;
}

pub(crate) fn assert_value_booster_not_supported(
    caller: &Algos,
    value_booster: Option<&dyn ValueBooster>,
) {
    assert!(
        value_booster.is_none(),
        "{caller:?} does not support ValueBooster"
    );
}

pub trait Algo: std::fmt::Debug + std::fmt::Display + Send + Sync {
    fn score(
        &self,
        game_state: &Option<GameState>,
        value_booster: Option<&dyn ValueBooster>,
    ) -> f64;
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
    fn score(&self, game_state: &Option<GameState>, booster: Option<&dyn ValueBooster>) -> f64 {
        if let Some(game_state) = game_state {
            match self {
                Algos::Empties => self.empties(game_state, booster),
                Algos::Merges => self.merges(game_state, booster),
                Algos::NearlyMerges => self.nearly_merges(game_state, booster),
                Algos::Squeezes => self.squeezes(game_state, booster) * -1.0,
                Algos::HighWall => self.high_walls(game_state, booster),
                Algos::HighCorner => self.high_corners(game_state, booster),
                Algos::Monotones => self.monotones(game_state, booster),
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

                boost_12: false, // meaningless
                time_positive_boost_12: false,
                time_negative_boost_12: false,

                boost_high: false, // meaningless

                boost_few_empties: false, // self-referential
                boost_few_empties_12: false,
            },
            Algos::Merges => AlgoConfig {
                base: false,
                time_positive: false,
                time_negative: false,

                boost_12: false,
                time_positive_boost_12: false,
                time_negative_boost_12: false,

                boost_high: false,

                boost_few_empties: true,
                boost_few_empties_12: true,
            },
            Algos::NearlyMerges => AlgoConfig {
                base: false,
                time_positive: true,
                time_negative: false,

                boost_12: false,
                time_positive_boost_12: false,
                time_negative_boost_12: false,

                boost_high: true,

                boost_few_empties: false,
                boost_few_empties_12: false,
            },
            Algos::Squeezes => AlgoConfig {
                base: false,
                time_positive: false,
                time_negative: false,

                boost_12: false,
                time_positive_boost_12: false,
                time_negative_boost_12: true,

                boost_high: false,

                boost_few_empties: true,
                boost_few_empties_12: false,
            },
            Algos::HighWall => AlgoConfig {
                base: false,
                time_positive: true,
                time_negative: false,

                boost_12: false,               // what does this even mean?
                time_positive_boost_12: false, // what does this even mean?
                time_negative_boost_12: false, // what does this even mean?

                boost_high: true,

                boost_few_empties: false,
                boost_few_empties_12: false, // what does this even mean?
            },
            Algos::HighCorner => AlgoConfig {
                base: true,
                time_positive: false,
                time_negative: false,

                boost_12: false,               // what does this even mean?
                time_positive_boost_12: false, // what does this even mean?
                time_negative_boost_12: false, // what does this even mean?

                boost_high: false,

                boost_few_empties: false,
                boost_few_empties_12: false, // what does this even mean?
            },
            Algos::Monotones => AlgoConfig {
                base: false,
                time_positive: false,
                time_negative: true,

                boost_12: false,
                time_positive_boost_12: false,
                time_negative_boost_12: false,

                boost_high: false,

                boost_few_empties: false,
                boost_few_empties_12: false, // not supported
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
            all_algos.push(algo_box(scale(algo, true)));
        }
        if config.time_negative {
            all_algos.push(algo_box(scale(algo, false)));
        }

        if config.boost_12 {
            all_algos.push(algo_box(booster(algo, 1, 2, 2.0)));
            all_algos.push(algo_box(booster(algo, 1, 2, 3.0)));
            all_algos.push(algo_box(booster(algo, 1, 2, 4.0)));
        }
        if config.time_positive_boost_12 {
            all_algos.push(algo_box(scale(booster(algo, 1, 2, 2.0), true)));
            all_algos.push(algo_box(scale(booster(algo, 1, 2, 3.0), true)));
            all_algos.push(algo_box(scale(booster(algo, 1, 2, 4.0), true)));
        }
        if config.time_negative_boost_12 {
            all_algos.push(algo_box(scale(booster(algo, 1, 2, 2.0), false)));
            all_algos.push(algo_box(scale(booster(algo, 1, 2, 3.0), false)));
            all_algos.push(algo_box(scale(booster(algo, 1, 2, 4.0), false)));
        }

        if config.boost_high {
            all_algos.push(algo_box(booster(algo, 96, 6144, 2.0)));
            all_algos.push(algo_box(booster(algo, 96, 6144, 3.0)));
            all_algos.push(algo_box(booster(algo, 96, 6144, 4.0)));
        }

        if config.boost_few_empties {
            all_algos.push(algo_box(empties(algo)));
        }
        if config.boost_few_empties_12 {
            all_algos.push(algo_box(empties(booster(algo, 1, 2, 2.0))));
            all_algos.push(algo_box(empties(booster(algo, 1, 2, 3.0))));
            all_algos.push(algo_box(empties(booster(algo, 1, 2, 4.0))));
        }
    }

    all_algos
}

fn algo_box<A: Algo + 'static>(algo: A) -> Box<dyn Algo> {
    Box::new(algo) as Box<dyn Algo>
}
fn scale<A: Algo>(wrapped: A, positive: bool) -> MovesScaler<A> {
    MovesScaler { wrapped, positive }
}
fn booster<A: Algo>(
    wrapped: A,
    min_value_to_boost: Card,
    max_value_to_boost: Card,
    boost: f64,
) -> ValueBoosterWrapper<A> {
    ValueBoosterWrapper {
        wrapped,
        min_value_to_boost,
        max_value_to_boost,
        boost,
    }
}
fn empties<A: Algo>(wrapped: A) -> FewEmptiesScaler<A> {
    FewEmptiesScaler { wrapped }
}
