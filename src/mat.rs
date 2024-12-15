#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Mat2D<T> {
    pub width: usize,
    pub height: usize,
    pub vec: Vec<T>,
}

impl<T> Mat2D<T>
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

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Mat3D<T> {
//     pub width: usize,
//     pub height: usize,
//     pub depth: usize,
//     pub vec: Vec<T>,
// }

// impl<T> Mat3D<T>
// where
//     T: Clone,
// {
//     pub fn filled_with(value: T, width: usize, height: usize, depth: usize) -> Self {
//         Self {
//             width,
//             height,
//             depth,
//             vec: vec![value; height * width * depth],
//         }
//     }

//     pub fn get(&self, index: (usize, usize, usize)) -> Result<&T, ()> {
//         if index.0 < self.height || index.1 < self.width {
//             let index = self.map_index(index);
//             Ok(&self.vec[index])
//         } else {
//             Err(())
//         }
//     }

//     pub fn set(&mut self, index: (usize, usize, usize), v: T) -> Result<(), ()> {
//         if index.0 < self.height || index.1 < self.width {
//             let index = self.map_index(index);
//             self.vec[index] = v;
//             Ok(())
//         } else {
//             Err(())
//         }
//     }

//     fn map_index(&self, index: (usize, usize, usize)) -> usize {
//         index.0 + index.1 * self.width + index.2 * self.width * self.depth
//     }
// }
