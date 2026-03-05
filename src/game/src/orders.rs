use crate::{Hex, ShipState};

pub enum PrimitiveAction {
    MoveTo(Hex),
    Shoot(Hex),
    Hold,
}

pub enum Shoot {
    Melee,
    Ranged,
}

pub trait Orderable {
    type T;

    fn move_to(&mut self, pos: Hex) -> Option<Self::T>;
    fn shoot_at(&mut self, pos: Hex) -> Option<Self::T>;
    fn hold_pos(&mut self) -> Option<Self::T>;
}

impl<T> dyn Orderable<T = T> {
    pub fn work(&mut self, order: PrimitiveAction) -> Option<T> {
        match order {
            PrimitiveAction::MoveTo(hx) => self.move_to(hx),
            PrimitiveAction::Shoot(hx) => self.shoot_at(hx),
            PrimitiveAction::Hold => self.hold_pos(),
        }
    }
}

impl Orderable for ShipState {
    type T = ShipState;
    fn hold_pos(&mut self) -> Option<Self::T> {
        todo!()
    }
    fn move_to(&mut self, pos: Hex) -> Option<Self::T> {
        todo!()
    }
    fn shoot_at(&mut self, pos: Hex) -> Option<Self::T> {
        todo!()
    }
}
//
// ...
