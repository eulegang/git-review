pub struct Window {
    base: usize,
    height: usize,
}

impl Window {
    pub fn new(base: usize, height: usize) -> Window {
        Window { base, height }
    }

    pub fn inc(&mut self) {
        if self.base > 0 {
            self.base = self.base.saturating_sub(1);
        } else {
            self.height = self.height.saturating_sub(1);
        }
    }

    pub fn fused(&self) -> bool {
        self.height == 0
    }

    pub fn visible(&self) -> bool {
        self.base == 0
    }
}
