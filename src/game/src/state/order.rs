use std::collections::VecDeque;

use crate::{state::world::World, Error, Hex, Result};

///  1. Where should orders live and execute? In game/state/order.rs only, or should execution touch MatchState/World/
///     ShipState as well?
///     -  Well this is not really clear at this point, keeping in mind that server.rs and client.rs are different binaries
///         and that they will run very different versions of /game (handled by GameBuilder in /game/api.rs). I was thinking
///         that the OrderIterators would exist on the stack, created by MathState implementation in resolution.rs. But the 
///         necessary information to create OrderIterator would exist in PlayerState  (for instance Action::Move(Hx, Hx)). So
///         I/O handles user input and validation then appends your order to your unique PlayerState.
/// 
/// 2. What is the intended ownership model for orders? For example: PlayerState owns Vec<Order> and each Order owns ship
///     id + actions.
///     -  An order is essentially just "move this hex over there", "purchase this" or "hold that position". And that is all they should intially own.
///         For instance, the order::Action::Move(hx, hx), comes from some known PlayerState into MatchState's resolve_turn() context. order.rs is responsible 
///         of validating it, and producing a finished iterable where calling next() makes work on the order. Specifically for Move, this might require pathfinding.
/// 
///  
/// 3. Do you want orders to mutate state directly via Iterator::next(), or should next() yield an “effect” (like
///     Move{ship_id, from, to}) that a resolver applies?
///     -  For now, I do not want them to alter the state. The main responsibility should be to create workable orders with a dynamic API. The reason is because
///         orders should be able to get interleaved whichever way we want.
/// 
/// 
/// 4. How should invalid actions behave (out of bounds, no stamina, missing ship)? Error on next(), skip, or abort the
///     whole order?
///     -  Everything that is not OK -> crate::Error, try and keep with general errors and not introducing new ones unnecessarily.
/// 
/// 
/// 5. Should chaining be explicit (like Order::new(ship).move_to(hx1).shoot(hx2)), or support concatenating multiple
///     Orders (like order_a.chain(order_b))?
///     - Chaining can, but doesnt have to be explicit. This type of system might be nice. The building, ".move_to(hx1).shoot(hx2)", should instantiatied
///         inside order.rs, not by the API. The API's specifics is up to you but I want it preferably to need as little as possible, preferably I would like it
///         if the API only accepted a single order::Action, which then gets expanded to some complicated OrderIterator.
/// 
///  
/// 6. What data structures exist or are acceptable for tracking per-ship movement range and stamina? We have ShipState
///     with stamina, but no movement system yet.
///     - The only way to know right now where ships are, is in MatchState's <world> field. That is just a wrapper for a Vec<Tile> and they hold a TypeID... Ok this might not be so good, perhaps review this as well if you think this is necessary. 

/// Context for materializing and validating orders.
/// Resolution can implement this over MatchState or any snapshot type.
pub trait OrderContext {
    fn ship_position(&self, ship_id: u32) -> Result<Hex>;
    fn ship_move_range(&self, ship_id: u32) -> Result<u8>;
}

impl OrderContext for World {
    fn ship_position(&self, ship_id: u32) -> Result<Hex> {
        self.ship_position(ship_id)
            .ok_or(Error::NoEntityAtIndex)
    }

    fn ship_move_range(&self, ship_id: u32) -> Result<u8> {
        self.ship_move_range(ship_id)
            .ok_or(Error::NoEntityAtIndex)
    }
}

/// Player-submitted order for a single ship.
#[derive(Debug, Clone)]
pub struct Order {
    ship_id: u32,
    actions: Vec<Action>,
}

impl Order {
    pub fn from_action(ship_id: u32, action: Action) -> Self {
        let mut actions = Vec::new();
        flatten_action(action, &mut actions);
        Self { ship_id, actions }
    }

    pub fn ship_id(&self) -> u32 {
        self.ship_id
    }

    pub fn iter<'a, C: OrderContext>(&'a self, ctx: &'a C) -> OrderIter<'a, C> {
        OrderIter::new(self.ship_id, &self.actions, ctx)
    }
}

/// Iterator of validated steps for an order.
pub struct OrderIter<'a, C: OrderContext> {
    ctx: &'a C,
    ship_id: u32,
    actions: VecDeque<Action>,
}

impl<'a, C: OrderContext> OrderIter<'a, C> {
    fn new(ship_id: u32, actions: &[Action], ctx: &'a C) -> Self {
        Self {
            ctx,
            ship_id,
            actions: actions.iter().cloned().collect(),
        }
    }
}

impl<'a, C: OrderContext> Iterator for OrderIter<'a, C> {
    type Item = Result<OrderStep>;

    fn next(&mut self) -> Option<Self::Item> {
        let action = self.actions.pop_front()?;
        Some(resolve_action(self.ctx, self.ship_id, action))
    }
}

/// Primitive unit of unchecked Action, unique for each player.
#[derive(Debug, Clone)]
pub enum Action {
    HoldPos(Hex),
    Move(Hex, Hex),
    Shoot(Hex),
    Chain(Vec<Action>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStep {
    Hold { ship_id: u32, at: Hex },
    Move { ship_id: u32, from: Hex, to: Hex },
    Shoot { ship_id: u32, target: Hex },
}

fn resolve_action<C: OrderContext>(ctx: &C, ship_id: u32, action: Action) -> Result<OrderStep> {
    match action {
        Action::HoldPos(at) => Ok(OrderStep::Hold { ship_id, at }),
        Action::Shoot(target) => Ok(OrderStep::Shoot { ship_id, target }),
        Action::Move(from, to) => {
            let current = ctx.ship_position(ship_id)?;
            if current != from {
                return Err(Error::IllegalOrder);
            }

            let max_range = ctx.ship_move_range(ship_id)?;
            let clamped = clamp_move(from, to, max_range)?;
            Ok(OrderStep::Move {
                ship_id,
                from,
                to: clamped,
            })
        }
        Action::Chain(_) => Err(Error::IllegalOrder),
    }
}

fn flatten_action(action: Action, out: &mut Vec<Action>) {
    match action {
        Action::Chain(actions) => {
            for action in actions {
                flatten_action(action, out);
            }
        }
        other => out.push(other),
    }
}

fn clamp_move(from: Hex, to: Hex, max_range: u8) -> Result<Hex> {
    let distance = from.distance(&to);
    if distance <= max_range as i32 {
        return Ok(to);
    }

    let mut current = from;
    for _ in 0..max_range {
        current = step_towards(current, to)?;
    }
    Ok(current)
}

fn step_towards(from: Hex, to: Hex) -> Result<Hex> {
    let mut best: Option<Hex> = None;
    let mut best_distance = i32::MAX;

    for neighbor in from.neighbors().into_iter().flatten() {
        let dist = neighbor.distance(&to);
        if dist < best_distance {
            best_distance = dist;
            best = Some(neighbor);
        }
    }

    best.ok_or(Error::IllegalOrder)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCtx {
        pos: Hex,
        range: u8,
    }

    impl OrderContext for TestCtx {
        fn ship_position(&self, _ship_id: u32) -> Result<Hex> {
            Ok(self.pos)
        }

        fn ship_move_range(&self, _ship_id: u32) -> Result<u8> {
            Ok(self.range)
        }
    }

    fn hx(q: i32, r: i32) -> Hex {
        Hex::try_from((q, r)).unwrap()
    }

    #[test]
    fn move_is_clamped_to_range() {
        let ctx = TestCtx {
            pos: hx(0, 0),
            range: 2,
        };
        let order = Order::from_action(7, Action::Move(hx(0, 0), hx(5, 0)));
        let step = order.iter(&ctx).next().unwrap().unwrap();

        assert_eq!(
            step,
            OrderStep::Move {
                ship_id: 7,
                from: hx(0, 0),
                to: hx(2, 0),
            }
        );
    }

    #[test]
    fn chain_flattens_and_iterates_in_order() {
        let ctx = TestCtx {
            pos: hx(0, 0),
            range: 1,
        };
        let order = Order::from_action(
            1,
            Action::Chain(vec![
                Action::Move(hx(0, 0), hx(3, 0)),
                Action::Shoot(hx(1, 0)),
            ]),
        );

        let mut iter = order.iter(&ctx);
        let first = iter.next().unwrap().unwrap();
        let second = iter.next().unwrap().unwrap();

        assert_eq!(
            first,
            OrderStep::Move {
                ship_id: 1,
                from: hx(0, 0),
                to: hx(1, 0),
            }
        );
        assert_eq!(second, OrderStep::Shoot { ship_id: 1, target: hx(1, 0) });
        assert!(iter.next().is_none());
    }
}
