use soa_derive::StructOfArray;

pub const WIDTH: usize = 100;
pub const HEIGHT: usize = 60;


/// A trait for 2D coordinate systems (Hex, Cartesian, Polar, etc.)
pub trait TwoDCoordinate {
    type Component;

    fn new(x1: Self::Component, x2: Self::Component) -> Self;
    
    fn components(&self) -> (Self::Component, Self::Component);
}

#[derive(StructOfArray)]
#[soa_derive(Debug, PartialEq)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hex { pub q: i32, pub r: i32 }


#[derive(StructOfArray)]
#[soa_derive(Debug, PartialEq)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cartesian { pub x: f32, pub y: f32 }


#[derive(StructOfArray)]
#[soa_derive(Debug, PartialEq)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Polar { pub r: f32, pub theta: f32 }



impl TwoDCoordinate for Hex {
    type Component = i32;
    fn new(q: i32, r: i32) -> Self { Self { q, r } }
    fn components(&self) -> (i32, i32) { (self.q, self.r) }
}

impl TwoDCoordinate for Cartesian {
    type Component = f32;
    fn new(x: f32, y: f32) -> Self { Self { x, y } }
    fn components(&self) -> (f32, f32) { (self.x, self.y) }
}

impl TwoDCoordinate for Polar {
    type Component = f32;
    fn new(r: f32, theta: f32) -> Self { Self { r, theta } }
    fn components(&self) -> (f32, f32) { (self.r, self.theta) }
}




impl From<Polar> for Cartesian {
    fn from(p: Polar) -> Self {
        Self::new(p.r * p.theta.cos(), p.r * p.theta.sin())
    }
}
impl From<Cartesian> for Polar {
    fn from(c: Cartesian) -> Self {
        let r = (c.x.powi(2) + c.y.powi(2)).sqrt();
        let theta = c.y.atan2(c.x);
        Self::new(r, theta)
    }
}
impl From<Hex> for Cartesian {
    fn from(value: Hex) -> Self {
        return Self::new(value.q as f32, value.r as f32);
    }
}
impl From<Cartesian> for Hex {
    fn from(value: Cartesian) -> Self {
        return Self::axial_round(value.x, value.y);
    }
}



impl Hex {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn neighbors(&self) -> [Hex; 6] {
        [
            Hex::new(self.q + 1, self.r),
            Hex::new(self.q - 1, self.r),
            Hex::new(self.q, self.r + 1),
            Hex::new(self.q, self.r - 1),
            Hex::new(self.q + 1, self.r - 1),
            Hex::new(self.q - 1, self.r + 1),
        ]
    }

    fn axial_round(frac_q: f32, frac_r: f32) -> Hex {
        let frac_s = -frac_q - frac_r;
        let q = frac_q.round();
        let r = frac_r.round();
        let s = frac_s.round();
        let dq = (q - frac_q).abs();
        let dr = (r - frac_r).abs();
        let ds = (s - frac_s).abs();
        if dq > dr && dq > ds {
            Hex::new((-r - s) as i32, r as i32)
        } else if dr > ds {
            Hex::new(q as i32, (-q - s) as i32)
        } else {
            Hex::new(q as i32, r as i32)
        }
    }

    /// Hex grid distance in axial coordinates.
    pub fn distance(&self, other: &Hex) -> i32 {
        let dq = self.q - other.q;
        let dr = self.r - other.r;
        (dq.abs() + dr.abs() + (dq + dr).abs()) / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_hex_distance_zero() {
        let h = Hex::new(2, -3);
        assert_eq!(h.distance(&h), 0);
    }

    #[test]
    fn all_neighbors_are_distance_one() {
        let origin = Hex::new(0, 0);
        for n in origin.neighbors() {
            assert_eq!(
                origin.distance(&n),
                1,
                "neighbor {n:?} should be distance 1 from origin"
            );
        }
    }

    #[test]
    fn neighbors_returns_six_unique_hexes() {
        let h = Hex::new(3, -1);
        let ns = h.neighbors();
        assert_eq!(ns.len(), 6);
        let unique: std::collections::HashSet<_> = ns.iter().copied().collect();
        assert_eq!(unique.len(), 6, "duplicate neighbors found");
    }

    #[test]
    fn known_axial_distances() {
        let o = Hex::new(0, 0);
        assert_eq!(o.distance(&Hex::new(3, 0)), 3);
        assert_eq!(o.distance(&Hex::new(0, -3)), 3);
        assert_eq!(o.distance(&Hex::new(3, -3)), 3);
        assert_eq!(o.distance(&Hex::new(2, -1)), 2);
        assert_eq!(o.distance(&Hex::new(-1, -1)), 2);
    }

    #[test]
    fn distance_is_symmetric() {
        let a = Hex::new(2, -3);
        let b = Hex::new(-1, 4);
        assert_eq!(a.distance(&b), b.distance(&a));
    }

    #[test]
    fn neighbors_are_not_equal_to_origin() {
        let h = Hex::new(1, 1);
        for n in h.neighbors() {
            assert_ne!(n, h);
        }
    }
}
