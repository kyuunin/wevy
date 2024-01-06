#[derive(Debug, Clone)]
pub struct MultiVec<T> {
    pub w: usize,
    pub h: usize,
    pub data: Vec<T>
}

impl<T> Default for MultiVec<T> {
    fn default()->Self {
        MultiVec {
            w: 0,
            h: 0,
            data: Vec::new(),
        }
    }
}

impl<T> MultiVec<T> where T: Clone {
    pub fn new(val: T, w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            data: vec![val; w * h]
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        if x >= self.w || y >= self.h {
            None
        } else {
            Some(&self.data[x + y * self.w])
        }
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x >= self.w || y >= self.h {
            None
        } else {
            Some(&mut self.data[x + y * self.w])
        }
    }
}
