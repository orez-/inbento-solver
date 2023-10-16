use std::fmt;
use crate::try_into_array;
use super::{Figure, ParserError, InbentoCell};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum CopyPasteCell {
    Copy,
    Paste,
}

impl InbentoCell for CopyPasteCell {
    fn to_char(&self) -> char {
        match self {
            CopyPasteCell::Copy => 'C',
            CopyPasteCell::Paste => 'V',
        }
    }

    fn parse(c: char) -> Result<Self, ()> {
        match c {
            'C' => Ok(CopyPasteCell::Copy),
            'V' => Ok(CopyPasteCell::Paste),
            _ => Err(()),
        }
    }

    fn rotate(&self) -> Self { *self }
}

// we could keep a `Shape` instead, but we'd probably want to convert
// back for the Debug anyway.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CopyPaste {
    pub(super) shape: Figure<CopyPasteCell>,
}

impl fmt::Debug for CopyPaste {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.shape)
    }
}

impl CopyPaste {
    #[allow(dead_code)]
    pub fn from_str(string: &str) -> Result<Self, ParserError> {
        let shape = Figure::from_str(string)?;
        let copy_idxs = shape.layout.iter().enumerate()
            .filter(|(_, cell)| matches!(cell, Some(CopyPasteCell::Copy)))
            .map(|(idx, _)| idx);
        let [_copy_idx] = try_into_array(copy_idxs)
            .map_err(|_| "expected exactly one copy cell")?;
        Ok(CopyPaste { shape })
    }
}

impl CopyPaste {
    pub fn all_transformations(&self) -> Vec<Self> {
        self.shape.all_transformations().into_iter()
            .map(|shape| CopyPaste { shape })
            .collect()
    }

    pub(super) fn copy_idx(&self) -> usize {
        let copy_idxs = self.shape.layout.iter().enumerate()
            .filter(|(_, cell)| matches!(cell, Some(CopyPasteCell::Copy)))
            .map(|(idx, _)| idx);
        let [copy_idx] = try_into_array(copy_idxs).unwrap();
        copy_idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_copy_validation() {
        let err = CopyPaste::from_str("(VVV)");
        assert!(err.is_err(), "{err:?}");
    }

    #[test]
    fn test_multi_copy_validation() {
        let err = CopyPaste::from_str("(CVC)");
        assert!(err.is_err(), "{err:?}");
    }
}
