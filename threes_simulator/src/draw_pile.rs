use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use std::fmt;
use std::vec::Drain;

use crate::Card;

pub struct DrawPile {
    cards: Vec<Card>,
}

impl DrawPile {
    pub fn initialize(rng: &mut ThreadRng) -> DrawPile {
        let mut cards = vec![1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3];
        cards.shuffle(rng);
        DrawPile { cards }
    }

    pub fn draw(&mut self, count: usize) -> Drain<'_, Card> {
        self.cards.drain(self.cards.len() - count..)
    }

    #[cfg(test)]
    pub fn len(&mut self) -> usize {
        self.cards.len()
    }
}

impl fmt::Display for DrawPile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.cards)
    }
}

/************ tests *************/

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn initialize() {
        let mut rng = thread_rng();

        let main1 = DrawPile::initialize(&mut rng);
        assert_eq!(main1.cards.len(), 12, "main pile has 12 cards");

        for value in 1..=3 {
            let value_count = main1.cards.iter().filter(|each| **each == value).count();
            assert_eq!(4, value_count, "4 cards with value {value}");
        }

        let main2 = DrawPile::initialize(&mut rng);
        assert_ne!(main1.cards, main2.cards, "main piles are shuffled");
    }
}
