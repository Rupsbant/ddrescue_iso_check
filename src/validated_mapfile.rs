use ddrescue_mapfile::{Address, Block, BlockStatus, MapFile, Size};

pub struct BadSectors {
    non_overlapping_intervals: Vec<(usize, usize)>,
}

impl From<MapFile> for BadSectors {
    fn from(m: MapFile) -> BadSectors {
        From::from(&m)
    }
}
impl<'a> From<&'a MapFile> for BadSectors {
    fn from(m: &MapFile) -> BadSectors {
        let mut end: usize = 0;
        let mut bad_sectors = vec![];
        for &block in &m.blocks {
            let start = block.pos.0 as usize;
            if start < end {
                panic!("Not yet implemented: possibly overlapping intervals");
            }
            end = start + block.size.0 as usize;
            if block.status != BlockStatus::Finished {
                bad_sectors.push((start, end));
            }
        }
        BadSectors::new(bad_sectors)
    }
}

impl BadSectors {
    pub fn new(non_overlapping_intervals: Vec<(usize, usize)>) -> BadSectors {
        BadSectors {
            non_overlapping_intervals,
        }
    }
    pub fn contains_bad_sector(&self, start: usize, end: usize) -> Result<(), (usize, usize)> {
        let location = self
            .non_overlapping_intervals
            .binary_search_by_key(&end, |&(start, end)| start);
        let idx = match location {
            Ok(found) => found,
            Err(found) => found,
        };
        if idx >= 1 {
            if self.non_overlapping_intervals[idx - 1].1 >= start {
                return Err(self.non_overlapping_intervals[idx-1]);
            }
        }
        if idx < self.non_overlapping_intervals.len() {
            if end >= self.non_overlapping_intervals[idx].0 {
                return Err(self.non_overlapping_intervals[idx]);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_contains_bad_sectors() {
        let bad_sectors = BadSectors::new(vec![(1, 5), (8, 10), (15, 19)]);
        let intervals = vec![
            (6, 7, false),
            (11, 14, false),
            (2, 3, true),
            (5, 7, true),
            (6, 8, true),
            (3, 9, true),
            (3, 16, true),
        ];
        for (start, end, expected) in intervals {
            assert_eq!(bad_sectors.contains_bad_sector(start, end).is_err(), expected, "{:?} {:?} {:?}", start, end, expected);
        }
    }
}
