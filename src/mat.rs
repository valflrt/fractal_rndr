use std::ops::{Index, IndexMut};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Mat2D<T> {
    pub width: usize,
    pub height: usize,
    pub vec: Vec<T>,
}

impl<T> Mat2D<T> {
    pub fn filled_with(value: T, width: usize, height: usize) -> Self
    where
        T: Clone,
    {
        Self {
            width,
            height,
            vec: vec![value; height * width],
        }
    }

    pub fn get(&self, index: (usize, usize)) -> Option<&T> {
        let index = self.map_index(index);
        if index < self.vec.len() {
            Some(&self.vec[index])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: (usize, usize)) -> Option<&mut T> {
        let index = self.map_index(index);
        if index < self.vec.len() {
            Some(&mut self.vec[index])
        } else {
            None
        }
    }

    #[inline]
    fn map_index(&self, index: (usize, usize)) -> usize {
        index.0 + index.1 * self.width
    }
}

impl<T> Index<(usize, usize)> for Mat2D<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| panic!("index {:?} out of bounds", index))
    }
}
impl<T> IndexMut<(usize, usize)> for Mat2D<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.get_mut(index)
            .unwrap_or_else(|| panic!("index {:?} out of bounds", index))
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

//     pub fn get(&self, index: (usize, usize, usize)) -> Option<&T> {
//         let index = self.map_index(index);
//         if index < self.vec.len() {
//             Some(&self.vec[index])
//         } else {
//             None
//         }
//     }

//     pub fn set(&mut self, index: (usize, usize, usize), v: T) -> Result<(), ()> {
//         let index = self.map_index(index);
//         if index < self.vec.len() {
//             self.vec[index] = v;
//             Ok(())
//         } else {
//             Err(())
//         }
//     }

//     #[inline]
//     fn map_index(&self, index: (usize, usize, usize)) -> usize {
//         index.0 + index.1 * self.width + index.2 * self.width * self.height
//     }
// }
