use std::collections::HashMap;

use fowin::{Position, Size};
use rand::{distributions::Standard, prelude::Distribution, Rng};

// TODO: can impl this without a hashmap
#[derive(Debug, Clone)]
pub struct State {
    properties: HashMap<PropertyKind, Property>,
}

impl State {
    pub fn random<R: Rng>(rng: &mut R) -> State {
        todo!()
    }

    pub fn set(&mut self, property: Property) -> Property {
        self.properties.insert(property.kind(), property).unwrap()
    }

    pub fn get(&self, kind: PropertyKind) -> &Property {
        self.properties.get(&kind).unwrap()
    }

    pub fn diff(&self, other: &State) -> Vec<Property> {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PropertyKind {
    Title,
    Size,
    Position,
    Fullscreened,
    Hidden,
    AtFront,
    Focused,
}

impl PropertyKind {
    pub const ALL: [PropertyKind; 7] = [
        PropertyKind::Title,
        PropertyKind::Size,
        PropertyKind::Position,
        PropertyKind::Fullscreened,
        PropertyKind::Hidden,
        PropertyKind::AtFront,
        PropertyKind::Focused,
    ];

    pub fn constraints(&self) -> &'static [Property] {
        &[]
    }

    pub fn random<R: Rng>(&self, rng: &mut R) -> Property {
        match self {
            PropertyKind::Title => todo!(),
            PropertyKind::Size => todo!(),
            PropertyKind::Position => todo!(),
            PropertyKind::Fullscreened => todo!(),
            PropertyKind::Hidden => todo!(),
            PropertyKind::AtFront => todo!(),
            PropertyKind::Focused => todo!(),
        }
    }
}

impl Distribution<PropertyKind> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PropertyKind {
        match rng.gen_range(0..8) {
            0 => PropertyKind::Title,
            _ => PropertyKind::Focused,
        }
    }
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
    pub fn kind(&self) -> PropertyKind {
        match self {
            Property::Title(_) => PropertyKind::Title,
            Property::Size(_) => PropertyKind::Size,
            Property::Position(_) => PropertyKind::Position,
            Property::Fullscreened(_) => PropertyKind::Fullscreened,
            Property::Hidden(_) => PropertyKind::Hidden,
            Property::AtFront(_) => PropertyKind::AtFront,
            Property::Focused(_) => PropertyKind::Focused,
        }
    }
}
