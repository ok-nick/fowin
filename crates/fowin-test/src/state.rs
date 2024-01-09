use std::collections::HashMap;

use fowin::{Position, Size};
use rand::Rng;

// TODO: can impl this without a hashmap
#[derive(Debug, Clone)]
pub struct State {
    properties: HashMap<PropertyKey, Property>,
}

impl State {
    pub fn random<R: Rng>(rng: &mut R) -> State {
        todo!()
    }

    pub fn set(&mut self, property: Property) -> Property {
        self.properties.insert(property.key(), property).unwrap()
    }

    pub fn get(&self, key: PropertyKey) -> &Property {
        self.properties.get(&key).unwrap()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PropertyKey {
    Title,
    Size,
    Position,
    Fullscreened,
    Hidden,
    AtFront,
    Focused,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Property {
    Title(String),
    Size(Size),
    Position(Position),
    Fullscreened(bool),
    Hidden(bool),
    AtFront(bool),
    Focused(bool),
}

impl Property {
    pub fn random<R: Rng>(rng: &mut R, key: PropertyKey) -> Property {
        match key {
            PropertyKey::Title => todo!(),
            PropertyKey::Size => todo!(),
            PropertyKey::Position => todo!(),
            PropertyKey::Fullscreened => todo!(),
            PropertyKey::Hidden => todo!(),
            PropertyKey::AtFront => todo!(),
            PropertyKey::Focused => todo!(),
        }
    }

    pub fn key(&self) -> PropertyKey {
        match self {
            Property::Title(_) => PropertyKey::Title,
            Property::Size(_) => PropertyKey::Size,
            Property::Position(_) => PropertyKey::Position,
            Property::Fullscreened(_) => PropertyKey::Fullscreened,
            Property::Hidden(_) => PropertyKey::Hidden,
            Property::AtFront(_) => PropertyKey::AtFront,
            Property::Focused(_) => PropertyKey::Focused,
        }
    }
}
