use std::ops::Range;

#[derive(Debug)]
pub struct ChangeRange {
    pub size: u32,
    range_start: u32,
    range_end: u32,
}

impl ChangeRange {
    pub fn new(size: u32) -> ChangeRange {
        ChangeRange {
            size,
            range_start: 0,
            range_end: size,
        }
    }

    pub fn resize(&mut self, size: u32) {
        self.size = size;
        self.range_end = size;
        if self.range_start > size {
            self.range_start = size;
        }
    }

    pub fn resize_and_update(&mut self, start: u32, size: u32) {
        self.size = size;
        self.update(start, size);
    }

    pub fn update(&mut self, start: u32, end: u32) {
        debug_assert!(start < end, "start wasn't less than end");

        if self.range_start == std::u32::MAX {
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
        self.range_start = std::u32::MAX;
        self.range_end = std::u32::MAX;
    }

    pub fn get_range(&self) -> Option<Range<u32>> {
        if self.range_start == std::u32::MAX {
            return None;
        }
        Some(Range {
            start: self.range_start,
            end: self.range_end,
        })
    }
}

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
    assert_eq!(change_range.get_range(), Some(Range { start: 0, end: 0 }));

    change_range.resize(10);
    assert_eq!(change_range.get_range(), Some(Range { start: 0, end: 10 }));

    change_range.reset();
    change_range.resize_and_update(10, 35);
    assert_eq!(change_range.get_range(), Some(Range { start: 10, end: 35 }));

    change_range.update(20, 25);
    change_range.resize(26);
    assert_eq!(change_range.get_range(), Some(Range { start: 10, end: 26 }));
}
