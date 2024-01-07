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

pub struct RestVec<'a,T> {
    start: &'a mut [T],
    end: &'a mut [T],
    pub w: usize,
    pub h: usize,
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
    pub fn isolate<'a>(&'a mut self, x: usize, y: usize) -> Option<(&mut T, RestVec<'a, T>)> {
        self.get(x, y)?;
        let (start, rest) = self.data.split_at_mut(x + y * self.w);
        let (me, end) = rest.split_at_mut(1);
        Some(
            (
                &mut me[0],
                RestVec {
                    start: start,
                    end: end,
                    w: self.w,
                    h: self.h,
                }
            )
        )
    }
}

impl<'a, T>  RestVec<'a, T> {
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        if x >= self.w || y >= self.h {
            None
        } else {
            let index = x + y * self.w;
            if index == self.start.len() {
                return None
            }
            self.start.get(index).or_else(|| self.end.get(index - self.start.len() - 1))
        }
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x >= self.w || y >= self.h {
            None
        } else {
            let index = x + y * self.w;
            let len_start = self.start.len();
            if index == len_start {
                return None
            }
            self.start.get_mut(index).or_else(|| self.end.get_mut(index - len_start - 1))
        }
    }
}