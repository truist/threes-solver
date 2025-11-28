use std::fmt;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use threes_simulator::game_state::{Card, GameState};

use super::wrappers::{AlgoValueFilterWrapper, MovesScaled};

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

pub trait AlgoValueFilter: std::fmt::Debug + std::fmt::Display {
    fn filter_values(&self, values: &[Card]) -> bool;
}

pub(crate) fn assert_not_supported(caller: &Algos, value_filter: Option<&dyn AlgoValueFilter>) {
    if value_filter.is_some() {
        panic!("{caller:?} does not support AlgoValueFilter");
    }
}

pub trait Algo: std::fmt::Debug + std::fmt::Display + Send + Sync {
    fn score(
        &self,
        game_state: &Option<GameState>,
        value_filter: Option<&dyn AlgoValueFilter>,
    ) -> i8;
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
    fn score(&self, game_state: &Option<GameState>, filter: Option<&dyn AlgoValueFilter>) -> i8 {
        if let Some(game_state) = game_state {
            match self {
                Algos::Empties => self.empties(game_state, filter) as i8,
                Algos::Merges => self.merges(game_state, filter) as i8,
                Algos::NearlyMerges => self.nearly_merges(game_state, filter) as i8,
                Algos::Squeezes => self.squeezes(game_state, filter) as i8 * -1,
                Algos::HighWall => self.high_walls(game_state, filter) as i8,
                Algos::HighCorner => self.high_corners(game_state, filter) as i8,
                Algos::Monotones => self.monotones(game_state, filter) as i8,
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
            all_algos.push(algo_box(scale(algo, true)));
        }
        if config.time_negative {
            all_algos.push(algo_box(scale(algo, false)));
        }

        if config.boost_12 {
            all_algos.push(algo_box(filter(algo, vec![1, 2])));
        }
        if config.time_positive_boost_12 {
            all_algos.push(algo_box(scale(filter(algo, vec![1, 2]), true)));
        }
        if config.time_negative_boost_12 {
            all_algos.push(algo_box(scale(filter(algo, vec![1, 2]), false)));
        }

        if config.boost_high {
            all_algos.push(algo_box(filter(
                algo,
                vec![96, 192, 384, 768, 1536, 3072, 6144],
            )));
        }
    }

    all_algos
}

fn algo_box<A: Algo + 'static>(algo: A) -> Box<dyn Algo> {
    Box::new(algo) as Box<dyn Algo>
}
fn scale<A: Algo>(wrapped: A, positive: bool) -> MovesScaled<A> {
    MovesScaled { wrapped, positive }
}
fn filter<A: Algo>(wrapped: A, values_to_keep: Vec<Card>) -> AlgoValueFilterWrapper<A> {
    AlgoValueFilterWrapper {
        wrapped,
        values_to_keep,
    }
}
