use std::collections::VecDeque;

mod figure;

use crate::figure::*;

fn swap_remove_each<T: Clone>(list: &[T]) -> impl Iterator<Item=(T, Vec<T>)> + '_ {
    (0..list.len()).map(|idx| {
        let mut list = list.to_vec();
        let elem = list.swap_remove(idx);
        (elem, list)
    })
}

fn solve(board: &Board, goal: &Board, tools: &[Tool]) {
    // TODO: `tools` in the `frontier` might make sense as a `Rc<Vec<Tools>>`
    let mut frontier = VecDeque::new();
    frontier.push_back((board.clone(), tools.to_vec()));
    let mut i = 0;
    while let Some((board, tools)) = frontier.pop_front() {
        i += 1;
        if i % 100 == 0 {
            println!("{i}");
        }
        if &board == goal {
            println!("found it");
            return;
        }
        for (tool, next_tools) in swap_remove_each(&tools) {
            for action in tool.all_transformations() {
                match action {
                    Tool::Push(fig) => {
                        let next_board = board.apply_push(&fig);
                        frontier.push_back((next_board, next_tools.clone()));
                    }
                    Tool::Lift(fig) => {
                        let (next_board, piece) = board.apply_lift(&fig);
                        let mut next_tools = next_tools.clone();
                        next_tools.push(Tool::Piece(piece));
                        frontier.push_back((next_board, next_tools));
                    }
                    Tool::Piece(fig) => {
                        let next_board = board.apply_piece(&fig);
                        frontier.push_back((next_board, next_tools.clone()));
                    },
                    Tool::Copy(fig) => todo!(),
                    Tool::Swap(fig) => todo!(),
                }
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Tool {
    Push(Push),
    Lift(Shape),
    Piece(Piece),
    Copy(Piece),
    Swap(Shape),
}

impl Tool {
    fn all_transformations(&self) -> Vec<Tool> {
        match self {
            Tool::Push(fig) => fig.all_transformations().into_iter().map(|f| Tool::Push(f)).collect(),
            Tool::Lift(fig) => fig.all_transformations().into_iter().map(|f| Tool::Lift(f)).collect(),
            Tool::Piece(fig) => fig.all_transformations().into_iter().map(|f| Tool::Piece(f)).collect(),
            Tool::Copy(fig) => fig.all_transformations().into_iter().map(|f| Tool::Copy(f)).collect(),
            Tool::Swap(fig) => fig.all_transformations().into_iter().map(|f| Tool::Swap(f)).collect(),
        }
    }
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
        Tool::Push(Push::from_str("(>)")?), // 36
        Tool::Push(Push::from_str("(>)")?), // 36
        Tool::Push(Push::from_str("(>>)")?), // 24
        Tool::Lift(Shape::from_str("(##)")?), // oof
    ];

    println!("{board:?}\n{goal:?}\n{tools:?}");
    solve(&board, &goal, &tools);
    Ok(())
}
