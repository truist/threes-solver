use strum::IntoEnumIterator;

use threes_simulator::game_state::Card;

use crate::algo::core::{Algo, Algos};
use crate::algo::wrappers::{
    few_empties_scaler::FewEmptiesScaler, moves_scaler::MovesScaler,
    value_booster_wrapper::ValueBoosterWrapper,
};

struct AlgoConfig {
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

// This seems like it should be a fn on Algo but I'm leaving it here because
// it gives us an easy way to toggle cases on and off all in one file.
fn default_config(algos: Algos) -> AlgoConfig {
    match algos {
        Algos::Empties => AlgoConfig {
            base: true,
            time_positive: true,
            time_negative: true,

            boost_12: false, // meaningless
            time_positive_boost_12: false,
            time_negative_boost_12: false,

            boost_high: false, // meaningless

            boost_few_empties: false, // self-referential
            boost_few_empties_12: false,
        },
        Algos::Merges => AlgoConfig {
            base: true,
            time_positive: true,
            time_negative: true,

            boost_12: true,
            time_positive_boost_12: true,
            time_negative_boost_12: true,

            boost_high: true,

            boost_few_empties: true,
            boost_few_empties_12: true,
        },
        Algos::NearlyMerges => AlgoConfig {
            base: true,
            time_positive: true,
            time_negative: true,

            boost_12: true,
            time_positive_boost_12: true,
            time_negative_boost_12: true,

            boost_high: true,

            boost_few_empties: true,
            boost_few_empties_12: true,
        },
        Algos::Squeezes => AlgoConfig {
            base: true,
            time_positive: true,
            time_negative: true,

            boost_12: true,
            time_positive_boost_12: true,
            time_negative_boost_12: true,

            boost_high: true,

            boost_few_empties: true,
            boost_few_empties_12: true,
        },
        Algos::HighWalls => AlgoConfig {
            base: true,
            time_positive: true,
            time_negative: true,

            boost_12: false,               // what does this even mean?
            time_positive_boost_12: false, // what does this even mean?
            time_negative_boost_12: false, // what does this even mean?

            boost_high: true, // seems redundant? (not really; it credits whatever is highest
            // at any given time. this just makes it count more for very-high cards.)
            boost_few_empties: true,
            boost_few_empties_12: false, // what does this even mean?
        },
        Algos::HighCorners => AlgoConfig {
            base: true,
            time_positive: true,
            time_negative: true,

            boost_12: false,               // what does this even mean?
            time_positive_boost_12: false, // what does this even mean?
            time_negative_boost_12: false, // what does this even mean?

            boost_high: false, // seems redundant? (not really; see above)

            boost_few_empties: true,
            boost_few_empties_12: false, // what does this even mean?
        },
        Algos::Monotones => AlgoConfig {
            base: true,
            time_positive: true,
            time_negative: true,

            boost_12: false,               // not supported
            time_positive_boost_12: false, // not supported
            time_negative_boost_12: false, // not supported

            boost_high: false, // not supported

            boost_few_empties: true,
            boost_few_empties_12: false, // not supported
        },
    }
}

pub fn build_all_algos() -> Vec<Box<dyn Algo>> {
    let mut all_algos: Vec<Box<dyn Algo>> = Vec::new();

    for algos in Algos::iter() {
        let config = default_config(algos);

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
