use std::fmt;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use threes_simulator::game_state::{Card, GameState};

use crate::algo::impls::*;
use crate::algo::wrappers::{
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

pub trait ValueBooster: fmt::Debug + fmt::Display {
    fn boost_score_for(&self, score: f64, values: &[Card]) -> f64;
}

pub trait Algo: fmt::Debug + Send + Sync {
    fn score(&self, game_state: &GameState, value_booster: Option<&dyn ValueBooster>) -> f64;

    // TODO: unit tests for all the implementations
    fn normalization_factor(&self) -> f64;

    fn assert_value_booster_not_supported(&self, value_booster: Option<&dyn ValueBooster>) {
        assert!(
            value_booster.is_none(),
            "{self:?} does not support ValueBooster"
        );
    }

    fn fmt_impl(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for dyn Algo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_impl(f)
    }
}

#[derive(Clone, Copy, Debug, EnumIter)]
pub enum Algos {
    Empties,
    Merges,
    NearlyMerges,
    Squeezes,
    HighWalls,
    HighCorners,
    Monotones,
}

impl Algos {
    pub const ALGO_MAX_BASE: f64 = 24.0;

    pub fn to_algo(&self) -> Box<dyn Algo> {
        match self {
            Algos::Empties => Box::new(empties::Empties),
            Algos::Merges => Box::new(merges::Merges),
            Algos::NearlyMerges => Box::new(merges::NearlyMerges),
            Algos::Squeezes => Box::new(squeezes::Squeezes),
            Algos::HighWalls => Box::new(highs::HighWalls),
            Algos::HighCorners => Box::new(highs::HighCorners),
            Algos::Monotones => Box::new(monotones::Monotones),
        }
    }

    // This seems like it should be a fn on Algo but I'm leaving it here because
    // it gives us compiler guarantees that we haven't missed any cases,
    // and an easy way to toggle cases on and off all in one file.
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
                base: true,
                time_positive: false,
                time_negative: false,

                boost_12: false,
                time_positive_boost_12: true,
                time_negative_boost_12: false,

                boost_high: false,

                boost_few_empties: false,
                boost_few_empties_12: false,
            },
            Algos::NearlyMerges => AlgoConfig {
                base: false,
                time_positive: false,
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

                boost_12: true,
                time_positive_boost_12: false,
                time_negative_boost_12: false,

                boost_high: false,

                boost_few_empties: false,
                boost_few_empties_12: false,
            },
            Algos::HighWalls => AlgoConfig {
                base: false,
                time_positive: false,
                time_negative: false,

                boost_12: false,               // what does this even mean?
                time_positive_boost_12: false, // what does this even mean?
                time_negative_boost_12: false, // what does this even mean?

                boost_high: true, // seems redundant? (not really; it credits whatever is highest
                // at any given time. this just makes it count more for very-high cards.)
                boost_few_empties: false,
                boost_few_empties_12: false, // what does this even mean?
            },
            Algos::HighCorners => AlgoConfig {
                base: false,
                time_positive: false,
                time_negative: false,

                boost_12: false,               // what does this even mean?
                time_positive_boost_12: false, // what does this even mean?
                time_negative_boost_12: false, // what does this even mean?

                boost_high: false, // seems redundant? (not really; see above)

                boost_few_empties: false,
                boost_few_empties_12: false, // what does this even mean?
            },
            Algos::Monotones => AlgoConfig {
                base: false,
                time_positive: false,
                time_negative: false,

                boost_12: false,               // not supported
                time_positive_boost_12: false, // not supported
                time_negative_boost_12: false, // not supported

                boost_high: false, // not supported

                boost_few_empties: false,
                boost_few_empties_12: false, // not supported
            },
        }
    }
}

pub fn build_all_algos() -> Vec<Box<dyn Algo>> {
    let mut all_algos: Vec<Box<dyn Algo>> = Vec::new();

    for algos in Algos::iter() {
        let config = algos.default_config();

        if config.base {
            all_algos.push(algos.to_algo());
        }
        if config.time_positive {
            all_algos.push(scale(algos.to_algo(), true));
        }
        if config.time_negative {
            all_algos.push(scale(algos.to_algo(), false));
        }

        if config.boost_12 {
            // all_algos.push(booster(algos.to_algo(), 1, 2, 2.0));
            all_algos.push(booster(algos.to_algo(), 1, 2, 3.0));
            // all_algos.push(booster(algos.to_algo(), 1, 2, 4.0));
        }
        if config.time_positive_boost_12 {
            all_algos.push(scale(booster(algos.to_algo(), 1, 2, 2.0), true));
            // all_algos.push(scale(booster(algos.to_algo(), 1, 2, 3.0), true));
            // all_algos.push(scale(booster(algos.to_algo(), 1, 2, 4.0), true));
        }
        if config.time_negative_boost_12 {
            // all_algos.push(scale(booster(algos.to_algo(), 1, 2, 2.0), false));
            all_algos.push(scale(booster(algos.to_algo(), 1, 2, 3.0), false));
            // all_algos.push(scale(booster(algos.to_algo(), 1, 2, 4.0), false));
        }

        if config.boost_high {
            // all_algos.push(booster(algos.to_algo(), 96, 6144, 2.0));
            all_algos.push(booster(algos.to_algo(), 96, 6144, 3.0));
            // all_algos.push(booster(algos.to_algo(), 96, 6144, 4.0));
        }

        if config.boost_few_empties {
            all_algos.push(empties(algos.to_algo()));
        }
        if config.boost_few_empties_12 {
            all_algos.push(empties(booster(algos.to_algo(), 1, 2, 2.0)));
            // all_algos.push(empties(booster(algos.to_algo(), 1, 2, 3.0)));
            // all_algos.push(empties(booster(algos.to_algo(), 1, 2, 4.0)));
        }
    }

    all_algos
}

fn scale(wrapped: Box<dyn Algo>, positive: bool) -> Box<dyn Algo> {
    Box::new(MovesScaler { wrapped, positive })
}
fn booster(
    wrapped: Box<dyn Algo>,
    min_value_to_boost: Card,
    max_value_to_boost: Card,
    boost: f64,
) -> Box<dyn Algo> {
    Box::new(ValueBoosterWrapper {
        wrapped,
        min_value_to_boost,
        max_value_to_boost,
        boost,
    })
}
fn empties(wrapped: Box<dyn Algo>) -> Box<dyn Algo> {
    Box::new(FewEmptiesScaler { wrapped })
}
