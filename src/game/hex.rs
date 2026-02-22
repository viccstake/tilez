#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
