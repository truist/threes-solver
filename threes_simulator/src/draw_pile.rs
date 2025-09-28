use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use std::fmt;

use crate::board_state::Card;

#[derive(Clone, Debug, PartialEq)]
pub struct DrawPile {
    cards: Vec<Card>,
}

impl DrawPile {
    pub fn initialize(rng: &mut ThreadRng) -> Self {
        let cards = DrawPile::new_cards(rng);
        DrawPile { cards }
    }

    fn new_cards(rng: &mut ThreadRng) -> Vec<Card> {
        let mut cards = vec![1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3];
        cards.shuffle(rng);
        cards
    }

    #[cfg(test)]
    pub fn initialize_test_pile(cards: Vec<Card>) -> Self {
        DrawPile { cards }
    }

    pub fn draw(&mut self, rng: &mut ThreadRng) -> Card {
        let drawn = self.cards.pop().unwrap();

        if 0 == self.len() {
            self.cards = DrawPile::new_cards(rng);
        }

        drawn
    }

    pub fn len(&self) -> usize {
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
        assert_eq!(main1.len(), 12, "main pile has 12 cards");

        for value in 1..=3 {
            let value_count = main1.cards.iter().filter(|each| **each == value).count();
            assert_eq!(4, value_count, "4 cards with value {value}");
        }

        let main2 = DrawPile::initialize(&mut rng);
        assert_ne!(main1.cards, main2.cards, "main piles are shuffled");
    }

    #[test]
    fn auto_reload() {
        let mut rng = thread_rng();
        let mut pile = DrawPile::initialize(&mut rng);
        assert_eq!(pile.len(), 12, "main pile has 12 cards");

        let pile_copy = pile.cards.clone();

        for _ in 0..11 {
            let _ = pile.draw(&mut rng);
        }
        assert_eq!(1, pile.len(), "drawing 11 cards left 1 card in the pile");

        let _ = pile.draw(&mut rng);
        assert_eq!(
            12,
            pile.len(),
            "drawing the last card auto-refilled the pile"
        );

        for value in 1..=3 {
            let value_count = pile.cards.iter().filter(|each| **each == value).count();
            assert_eq!(4, value_count, "4 cards with value {value}");
        }

        assert_ne!(pile_copy, pile.cards, "New stack is shuffled differently");

        for _ in 0..9 {
            let _ = pile.draw(&mut rng);
        }
        assert_eq!(3, pile.len(), "drawing 9 cards left 3 cards in the pile");

        for _ in 0..4 {
            let _ = pile.draw(&mut rng);
        }
        assert_eq!(
            11,
            pile.len(),
            "drawing 4 cards from a pile of three left a pile of 11"
        );
    }
}
