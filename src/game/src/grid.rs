use soa_derive::StructOfArray;


use crate::{Error, Result};

pub const WIDTH: usize = 100;
pub const HEIGHT: usize = 60;


#[derive(StructOfArray)]
#[soa_derive(Debug, PartialEq)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hex {
    pub q: i32,
    pub r: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexUnchecked {
    pub q: i32,
    pub r: i32,
}

impl HexUnchecked {
    fn new(q: i32, r: i32) -> Self { Self { q, r }}

    pub fn validate(self) -> Option<Hex> {
        if (self.q.abs() as usize) < WIDTH/2 &&
           (self.r.abs() as usize) < HEIGHT/2
        {
            Some(Hex {
                q: self.q,
                r: self.r,
            })
        } else {
            None
        }
    }

    fn axial_round(frac_q: f32, frac_r: f32) -> Self {
        let frac_s = -frac_q - frac_r;
        let q = frac_q.round();
        let r = frac_r.round();
        let s = frac_s.round();
        let dq = (q - frac_q).abs();
        let dr = (r - frac_r).abs();
        let ds = (s - frac_s).abs();
        if dq > dr && dq > ds {
            Self::new((-r - s) as i32, r as i32)
        } else if dr > ds {
            Self::new(q as i32, (-q - s) as i32)
        } else {
            Self::new(q as i32, r as i32)
        }
    }


}

impl Hex {
    pub fn index(&self) -> usize {
        let q_shifted = (self.q + (WIDTH / 2) as i32) as usize;
        let r_shifted = (self.r + (HEIGHT / 2) as i32) as usize;
        r_shifted * WIDTH + q_shifted
    }
    pub fn neighbors(&self) -> [Option<Self>; 6] {
        [
            HexUnchecked::new(self.q + 1, self.r).validate(),
            HexUnchecked::new(self.q - 1, self.r).validate(),
            HexUnchecked::new(self.q, self.r + 1).validate(),
            HexUnchecked::new(self.q, self.r - 1).validate(),
            HexUnchecked::new(self.q + 1, self.r - 1).validate(),
            HexUnchecked::new(self.q - 1, self.r + 1).validate(),
        ]
    }
    pub fn distance(&self, other: &Hex) -> i32 {
        let dq = self.q - other.q;
        let dr = self.r - other.r;
        (dq.abs() + dr.abs() + (dq + dr).abs()) / 2
    }
}

impl TryFrom<Cartesian> for Hex {
    type Error = Error;
    fn try_from(value: Cartesian) -> Result<Self> {
        HexUnchecked::axial_round(value.x, value.y).try_into()
    }
}

impl TryFrom<(i32, i32)> for Hex {
    type Error = Error;
    fn try_from((x, y): (i32, i32)) -> Result<Self> {
        HexUnchecked::new(x,y).try_into()
    }
}

impl TryFrom<HexUnchecked> for Hex {
    type Error = Error;
    fn try_from(value: HexUnchecked) -> Result<Self> {
        value.validate().ok_or(Error::OutOfBounds)
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cartesian {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Polar {
    pub r: f32,
    pub theta: f32,
}

impl From<Polar> for Cartesian {
    fn from(p: Polar) -> Self {
        let x = p.r * p.theta.cos();
        let y = p.r * p.theta.sin();
        Self { x, y }
    }
}
impl From<Cartesian> for Polar {
    fn from(c: Cartesian) -> Self {
        let r = (c.x.powi(2) + c.y.powi(2)).sqrt();
        let theta = c.y.atan2(c.x);
        Self { r, theta }
    }
}
impl From<Hex> for Cartesian {
    fn from(value: Hex) -> Self {
        let x = value.q as f32;
        let y = value.r as f32;
        Self { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_hex(q: i32, r: i32) -> Hex { 
        Hex::try_from((q,r)).unwrap() 
    }

    #[test]
    fn same_hex_distance_zero() {
        let h: Hex = new_hex(2, -3);
        assert_eq!(h.distance(&h), 0);
    }

    #[test]
    fn all_neighbors_are_distance_one() {
        let origin = new_hex(0, 0);
        for n in origin.neighbors() {
            let neighbour = n.unwrap();
            assert_eq!(
                origin.distance(&neighbour),
                1,
                "neighbor {n:?} should be distance 1 from origin"
            );
        }
    }

    #[test]
    fn neighbors_returns_six_unique_hexes() {
        let h = new_hex(3, -1);
        let ns = h.neighbors();
        assert_eq!(ns.len(), 6);
        let unique: std::collections::HashSet<_> = ns.iter().copied().collect();
        assert_eq!(unique.len(), 6, "duplicate neighbors found");
    }

    #[test]
    fn known_axial_distances() {
        let o = new_hex(0, 0);
        assert_eq!(o.distance(&new_hex(3, 0)), 3);
        assert_eq!(o.distance(&new_hex(0, -3)), 3);
        assert_eq!(o.distance(&new_hex(3, -3)), 3);
        assert_eq!(o.distance(&new_hex(2, -1)), 2);
        assert_eq!(o.distance(&new_hex(-1, -1)), 2);
    }

    #[test]
    fn distance_is_symmetric() {
        let a = new_hex(2, -3);
        let b = new_hex(-1, 4);
        assert_eq!(a.distance(&b), b.distance(&a));
    }

    #[test]
    fn neighbors_are_not_equal_to_origin() {
        let h = new_hex(1, 1);
        for n in h.neighbors() {
            let n = n.unwrap();
            assert_ne!(n, h);
        }
    }
}
