use std::fmt;
use std::iter::zip;
use itertools::iproduct;

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

fn try_into_array<I: Iterator, const N: usize>(mut it: I) -> Result<[I::Item; N], ()> {
    // it'd be cool if we could skip allocating the vec here,
    // but it's fine.
    let vec: Vec<_> = it.by_ref().take(N).collect();
    if it.next().is_some() {
        return Err(());
    }
    vec.try_into().map_err(|_| ())
}

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

fn min_x<T>(layout: &[Option<T>; AREA]) -> usize {
    layout.iter().enumerate()
        .flat_map(|(idx, elem)| elem.is_some().then_some(idx % 3 + 1))
        .min()
        .unwrap_or(0)
}

fn min_y<T>(layout: &[Option<T>; AREA]) -> usize {
    layout.iter().enumerate()
        .flat_map(|(idx, elem)| elem.is_some().then_some(idx / 3 + 1))
        .min()
        .unwrap_or(0)
}

fn max_x<T>(layout: &[Option<T>; AREA]) -> usize {
    layout.iter().enumerate()
        .flat_map(|(idx, elem)| elem.is_some().then_some(idx % 3 + 1))
        .max()
        .unwrap_or(0)
}

fn max_y<T>(layout: &[Option<T>; AREA]) -> usize {
    layout.iter().enumerate()
        .flat_map(|(idx, elem)| elem.is_some().then_some(idx / 3 + 1))
        .max()
        .unwrap_or(0)
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
        let bounding_width = max_x(&layout);
        let bounding_height = max_y(&layout);
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

    fn shift(&self, Δx: isize, Δy: isize) -> Self {
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

    fn all_translations(&self) -> impl Iterator<Item=Self> + '_ {
        iproduct!(
            (0..=SIZE - self.bounding_height),
            (0..=SIZE - self.bounding_width)
        ).map(|(x, y)| self.shift(x as isize, y as isize))
    }

    // ignores `rotatable`
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

    pub fn all_transformations(&self) -> Vec<Self> {
        if !self.rotatable {
            return self.all_translations().collect();
        }
        let rotations = self.all_rotations();
        rotations.iter().flat_map(|aligned| aligned.all_translations()).collect()
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

// ===
// tool apply fns
// ===
#[allow(dead_code)]
impl Board {
    pub fn apply_push(&self, push: &Push) -> Self {
        let mut out = self.clone();
        for src in 0..AREA {
            let Some(dir) = push.layout[src] else { continue };
            let Some(cell) = self.layout[src].clone() else { continue };
            let dest = src.wrapping_add(dir);
            // XXX: this boundscheck sucks.
            // rethink how we're representing Direction.
            let sx = src % SIZE;
            let sy = src / SIZE;
            let dx = dest % SIZE;
            let dy = dest / SIZE;
            if (sx != dx && sy != dy) || dest >= AREA { continue }
            out.layout[dest] = Some(cell);
        }
        out
    }

    pub fn apply_lift(&self, lift: &Shape) -> (Self, Piece) {
        let mut out = self.clone();
        let mut lifted = Piece {
            layout: Default::default(),
            rotatable: lift.rotatable, // XXX: idk that this is true in-game
            bounding_width: SIZE,
            bounding_height: SIZE,
        };
        for idx in 0..AREA {
            if lift.layout[idx].is_none() { continue }
            lifted.layout[idx] = out.layout[idx].take();
        }
        let min_x = min_x(&lifted.layout) as isize;
        let min_y = min_y(&lifted.layout) as isize;
        let mut lifted = lifted.shift(-min_x, -min_y);
        lifted.bounding_width = max_x(&lifted.layout);
        lifted.bounding_height = max_y(&lifted.layout);
        (out, lifted)
    }

    pub fn apply_piece(&self, piece: &Piece) -> Self {
        let mut out = self.clone();
        for (src, dest) in zip(piece.layout, &mut out.layout) {
            if src.is_some() {
                *dest = src.clone();
            }
        }
        out
    }

    pub fn apply_copy(&self, copy: &Piece) -> Self {
        // TODO: oh this should DEFINITELY be an enum.
        const COPY: u8 = 1;
        const PASTE: u8 = 2;

        let copied_idxs = copy.layout.iter().enumerate()
            .filter(|(_, cell)| matches!(cell, Some(COPY)))
            .map(|(idx, _)| idx);
        let [copied_idx] = try_into_array(copied_idxs).unwrap();
        let copied_cell = self.layout[copied_idx];
        let mut out = self.clone();
        for (src, dest) in zip(copy.layout, &mut out.layout) {
            if matches!(src, Some(PASTE)) {
                *dest = copied_cell;
            }
        }
        out
    }

    pub fn apply_swap(&self, shape: &Shape) -> Self {
        let swapped_idxs = shape.layout.iter().enumerate()
            .filter(|(_, cell)| cell.is_some())
            .map(|(idx, _)| idx);
        let [cell1, cell2] = try_into_array(swapped_idxs).unwrap();
        let mut out = self.clone();
        out.layout[cell1] = self.layout[cell2];
        out.layout[cell2] = self.layout[cell1];
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
            writeln!(f)?;
            for y in 0..self.bounding_height {
                write_row(f, y)?;
                writeln!(f)?;
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

    #[test]
    fn test_rotations() {
        let shape = Push::from_str("(^>)").unwrap();
        // TODO: ideally this assertion would be orderless
        assert_eq!(shape.all_rotations(), vec![
            Push::from_str("(^>)").unwrap(),
            Push::from_str("(<v)").unwrap(),
            Push::from_str("(>)(v)").unwrap(),
            Push::from_str("(^)(<)").unwrap(),
        ]);
    }

    #[test]
    fn test_180_sym_rotations() {
        let shape = Shape::from_str("(##)").unwrap();
        // TODO: ideally this assertion would be orderless
        assert_eq!(shape.all_rotations(), vec![
            Shape::from_str("(##)").unwrap(),
            Shape::from_str("(#)(#)").unwrap(),
        ]);
    }

    #[test]
    fn test_90_sym_rotations() {
        let shape = Shape::from_str("(##)(##)").unwrap();
        assert_eq!(shape.all_rotations(), vec![
            Shape::from_str("(##)(##)").unwrap(),
        ]);
    }
}
