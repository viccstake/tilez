
use crate::Hex;

enum PrimitiveAction {
    MoveTo(usize, usize),
    Shoot(usize, usize),
    Hold,
}

trait Orderable {
    fn move_to(&mut self, pos: Hex);
    fn shoot_at(&mut self, pos: Hex);
    fn hold_pos(&mut self);    
}

trait Workable<T> {
    fn work(&mut self) -> Option<T>;
}

struct Order<E, A> {
    entity: E,
    action: A
}

impl From<(Entity, PrimitiveAction)> for Order<Entity, Action> {
    fn from(value: (Entity, PrimitiveAction)) -> Self {
        Self { entity: value.0, action: value.1 }
    }
}