// use std::collections::VecDeque;

mod figure;

use crate::figure::*;

fn solve(board: &Board, goal: &Board, tools: &[Tool]) {

}

#[allow(dead_code)]
#[derive(Debug)]
enum Tool {
    Push(Push),
    Lift(Shape),
    Piece(Piece),
    Copy(Piece),
    Swap(Shape),
}

fn main() -> Result<(), &'static str> {
    let board = Board::from_str("
        [122]
        [111]
        [112]
    ")?;
    let goal = Board::from_str("
        [121]
        [121]
        [121]
    ")?;
    let tools = vec![
        Tool::Push(Push::from_str("(>)")?),
        Tool::Push(Push::from_str("(>)")?),
        Tool::Push(Push::from_str("(>>)")?),
        Tool::Lift(Shape::from_str("(##)")?),
    ];

    println!("{board:?}\n{goal:?}\n{tools:?}");
    solve(&board, &goal, &tools);
    Ok(())
}
