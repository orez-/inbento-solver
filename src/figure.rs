use std::fmt;

const SIZE: usize = 3;
const AREA: usize = SIZE * SIZE;
// XXX: these directions could be an enum, but I think that'd just complicate things
// (not like all the other decisions i've made here 😐)
type Direction = usize;
const UP: Direction = 0_usize.wrapping_sub(SIZE);
const RIGHT: Direction = 1;
const DOWN: Direction = SIZE;
const LEFT: Direction = usize::MAX;

type ParserError = &'static str;

pub type Piece = Figure<u8>;
pub type Board = Figure<u8>; // ehh
pub type Shape = Figure<()>;
pub type Push = Figure<Direction>;

pub trait InbentoCell: Clone {
    fn to_char(&self) -> char;
    fn parse(c: char) -> Result<Self, ()> where Self: Sized;
    fn rotate(&self) -> Self;
}

impl InbentoCell for () {
    fn to_char(&self) -> char { '#' }
    fn parse(_: char) -> Result<Self, ()> { Ok(()) }
    fn rotate(&self) -> Self { () }
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
            UP => '^',
            RIGHT => '>',
            DOWN => 'v',
            LEFT => '<',
            _ => panic!(),
        }
    }

    fn parse(c: char) -> Result<Self, ()> {
        Ok(match c {
            '^' => UP,
            '>' => RIGHT,
            'v' => DOWN,
            '<' => LEFT,
            _ => return Err(()),
        })
    }

    fn rotate(&self) -> Self {
        match *self {
            UP => RIGHT,
            RIGHT => DOWN,
            DOWN => LEFT,
            LEFT => UP,
            _ => panic!(),
        }
    }
}

#[derive(PartialEq, Eq, std::hash::Hash, Clone)]
pub struct Figure<T: InbentoCell> {
    layout: [Option<T>; AREA],
    rotatable: bool,
    bounding_width: usize,
    bounding_height: usize,
}

impl<T: InbentoCell> Figure<T> {
    pub fn new(layout: [Option<T>; AREA], rotatable: bool) -> Self {
        let bounding_width = layout.iter().enumerate()
            .flat_map(|(idx, elem)| elem.is_some().then(|| idx % 3 + 1))
            .max()
            .unwrap_or(0);
        let bounding_height = layout.iter().enumerate()
            .flat_map(|(idx, elem)| elem.is_some().then(|| idx / 3 + 1))
            .max()
            .unwrap_or(0);
        Figure { layout, rotatable, bounding_width, bounding_height }
    }

    pub fn from_str(string: &str) -> Result<Self, ParserError> {
        let string: String = string.split_whitespace().collect();

        // first pass to learn metadata
        let mut metadata_it = string.chars();
        let rotatable = match metadata_it.next().ok_or("figure must not be empty")? {
            '[' => false,
            '(' => true,
            _ => return Err("figure must be wrapped in `[` or `(`")
        };
        let width = metadata_it.position(|c| matches!(c, ']' | ')'))
            .ok_or("figure must have matching closing delimiter (`[ ]`, `( )`)")?;
        if width > SIZE {
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

        let mut is_open = false;
        let mut writer = LayoutWriter {
            layout: Default::default(),
            idx: 0,
            expected_end: width,
            line_end: SIZE,
        };

        for chr in string.chars() {
            match chr {
                '[' => {
                    if writer.idx >= AREA { return Err("too many rows") } // TODO: String
                    if rotatable { return homogeneity_error }
                    if is_open { return Err("unexpected start of row") }
                    is_open = true;
                }
                '(' => {
                    if writer.idx >= AREA { return Err("too many rows") } // TODO: String
                    if !rotatable { return homogeneity_error }
                    if is_open { return Err("unexpected start of row") }
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
        Ok(Figure::new(layout, rotatable))
    }

    fn configurations(&self) -> Vec<Self> {
        let rotations: Vec<Self> = if self.rotatable {
            let turn90 = self.rotate();
            let turn180 = turn90.rotate();
            let turn270 = turn180.rotate();
            // TODO: dedupe
            vec![self.clone(), turn90, turn180, turn270]
        } else { vec![self.clone()] };

        vec![]
        // for offset in
    }

    fn rotate(&self) -> Self {
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

        write!(f, "\"")?;
        if self.bounding_height == 1 {
            write_row(f, 0)?;
        } else {
            write!(f, "\n")?;
            for y in 0..self.bounding_height {
                write_row(f, y)?;
                write!(f, "\n")?;
            }
        }
        write!(f, "\"")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
