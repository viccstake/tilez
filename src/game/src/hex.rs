#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hex {
    pub q: i32,
    pub r: i32,
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

    /// Flat-top axial hex → world pixel position.
    /// `size` is the circumradius (center-to-corner distance).
    pub fn to_world(&self, size: f32) -> bevy::math::Vec2 {
        let x = size * 1.5 * self.q as f32;
        let y = size * (f32::sqrt(3.0) / 2.0 * self.q as f32 + f32::sqrt(3.0) * self.r as f32) + (self.r % 2) as f32;
        bevy::math::Vec2::new(x, y)
    }

    /// World pixel position → nearest flat-top hex (inverse of `to_world`).
    pub fn pixel_to_hex(pos: bevy::math::Vec2, size: f32) -> Hex {
        let q_frac = pos.x * 2.0 / 3.0 / size;
        let r_frac = (-pos.x + pos.y * f32::sqrt(3.0)) / (3.0 * size);
        Self::axial_round(q_frac, r_frac)
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
