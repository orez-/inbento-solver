use std::fmt;

const SIZE: usize = 3;
// XXX: these directions could be an enum, but I think that'd just complicate things
// (not like all the other decisions i've made here 😐)
const UP: usize = 0_usize.wrapping_sub(SIZE);
const RIGHT: usize = 1;
const DOWN: usize = SIZE;
const LEFT: usize = usize::MAX;

type ParserError = &'static str;

pub type Piece = Figure<u8>;
pub type Board = Figure<u8>; // ehh
pub type Shape = Figure<()>;
pub type Push = Figure<usize>;

pub trait InbentoParsable {
    fn to_char(&self) -> char;
    fn parse(c: char) -> Result<Self, ()> where Self: Sized;
}

impl InbentoParsable for () {
    fn to_char(&self) -> char { '#' }
    fn parse(_: char) -> Result<Self, ()> { Ok(()) }
}

impl InbentoParsable for u8 {
    fn to_char(&self) -> char {
        if *self >= 10 { panic!() }
        (self + b'0') as char
    }

    fn parse(c: char) -> Result<Self, ()> {
        c.to_digit(10).ok_or(()).map(|d| d as u8)
    }
}

impl InbentoParsable for usize {
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
}

#[derive(PartialEq, Eq, std::hash::Hash)]
pub struct Figure<T: InbentoParsable> {
    layout: [Option<T>; SIZE * SIZE],
    rotatable: bool,
}

impl<T: InbentoParsable> Figure<T> {
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
            layout: [Option<T>; SIZE * SIZE],
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
                self.idx = self.expected_end;
                self.expected_end += SIZE;
                self.line_end += SIZE;
                Ok(())
            }
        }

        let mut writer = LayoutWriter {
            layout: Default::default(),
            idx: 0,
            expected_end: width,
            line_end: SIZE,
        };

        for chr in string.chars() {
            match chr {
                '[' => if rotatable { return homogeneity_error }
                '(' => if !rotatable { return homogeneity_error }
                '.' => writer.write(None)?,
                ']' => {
                    if rotatable { return homogeneity_error }
                    writer.new_line()?;
                }
                ')' => {
                    if !rotatable { return homogeneity_error }
                    writer.new_line()?;
                }
                chr => writer.write(Some(
                    InbentoParsable::parse(chr)
                        .map_err(|_| "could not parse character")? // TODO: String
                ))?,
            }
        }
        let LayoutWriter { layout, .. } = writer;
        Ok(Figure { layout, rotatable })
    }
    // fn configurations(&self) -> Vec<Self> {
    //     // TODO: Push rotates different

    // }
}

impl<T: fmt::Debug + InbentoParsable> fmt::Debug for Figure<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..SIZE {
            write!(f, "{}", if self.rotatable { '(' } else { '[' })?;
            for x in 0..SIZE {
                let idx = y * SIZE + x;
                write!(f, "{}", match &self.layout[idx] {
                    None => '.',
                    Some(c) => InbentoParsable::to_char(c),
                })?;
            }
            write!(f, "{}\n", if self.rotatable { ')' } else { ']' })?;
        }
        Ok(())
    }
}
