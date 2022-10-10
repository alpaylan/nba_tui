use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Position {
    ANY,
    PG,
    SG,
    SF,
    PF,
    C,
    F,
    G,
    TALL,
    SHORT,
}

impl Position {
    pub fn does_position_belong(&self, group: &Self) -> bool {
        match self {
            Position::PG => [Position::PG, Position::G, Position::SHORT, Position::ANY].contains(group),
            Position::SG => [Position::SG, Position::G, Position::SHORT, Position::ANY].contains(group),
            Position::SF => [Position::SF, Position::F, Position::TALL, Position::ANY].contains(group),
            Position::PF => [Position::PF, Position::F, Position::TALL, Position::ANY].contains(group),
            Position::C => [Position::C, Position::TALL, Position::ANY].contains(group),
            _ => false
        }
    }

    pub fn get_all_positions() -> Vec<Position> {
        vec![
            Position::ANY,
            Position::PG,
            Position::SG,
            Position::SF,
            Position::PF,
            Position::C,
            Position::F,
            Position::G,
            Position::TALL,
            Position::SHORT,
        ]
    }
}

