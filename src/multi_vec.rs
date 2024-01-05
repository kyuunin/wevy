struct MultiVec<T> {
    w: i32,
    h: i32,
    data: Vec<T>
}

impl MultiVec<T> {
    fn new(val: T, w: i32, h: i32) -> Self {
        Self {
            w,
            h,
            data: vec![val; w * h]
        }
    }

    fn get(&self, x: i32, y: i32) -> Option<&T> {
        if x < 0 || x >= self.w || y < 0 || y >= self.h {
            None
        } else {
            Some(&self.data[x + y * self.w])
        }
    }

    fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut T> {
        if x < 0 || x >= self.w || y < 0 || y >= self.h {
            None
        } else {
            Some(&mut self.data[x + y * self.w])
        }
    }
}
