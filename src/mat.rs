#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Mat2D<T> {
    pub width: usize,
    pub height: usize,
    pub vec: Vec<T>,
}

impl<T> Mat2D<T>
where
    T: Copy,
{
    pub fn filled_with(value: T, width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            vec: vec![value; height * width],
        }
    }

    pub fn get(&self, index: (usize, usize)) -> T {
        assert!(index.0 < self.height, "row index out of bounds");
        assert!(index.1 < self.width, "column index out of bounds");
        let index = self.map_index(index);
        self.vec[index]
    }

    pub fn set(&mut self, index: (usize, usize), v: T) {
        assert!(index.0 < self.height, "row index out of bounds");
        assert!(index.1 < self.width, "column index out of bounds");
        let index = self.map_index(index);
        self.vec[index] = v;
    }

    fn map_index(&self, index: (usize, usize)) -> usize {
        index.0 + index.1 * self.width
    }
}
