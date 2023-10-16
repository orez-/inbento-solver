use std::fmt;
use itertools::iproduct;

pub(super) const SIZE: usize = 3;
pub(super) const AREA: usize = SIZE * SIZE;

#[repr(i8)]
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Direction {
    Down = SIZE as i8,
    Left = -1,
    Right = 1,
    Up = -(SIZE as i8),
}

pub(super) type ParserError = &'static str;

pub type Piece = Figure<u8>;
pub type Shape = Figure<()>;
pub type Push = Figure<Direction>;

pub trait InbentoCell: Clone + PartialEq {
    fn to_char(&self) -> char;
    fn parse(c: char) -> Result<Self, ()> where Self: Sized;
    fn rotate(&self) -> Self;
}

impl InbentoCell for () {
    fn to_char(&self) -> char { '#' }
    fn parse(_: char) -> Result<Self, ()> { Ok(()) }
    fn rotate(&self) -> Self {}
}

impl InbentoCell for u8 {
    fn to_char(&self) -> char {
        if *self >= 10 { panic!() }
        (self + b'0') as char
    }

    fn parse(c: char) -> Result<Self, ()> {
        c.to_digit(10).ok_or(()).map(|d| d as u8)
    }

    fn rotate(&self) -> Self { *self }
}

impl InbentoCell for Direction {
    fn to_char(&self) -> char {
        match *self {
            Self::Up => '^',
            Self::Right => '>',
            Self::Down => 'v',
            Self::Left => '<',
        }
    }

    fn parse(c: char) -> Result<Self, ()> {
        Ok(match c {
            '^' => Self::Up,
            '>' => Self::Right,
            'v' => Self::Down,
            '<' => Self::Left,
            _ => return Err(()),
        })
    }

    fn rotate(&self) -> Self {
        match *self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Figure<T: InbentoCell> {
    pub(super) layout: [Option<T>; AREA],
    pub(super) rotatable: bool,
    pub(super) bounding_width: usize,
    pub(super) bounding_height: usize,
}

impl<T: InbentoCell> Figure<T> {
    pub fn from_str(string: &str) -> Result<Self, ParserError> {
        let string: String = string.split_whitespace().collect();

        // first pass to learn metadata
        let mut metadata_it = string.chars();
        let rotatable = match metadata_it.next().ok_or("figure must not be empty")? {
            '[' => false,
            '(' => true,
            _ => return Err("figure must be wrapped in `[` or `(`")
        };
        let bounding_width = metadata_it.position(|c| matches!(c, ']' | ')'))
            .ok_or("figure must have matching closing delimiter (`[ ]`, `( )`)")?;
        if bounding_width > SIZE {
            return Err("figure is too wide"); // TODO: String (or const interpolation would be nice)
        }

        // second pass to parse
        let homogeneity_error = Err("figure must be wrapped entirely in `[ ]` or `( )`");

        struct LayoutWriter<T> {
            layout: [Option<T>; AREA],
            idx: usize,
            line_end: usize,
            expected_end: usize,
        }

        impl<T> LayoutWriter<T> {
            fn write(&mut self, val: Option<T>) -> Result<(), ParserError> {
                if self.idx >= self.expected_end {
                    return Err("too many elements in row"); // TODO: String
                }
                self.layout[self.idx] = val;
                self.idx += 1;
                Ok(())
            }

            fn new_line(&mut self) -> Result<(), ParserError> {
                if self.idx < self.expected_end {
                    return Err("not enough elements in row"); // TODO: String
                }
                self.idx = self.line_end;
                self.expected_end += SIZE;
                self.line_end += SIZE;
                Ok(())
            }
        }

        let mut bounding_height = 0;
        let mut is_open = false;
        let mut writer = LayoutWriter {
            layout: Default::default(),
            idx: 0,
            expected_end: bounding_width,
            line_end: SIZE,
        };

        for chr in string.chars() {
            match chr {
                '[' => {
                    if writer.idx >= AREA { return Err("too many rows") } // TODO: String
                    if rotatable { return homogeneity_error }
                    if is_open { return Err("unexpected start of row") }
                    bounding_height += 1;
                    is_open = true;
                }
                '(' => {
                    if writer.idx >= AREA { return Err("too many rows") } // TODO: String
                    if !rotatable { return homogeneity_error }
                    if is_open { return Err("unexpected start of row") }
                    bounding_height += 1;
                    is_open = true;
                }
                '.' => writer.write(None)?,
                ']' => {
                    if rotatable { return homogeneity_error }
                    if !is_open { return Err("unexpected end of row") }
                    writer.new_line()?;
                    is_open = false;
                }
                ')' => {
                    if !rotatable { return homogeneity_error }
                    if !is_open { return Err("unexpected end of row") }
                    writer.new_line()?;
                    is_open = false;
                }
                chr => {
                    if !is_open { return Err("unexpected character outside of row") }
                    writer.write(Some(
                        InbentoCell::parse(chr)
                            .map_err(|_| "could not parse character")? // TODO: String
                    ))?;
                }
            }
        }
        if is_open { return Err("unterminated row") }
        let LayoutWriter { layout, .. } = writer;
        Ok(Figure { layout, rotatable, bounding_width, bounding_height })
    }

    /// Return a clone of this Figure rotated 90˚ clockwise. The Figure's
    /// bounding width and height will be rotated as well. If the Figure's
    /// elements have directionality, they will also be rotated appropriately.
    ///
    /// eg:
    ///  (123)    (41)
    ///  (4..) => (.2)
    ///           (.3)
    pub(super) fn rotate(&self) -> Self {
        let mut out = Self {
            layout: Default::default(),
            rotatable: self.rotatable,
            bounding_width: self.bounding_height, // nb: swapped
            bounding_height: self.bounding_width, // nb: swapped
        };
        for sy in 0..self.bounding_height {
            for sx in 0..self.bounding_width {
                let dx = self.bounding_height - sy - 1;
                let dy = sx;
                let sidx = sy * SIZE + sx;
                let didx = dy * SIZE + dx;
                // rotate the individual cell as well, if it needs it.
                out.layout[didx] = self.layout[sidx].as_ref().map(|ic| ic.rotate());
            }
        }
        out
    }

    /// Return a clone of this Figure translated by some Δx and Δy.
    /// Expands the bounding width and height to the full 3×3 area,
    /// representing the Figure's absolute position within the space.
    ///
    /// Any cells which are translated past the 3×3 area are discarded.
    ///
    /// eg, shifting the following figure by 1, 2:
    ///   (56)    (...)
    ///        => (...)
    ///           (.56)
    #[allow(non_snake_case)]
    pub(super) fn shift(&self, Δx: isize, Δy: isize) -> Self {
        // assert!(self.bounding_width + Δx < SIZE);
        // assert!(self.bounding_height + Δy < SIZE);
        let mut out = Self {
            layout: Default::default(),
            rotatable: self.rotatable,
            bounding_width: SIZE,
            bounding_height: SIZE,
        };
        for sy in 0..self.bounding_height {
            let Some(dy) = sy.checked_add_signed(Δy) else { continue };
            if dy >= SIZE { continue }
            for sx in 0..self.bounding_width {
                let Some(dx) = sx.checked_add_signed(Δx) else { continue };
                if dx >= SIZE { continue }
                let sidx = sy * SIZE + sx;
                let didx = dy * SIZE + dx;
                out.layout[didx] = self.layout[sidx].clone();
            }
        }
        out
    }

    /// Returns an iterator of the different translations of this Figure
    /// inscribed within the 3×3 area. Expands the bounding width and height
    /// to the full area, representing the Figure's absolute position within
    /// the space.
    ///
    /// eg, for the Piece:
    ///   (12)
    ///   (34)
    ///
    /// its translations are:
    ///   (12.)  (.12)  (...)  (...)
    ///   (34.)  (.34)  (12.)  (.12)
    ///   (...)  (...)  (34.)  (.34)
    ///
    /// Note that the exact order of the translations should not be relied on.
    fn all_translations(&self) -> impl Iterator<Item=Self> + '_ {
        iproduct!(
            (0..=SIZE - self.bounding_width),
            (0..=SIZE - self.bounding_height)
        ).map(|(x, y)| self.shift(x as isize, y as isize))
    }

    /// Returns a Vec of the *unique* rotations of this Figure. The Figure's
    /// bounding width and height will be rotated as well. If the Figure's
    /// elements have directionality, they will also be rotated appropriately.
    ///
    /// eg, for the Push:
    ///   (>v)
    ///   (<.)
    ///   (>.)
    ///
    /// its rotations are:
    ///   (>v)  (v^v)  (.<)  (>..)
    ///   (<.)  (..<)  (.>)  (^v^)
    ///   (>.)         (^<)
    ///
    /// Note that the exact order of the rotations should not be relied on.
    /// Note also that this function will provide rotations regardless of
    /// the `rotatable` flag.
    fn all_rotations(&self) -> Vec<Self> {
        let turn90 = self.rotate();
        let turn180 = turn90.rotate();
        let turn270 = turn180.rotate();
        // XXX: is it true to say that 0˚=90˚ -> 0˚=180˚ ?
        // like, either
        // - all rotations are unique (len = 4)
        // - 180˚ symmetry (len = 2)
        // - 90˚ symmetry (len = 1)
        // I hope so, because otherwise we could have duplicate rotations here.
        let mut rotations = vec![self.clone(), turn180, turn90, turn270];
        rotations.dedup();
        rotations
    }

    /// Returns a Vec of the *unique* transformations that may be applied
    /// to this Figure. That is, all the translations of all the unique
    /// rotations of the Figure.
    ///
    /// If the Figure is not `rotatable`, returns only the translations.
    ///
    /// eg, for the Piece:
    ///   (123)
    ///   (4.6)
    ///
    /// its transformations are:
    ///   (123)  (...)  (41.)  (.41)  (6.4)  (...)  (36.)  (.36)
    ///   (4.6)  (123)  (.2.)  (..2)  (321)  (6.4)  (2..)  (.2.)
    ///   (...)  (4.6)  (63.)  (.63)  (...)  (321)  (14.)  (.14)
    ///
    /// Note that the exact order of the transformations should not be relied on.
    pub fn all_transformations(&self) -> Vec<Self> {
        if !self.rotatable {
            return self.all_translations().collect();
        }
        let rotations = self.all_rotations();
        rotations.iter().flat_map(|aligned| aligned.all_translations()).collect()
    }
}

impl<T: InbentoCell> fmt::Debug for Figure<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let write_row = |f: &mut fmt::Formatter, y: usize| {
            write!(f, "{}", if self.rotatable { '(' } else { '[' })?;
            for x in 0..self.bounding_width {
                let idx = y * SIZE + x;
                write!(f, "{}", match &self.layout[idx] {
                    None => '.',
                    Some(c) => InbentoCell::to_char(c),
                })?;
            }
            write!(f, "{}", if self.rotatable { ')' } else { ']' })
        };

        if self.bounding_height == 1 {
            write_row(f, 0)?;
        } else {
            writeln!(f)?;
            for y in 0..self.bounding_height {
                write_row(f, y)?;
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: currently uhhh not orderless.
    // until we implement this, this is more a way to document
    // which eqs should be orderless.
    macro_rules! assert_eq_orderless {
        ($left:expr, $right:expr $(,)?) => { assert_eq!($left, $right) };
        ($left:expr, $right:expr, $($arg:tt)+) => { assert_eq!($left, $right, $($arg)+) };
    }

    #[test]
    fn test_unterminated_row() {
        let err = Shape::from_str("(#.#)(.#");
        assert!(err.is_err(), "{err:?}");
    }

    #[test]
    fn test_extra_opener() {
        let err = Shape::from_str("(#.#)(.#(.)");
        assert!(err.is_err(), "{err:?}");
    }

    #[test]
    fn test_too_tall() {
        assert_eq!(SIZE, 3); // test only works if we know what too-tall is
        let err = Shape::from_str("(#)(.)(#)(#)");
        assert!(err.is_err(), "{err:?}");
    }

    #[test]
    fn test_multirow() {
        let shape = Shape::from_str("(#)(.)(#)");
        assert!(shape.is_ok(), "{shape:?}");
    }

    #[test]
    fn test_rotate() {
        let shape = Piece::from_str("(12)(34)(56)").unwrap();
        assert_eq!(shape.rotate(), Piece::from_str("(531)(642)").unwrap());
    }

    #[test]
    fn test_rotate_directions() {
        let shape = Push::from_str("(^>)").unwrap();
        assert_eq!(shape.rotate(), Push::from_str("(>)(v)").unwrap());
    }

    #[test]
    fn test_rotations() {
        let shape = Push::from_str("(^>)").unwrap();
        assert_eq_orderless!(shape.all_rotations(), vec![
            Push::from_str("(^>)").unwrap(),
            Push::from_str("(<v)").unwrap(),
            Push::from_str("(>)(v)").unwrap(),
            Push::from_str("(^)(<)").unwrap(),
        ]);
    }

    #[test]
    fn test_180_sym_rotations() {
        let shape = Shape::from_str("(##)").unwrap();
        assert_eq_orderless!(shape.all_rotations(), vec![
            Shape::from_str("(##)").unwrap(),
            Shape::from_str("(#)(#)").unwrap(),
        ]);
    }

    #[test]
    fn test_90_sym_rotations() {
        let shape = Shape::from_str("(##)(##)").unwrap();
        assert_eq_orderless!(shape.all_rotations(), vec![
            Shape::from_str("(##)(##)").unwrap(),
        ]);
    }

    #[test]
    fn test_transformations() {
        let shape = Shape::from_str("(#.#)").unwrap();
        assert_eq_orderless!(shape.all_transformations(), vec![
            Shape::from_str("(#.#)(...)(...)").unwrap(),
            Shape::from_str("(...)(#.#)(...)").unwrap(),
            Shape::from_str("(...)(...)(#.#)").unwrap(),
            Shape::from_str("(#..)(...)(#..)").unwrap(),
            Shape::from_str("(.#.)(...)(.#.)").unwrap(),
            Shape::from_str("(..#)(...)(..#)").unwrap(),
        ]);
    }
}
