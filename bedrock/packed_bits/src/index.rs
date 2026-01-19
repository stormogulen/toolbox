#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ElemIndex(pub usize);

impl<const N: usize> From<ElemIndex> for BitIndex {
    #[inline]
    fn from(e: ElemIndex) -> Self {
        BitIndex(HEADER_SIZE * 8 + e.0 * N)
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BitIndex(pub usize);

impl BitIndex {
    #[inline]
    pub fn get(self) -> usize {
        self.0
    }
}

