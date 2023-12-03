pub struct LinearInterpolationTable {
    table: Vec<(f64, f64)>,
}

impl LinearInterpolationTable {
    pub fn new(table: Vec<(f64, f64)>) -> Self {
        assert!(!table.is_empty());
        assert!(table.windows(2).all(|pair| pair[0].0 < pair[1].0));

        Self { table }
    }

    pub fn get(&self, x: f64) -> f64 {
        assert!(x.is_normal() || x == 0.0 || x == -0.0);
        assert!(x >= self.table.first().unwrap().0);
        assert!(x <= self.table.last().unwrap().0);

        match self
            .table
            .binary_search_by(|pair| pair.0.partial_cmp(&x).unwrap())
        {
            Ok(index) => self.table[index].1,
            Err(index) => {
                assert!(index > 0);
                assert!(index < self.table.len());
                let (key1, value1) = self.table[index - 1];
                let (key2, value2) = self.table[index];

                ((key2 - x) * value1 + (x - key1) * value2) / (key2 - key1)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LinearInterpolationTable;

    #[test]
    fn test() {
        let interpolation_table =
            LinearInterpolationTable::new(vec![(0.0, 1.0), (2.0, 2.0), (3.0, 0.0)]);
        assert!((interpolation_table.get(0.0) - 1.0).abs() < 1e-10);
        assert!((interpolation_table.get(2.0) - 2.0).abs() < 1e-10);
        assert!((interpolation_table.get(3.0) - 0.0).abs() < 1e-10);

        assert!((interpolation_table.get(1.0) - 1.5).abs() < 1e-10);
        assert!((interpolation_table.get(2.5) - 1.0).abs() < 1e-10);
        assert!((interpolation_table.get(0.5) - 1.25).abs() < 1e-10);
    }
}
