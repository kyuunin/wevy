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

    pub fn new_like<U>(val: T, other: &MultiVec<U>) -> Self {
        Self {
            w: other.w,
            h: other.h,
            data: vec![val; other.w * other.h]
        }
    }

    pub fn checked_xy_to_index(x: usize, y: usize, w: usize, h: usize) -> Option<usize> {
        if x >= w || y >= h {
            None
        } else {
            Some(x + y * w)
        }
    }

    pub fn checked_index_to_xy(index: usize, w: usize, h: usize) -> Option<(usize, usize)> {
        if index >= w * h {
            None
        } else {
            Some((index % w, index / w))
        }
    }

    pub fn xy_to_index(&self, x: usize, y: usize) -> Option<usize> {
        Self::checked_xy_to_index(x, y, self.w, self.h)
    }

    pub fn index_to_xy(&self, index: usize) -> Option<(usize, usize)> {
        Self::checked_index_to_xy(index, self.w, self.h)
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        let i = self.xy_to_index(x, y)?;
        Some(&self.data[i])
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        let i = self.xy_to_index(x, y)?;
        Some(&mut self.data[i])
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut()
    }

    pub fn enum_iter(&self) -> impl Iterator<Item = (usize, usize, &T)> {
        let (w, h) = (self.w, self.h);
        self.data.iter().enumerate().map(move |(i, t)| {
            let (x, y) = Self::checked_index_to_xy(i, w, h).expect("index out of bounds");
            (x, y, t)
        })
    }

    pub fn enum_iter_mut(&mut self) -> impl Iterator<Item = (usize, usize, &mut T)> {
        let (w, h) = (self.w, self.h);
        self.data.iter_mut().enumerate().map(move |(i, t)| {
            let (x, y) = Self::checked_index_to_xy(i, w, h).expect("index out of bounds");
            (x, y, t)
        })
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
