#[allow(dead_code)]

use crate::{Hex, Result};

use std::collections::LinkedList;

pub trait Orderable {
    /// Return template
    ///  Some()     : If you did some mutating work
    ///  None       : If no work was done
    ///  crate::Error  : For invalid orders
    fn move_to(&mut self, pos: Hex) -> Result<Option<()>>;
    fn shoot_at(&mut self, pos: Hex) -> Result<Option<()>>;
    fn hold_pos(&mut self) -> Result<Option<()>>;
}

impl dyn Orderable {
    pub fn work(&mut self, order: PrimitiveAction) -> Result<Option<()>> {
        match order {
            PrimitiveAction::MoveTo(hx) => self.move_to(hx),
            PrimitiveAction::Shoot(hx) => self.shoot_at(hx),
            PrimitiveAction::Hold => self.hold_pos(),
        }
    }
}

pub type Order = LinkedList<PrimitiveAction>;


#[derive(Clone, Copy)]
pub enum PrimitiveAction {
    MoveTo(Hex),
    Shoot(Hex),
    Hold,
}
