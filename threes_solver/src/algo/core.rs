use std::fmt;

use strum_macros::EnumIter;

use threes_simulator::game_state::GameState;

use crate::algo::impls::*;
pub(crate) use crate::algo::wrappers::value_booster_wrapper::ValueBooster;

pub trait Algo: fmt::Debug + Send + Sync {
    fn score(&self, game_state: &GameState, value_booster: Option<&dyn ValueBooster>) -> f64;

    fn normalization_factor(&self) -> f64;

    fn assert_value_booster_not_supported(&self, value_booster: Option<&dyn ValueBooster>) {
        assert!(
            value_booster.is_none(),
            "{self:?} does not support ValueBooster"
        );
    }

    // default behavior; overridden in the wrappers
    fn fmt_impl(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for dyn Algo + '_ {
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
}
