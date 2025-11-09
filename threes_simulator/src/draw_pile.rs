use std::fmt;

use rng_util::{Rng, RngType, SliceRandom};

use crate::board_state::Card;

#[derive(Clone, PartialEq)]
pub enum DrawType {
    Regular(Card),
    Bonus([Card; 3]),
}

impl DrawType {
    pub fn unwrap_regular(&self) -> Card {
        match self {
            DrawType::Regular(card) => *card,
            DrawType::Bonus(_) => panic!("tried to unwrap_regular() a bonus draw"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrawPile {
    cards: Vec<Card>,
    bonus_cards: Vec<Card>,
}

impl DrawPile {
    pub fn initialize(rng: &mut RngType) -> Self {
        let cards = DrawPile::new_cards(rng);
        DrawPile {
            cards,
            bonus_cards: vec![],
        }
    }

    fn new_cards(rng: &mut RngType) -> Vec<Card> {
        let mut cards = vec![1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3];
        cards.shuffle(rng);
        cards
    }

    #[cfg(any(test, feature = "workspace_test"))]
    pub fn initialize_test_pile(cards: Vec<Card>) -> Self {
        DrawPile {
            cards,
            bonus_cards: vec![],
        }
    }

    pub fn draw(&mut self, rng: &mut RngType) -> DrawType {
        let drawn;
        if self.bonus_cards.len() >= 3 && rng.random_ratio(1, 21) {
            drawn = DrawType::Bonus(self.draw_bonus(rng));
        } else {
            drawn = DrawType::Regular(self.cards.pop().unwrap());

            if 0 == self.cards.len() {
                self.cards = DrawPile::new_cards(rng);
            }
        }

        drawn
    }

    fn draw_bonus(&mut self, rng: &mut RngType) -> [Card; 3] {
        let first = rng.random_range(0..=self.bonus_cards.len() - 3);
        self.bonus_cards[first..first + 3].try_into().unwrap()
    }

    pub fn new_high_card(&mut self, new_high_card: &Card) {
        if *new_high_card < 192 {
            return;
        }

        if self.bonus_cards.len() == 0 {
            self.bonus_cards.append(&mut vec![6, 12, 24]);
        }

        if self.bonus_cards[self.bonus_cards.len() - 1] < new_high_card / 8 {
            self.bonus_cards.push(new_high_card / 8);
        }
    }

    pub fn len(&self) -> (usize, usize) {
        (self.cards.len(), self.bonus_cards.len())
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

    use rng_util::test_rng;

    #[test]
    fn initialize() {
        let mut rng = test_rng();

        let main1 = DrawPile::initialize(&mut rng);
        assert_eq!(main1.len().0, 12, "main pile has 12 cards");

        for value in 1..=3 {
            let value_count = main1.cards.iter().filter(|each| **each == value).count();
            assert_eq!(4, value_count, "4 cards with value {value}");
        }

        let main2 = DrawPile::initialize(&mut rng);
        assert_ne!(main1.cards, main2.cards, "main piles are shuffled");
    }

    #[test]
    fn auto_reload() {
        let mut rng = test_rng();

        let mut pile = DrawPile::initialize(&mut rng);
        assert_eq!(pile.len().0, 12, "main pile has 12 cards");

        let pile_copy = pile.cards.clone();

        for _ in 0..11 {
            let _ = pile.draw(&mut rng);
        }
        assert_eq!(1, pile.len().0, "drawing 11 cards left 1 card in the pile");

        let _ = pile.draw(&mut rng);
        assert_eq!(
            12,
            pile.len().0,
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
        assert_eq!(3, pile.len().0, "drawing 9 cards left 3 cards in the pile");

        for _ in 0..4 {
            let _ = pile.draw(&mut rng);
        }
        assert_eq!(
            11,
            pile.len().0,
            "drawing 4 cards from a pile of three left a pile of 11"
        );
    }

    #[test]
    fn draw_bonus() {
        let mut rng = test_rng();

        let mut draw_pile = DrawPile::initialize(&mut rng);

        draw_pile.new_high_card(&3); // ok to call it with low values
        assert_eq!(0, draw_pile.bonus_cards.len(), "no cards in the bonus pile");

        draw_pile.new_high_card(&192);
        draw_pile.new_high_card(&192); // ok to call it multiple times with the same value
        assert_eq!(
            vec![6, 12, 24],
            draw_pile.bonus_cards,
            "the bonus pile was seeded with the base values"
        );
        assert_eq!(
            vec![6, 12, 24],
            draw_pile.draw_bonus(&mut rng),
            "with minimal bonus pile, we always draw the same set"
        );

        draw_pile.new_high_card(&(192 * 2));
        draw_pile.new_high_card(&(192 * 4));
        let (mut got_lower, mut got_mid, mut got_upper) = (false, false, false);
        for _ in 1..50 {
            let bonus_cards = draw_pile.draw_bonus(&mut rng);
            match bonus_cards {
                [6, 12, 24] => got_lower = true,
                [12, 24, 48] => got_mid = true,
                [24, 48, 96] => got_upper = true,
                _ => panic!("got an unexpected set of bonus cards: {bonus_cards:#?}"),
            }
            if got_lower && got_mid && got_upper {
                break; // optimization
            }
        }
        assert!(
            got_lower && got_mid && got_upper,
            "didn't get all expected cases, even after 50 tries: {got_lower}, {got_mid}, {got_upper}"
        );
    }

    #[test]
    fn draw_with_bonus() {
        let mut rng = test_rng();

        let mut draw_pile = DrawPile::initialize(&mut rng);

        draw_pile.new_high_card(&192);
        draw_pile.new_high_card(&(192 * 2));
        draw_pile.new_high_card(&(192 * 4));

        let mut got_bonus = false;
        for _ in 1..100 {
            match draw_pile.draw(&mut rng) {
                DrawType::Bonus(cards) => match cards {
                    [6, 12, 24] => got_bonus = true,
                    [12, 24, 48] => got_bonus = true,
                    [24, 48, 96] => got_bonus = true,
                    _ => panic!("got an unexpected set of bonus cards: {cards:#?}"),
                },
                DrawType::Regular(_) => (),
            }
            if got_bonus {
                break; // optimization
            }
        }
        assert!(got_bonus, "we got at least one bonus draw in 100 tries");
    }
}
