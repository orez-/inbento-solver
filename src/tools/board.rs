use std::iter::zip;
use super::{Figure, Push, Piece, Shape, CopyPaste, Swap, AREA, SIZE};

pub type Board = Figure<u8>; // ehh

// ===
// tool apply fns
// ===
#[allow(dead_code)]
impl Board {
    pub fn apply_push(&self, push: &Push) -> Self {
        // to ensure we're moving all the cells simultaneously, we lift
        // each cell with a valid destination from `out` and place it at
        // its destination in `lifted`, then apply `lifted` to `out`.
        let mut out = self.clone();
        let mut lifted = Piece {
            layout: Default::default(),
            rotatable: false,
            bounding_width: SIZE,
            bounding_height: SIZE,
        };
        for src in 0..AREA {
            let Some(dir) = push.layout[src] else { continue };
            if out.layout[src].is_none() { continue };
            let Some(dest) = src.checked_add_signed(dir as isize) else { continue };
            // edge check
            let sx = src % SIZE;
            let sy = src / SIZE;
            let dx = dest % SIZE;
            let dy = dest / SIZE;
            if (sx != dx && sy != dy) || dest >= AREA { continue }
            lifted.layout[dest] = out.layout[src].take();
        }
        out.apply_piece_mut(&lifted);
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
        out.apply_piece_mut(piece);
        out
    }

    fn apply_piece_mut(&mut self, piece: &Piece) {
        for (src, dest) in zip(piece.layout, &mut self.layout) {
            if src.is_some() {
                *dest = src;
            }
        }
    }

    pub fn apply_copy(&self, copy: &CopyPaste) -> Self {
        let copied_idx = copy.copy_idx();
        let copied_cell = self.layout[copied_idx];
        let mut out = self.clone();
        for (src, dest) in zip(copy.shape.layout, &mut out.layout) {
            if src.is_some() {
                *dest = copied_cell;
            }
        }
        out
    }

    pub fn apply_swap(&self, swap: &Swap) -> Self {
        let mut out = self.clone();
        let [cell1, cell2] = swap.swap_idxs();
        out.layout[cell1] = self.layout[cell2];
        out.layout[cell2] = self.layout[cell1];
        out
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push() {
        let board = Board::from_str("[123][456][789]").unwrap();
        let push = Push::from_str("(...)(..v)(..>)").unwrap();
        let expected = Board::from_str("[123][45.][786]").unwrap();
        let actual = board.apply_push(&push);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_push_no_wrap_right() {
        let board = Board::from_str("[123][456][789]").unwrap();
        let push = Push::from_str("(...)(..>)(...)").unwrap();
        let actual = board.apply_push(&push);
        assert_eq!(actual, board);
    }

    #[test]
    fn test_push_no_wrap_down() {
        let board = Board::from_str("[123][456][789]").unwrap();
        let push = Push::from_str("(...)(...)(.v.)").unwrap();
        let actual = board.apply_push(&push);
        assert_eq!(actual, board);
    }
}
