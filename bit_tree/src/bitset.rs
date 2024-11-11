#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BitSet<const NU32: usize>([u32; NU32]);

pub(crate) type ByteSet = BitSet<8>;

impl<const NU32: usize> BitSet<NU32> {
    #[inline(always)]
    #[cfg(test)]
    pub(crate) const fn size(&self) -> usize {
        NU32 * 32
    }

    #[inline(always)]
    const fn entry_bit(&self, id: usize) -> (usize, usize) {
        (id / 32, id % 32)
    }

    #[inline(always)]
    pub(crate) const fn includes(&self, id: usize) -> bool {
        let (entry, bit) = self.entry_bit(id);
        let store = self.0[entry];
        (store >> bit) & 1 == 1
    }

    #[inline(always)]
    pub(crate) fn insert(&mut self, id: usize) {
        let (entry, bit) = self.entry_bit(id);
        let bitmask = 1 << bit;
        self.0[entry] |= bitmask;
    }

    #[inline(always)]
    #[cfg(test)]
    pub(crate) fn remove(&mut self, id: usize) {
        assert!(self.includes(id), "Removing non-existing element {id}");
        self.remove_unchecked(id);
    }

    #[inline(always)]
    #[cfg(test)]
    pub(crate) fn remove_unchecked(&mut self, id: usize) {
        let (entry, bit) = self.entry_bit(id);
        let bitmask = 1 << bit;
        self.0[entry] &= !bitmask;
    }

    #[inline(always)]
    pub(crate) fn index(&self, id: usize) -> Option<u32> {
        if !self.includes(id) {
            None
        } else {
            let (entry, bit) = self.entry_bit(id);
            let pre_entry_offset: u32 = self.0[..entry].iter().map(|e| e.count_ones()).sum();
            let entry_bitmask = (0b1 << bit) - 1;
            let entry_index = (self.0[entry] & entry_bitmask).count_ones();

            Some(pre_entry_offset + entry_index)
        }
    }

    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self([0; NU32])
    }

    #[inline(always)]
    #[cfg(test)]
    pub(crate) fn from_entries(entries: impl IntoIterator<Item = usize>) -> Self {
        let mut set = Self::new();
        for entry in entries {
            set.insert(entry);
        }
        set
    }
}

#[test]
fn test_bitset_includes() {
    let set = BitSet([0b11]);
    assert!(set.includes(0));
    assert!(set.includes(1));
    assert!(!set.includes(2));

    let byteset = BitSet([0b011, u32::MAX, 0b101, 0b100]);
    assert!(byteset.includes(0));
    assert!(byteset.includes(1));

    for i in 32..64 {
        assert!(byteset.includes(i));
    }

    assert!(byteset.includes(64));
    assert!(!byteset.includes(65));
    assert!(byteset.includes(66));

    assert!(!byteset.includes(32 * 3 + 0));
    assert!(!byteset.includes(32 * 3 + 1));
    assert!(byteset.includes(32 * 3 + 2));
}

#[test]
fn test_remove() {
    let mut set = BitSet([0, 0]);
    set.insert(10);
    for i in 0..set.size() {
        if i == 10 {
            assert!(set.includes(i));
        } else {
            assert!(!set.includes(i));
        }
    }
    set.remove(10);
    for i in 0..set.size() {
        assert!(!set.includes(i));
    }
}

#[test]
#[should_panic]
fn test_remove_nonexistant() {
    let mut set = BitSet([0b11]);
    set.remove(3);
}

#[test]
fn test_index() {
    let mut set = ByteSet::from_entries([1, 2, 128]);
    assert_eq!(set.index(0), None);
    assert_eq!(set.index(1), Some(0));
    assert_eq!(set.index(2), Some(1));
    assert_eq!(set.index(128), Some(2));

    set.insert(5);
    assert_eq!(set.index(1), Some(0));
    assert_eq!(set.index(2), Some(1));
    assert_eq!(set.index(5), Some(2));
    assert_eq!(set.index(128), Some(3));
}
