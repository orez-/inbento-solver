use std::fmt;
use crate::try_into_array;
use super::{Shape, ParserError};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Swap {
    shape: Shape,
}

impl fmt::Debug for Swap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.shape)
    }
}

impl Swap {
    #[allow(dead_code)]
    pub fn from_str(string: &str) -> Result<Self, ParserError> {
        let shape = Shape::from_str(string)?;
        let swapped_idxs = shape.layout.iter().enumerate()
            .filter(|(_, cell)| cell.is_some())
            .map(|(idx, _)| idx);
        let _cells: [_; 2] = try_into_array(swapped_idxs)
            .map_err(|_| "expected exactly two swap cells")?;
        Ok(Swap { shape })
    }
}

impl Swap {
    pub fn all_transformations(&self) -> Vec<Self> {
        self.shape.all_transformations().into_iter()
            .map(|shape| Swap { shape })
            .collect()
    }

    pub(super) fn swap_idxs(&self) -> [usize; 2] {
        let swapped_idxs = self.shape.layout.iter().enumerate()
            .filter(|(_, cell)| cell.is_some())
            .map(|(idx, _)| idx);
        try_into_array(swapped_idxs).unwrap()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_swap_validation() {
        let err = Swap::from_str("(.#)");
        assert!(err.is_err(), "{err:?}");
    }

    #[test]
    fn test_multi_swap_validation() {
        let err = Swap::from_str("(###)");
        assert!(err.is_err(), "{err:?}");
    }
}
