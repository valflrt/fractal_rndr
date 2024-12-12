#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Mat<T> {
    pub width: usize,
    pub height: usize,
    pub vec: Vec<T>,
}

impl<T> Mat<T>
where
    T: Clone,
{
    pub fn filled_with(value: T, width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            vec: vec![value; height * width],
        }
    }

    pub fn get(&self, index: (usize, usize)) -> Result<&T, ()> {
        if index.0 < self.height || index.1 < self.width {
            let index = self.map_index(index);
            Ok(&self.vec[index])
        } else {
            Err(())
        }
    }

    pub fn set(&mut self, index: (usize, usize), v: T) -> Result<(), ()> {
        if index.0 < self.height || index.1 < self.width {
            let index = self.map_index(index);
            self.vec[index] = v;
            Ok(())
        } else {
            Err(())
        }
    }

    fn map_index(&self, index: (usize, usize)) -> usize {
        index.0 + index.1 * self.width
    }
}
