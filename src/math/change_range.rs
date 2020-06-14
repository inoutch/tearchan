use std::ops::Range;

#[derive(Debug, PartialEq)]
pub struct ChangeRange {
    pub size: usize,
    range_start: usize,
    range_end: usize,
}

impl ChangeRange {
    pub fn new(size: usize) -> ChangeRange {
        ChangeRange {
            size,
            range_start: 0,
            range_end: size,
        }
    }

    pub fn resize(&mut self, size: usize) {
        if self.range_start == std::usize::MAX && self.size == 0 {
            self.range_start = 0;
        }

        self.size = size;
        if self.range_start >= size {
            self.reset();
            return;
        }
        self.range_end = size;
    }

    pub fn resize_and_update(&mut self, start: usize, size: usize) {
        self.size = size;
        self.range_end = size;
        self.update(start, size);
    }

    pub fn update(&mut self, start: usize, end: usize) {
        debug_assert!(
            start <= end,
            format!("start wasn't less than end [s={}, e={}]", start, end)
        );

        if self.range_start == std::usize::MAX {
            self.range_start = start;
            self.range_end = end;
            return;
        }

        if start < self.range_start {
            self.range_start = start;
        }
        if end > self.range_end && end <= self.size {
            self.range_end = end
        }
    }

    pub fn update_all(&mut self) {
        self.update(0, self.size);
    }

    pub fn reset(&mut self) {
        self.range_start = std::usize::MAX;
        self.range_end = std::usize::MAX;
    }

    pub fn get_range(&self) -> Option<Range<usize>> {
        if self.range_start == std::usize::MAX {
            return None;
        }
        Some(Range {
            start: self.range_start,
            end: self.range_end,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::math::change_range::ChangeRange;
    use std::ops::Range;

    #[test]
    fn test_standard() {
        let mut change_range = ChangeRange::new(32);

        assert_eq!(change_range.size, 32);

        assert_eq!(change_range.range_start, 0);
        assert_eq!(change_range.range_end, 32);

        assert_eq!(change_range.get_range(), Some(Range { start: 0, end: 32 }));

        change_range.reset();
        assert_eq!(change_range.get_range(), None);

        change_range.update(10, 25);
        assert_eq!(change_range.get_range(), Some(Range { start: 10, end: 25 }));

        change_range.update(5, 15);
        assert_eq!(change_range.get_range(), Some(Range { start: 5, end: 25 }));

        change_range.update_all();
        assert_eq!(change_range.get_range(), Some(Range { start: 0, end: 32 }));

        change_range.resize(0);
        assert_eq!(change_range.get_range(), None);

        change_range.resize(10);
        assert_eq!(change_range.get_range(), Some(Range { start: 0, end: 10 }));

        change_range.reset();
        change_range.resize_and_update(10, 35);
        assert_eq!(change_range.get_range(), Some(Range { start: 10, end: 35 }));

        change_range.update(20, 25);
        change_range.resize(26);
        assert_eq!(change_range.get_range(), Some(Range { start: 10, end: 26 }));
    }

    #[test]
    fn test_resize_to_none() {
        let mut change_range = ChangeRange::new(32);
        change_range.reset();
        change_range.update(20, 32);
        change_range.resize(20);

        assert_eq!(change_range.get_range(), None);

        change_range.update(10, 15);
        change_range.resize(12);

        assert_eq!(change_range.get_range(), Some(Range { start: 10, end: 12 }));
    }

    #[test]
    fn test_resize_and_update() {
        let mut change_range = ChangeRange::new(32);
        change_range.reset();
        change_range.update(20, 32);
        change_range.resize_and_update(10, 25);

        assert_eq!(change_range.get_range(), Some(Range { start: 10, end: 25 }));
    }
}
