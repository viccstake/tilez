

use crate::state::*;
use crate::grid::*;


/// Simultaneous turn resolution.
///
/// For every `Move` order, the target hex is recorded by ship ID.
/// Ships with contested destinations (two or more ships heading to the
/// same hex) are blocked and remain in place.
/// Ships with no order, or with a `Hold` order, also stay put.
/// 

struct StateResolver<'a> {
    match_state: &'a mut MatchState,

}

impl MatchState {
    pub fn resolve_turn(&mut self) {
        for player in self.users() {

        }
    }
}









#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ship_cannot_displace_stationary_ship() {
        // ...
        // ...
    }
}
