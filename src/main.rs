use std::collections::{HashMap, VecDeque};

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
    let start = (board.clone(), tools.to_vec());
    let mut paths = HashMap::new();
    paths.insert(start.clone(), None);

    let mut frontier = VecDeque::new();
    frontier.push_back(start);
    // let mut i = 0;
    while let Some((board, tools)) = frontier.pop_front() {
        // i += 1;
        // if i % 100 == 0 {
        //     println!("{i}");
        // }
        if &board == goal {
            println!("found it");
            let path = rebuild_path((board, tools), paths);
            println!("{path:?}");
            return;
        }
        // let prev = Some((board.clone(), tools.clone()));
        for (tool, next_tools) in swap_remove_each(&tools) {
            for action in tool.all_transformations() {
                match &action {
                    Tool::Push(fig) => {
                        let next_board = board.apply_push(&fig);
                        let state = (next_board, next_tools.clone());
                        let prev = (board.clone(), tools.clone(), action);
                        paths.insert(state.clone(), Some(prev));
                        frontier.push_back(state);
                    }
                    Tool::Lift(fig) => {
                        let (next_board, piece) = board.apply_lift(&fig);
                        let mut next_tools = next_tools.clone();
                        next_tools.push(Tool::Piece(piece));
                        let state = (next_board, next_tools);
                        let prev = (board.clone(), tools.clone(), action);
                        paths.insert(state.clone(), Some(prev));
                        frontier.push_back(state);
                    }
                    Tool::Piece(fig) => {
                        let next_board = board.apply_piece(&fig);
                        let state = (next_board, next_tools.clone());
                        let prev = (board.clone(), tools.clone(), action);
                        paths.insert(state.clone(), Some(prev));
                        frontier.push_back(state);
                    },
                    Tool::Copy(fig) => {
                        let next_board = board.apply_copy(&fig);
                        let state = (next_board, next_tools.clone());
                        let prev = (board.clone(), tools.clone(), action);
                        paths.insert(state.clone(), Some(prev));
                        frontier.push_back(state);
                    }
                    Tool::Swap(fig) => {
                        let next_board = board.apply_swap(&fig);
                        let state = (next_board, next_tools.clone());
                        let prev = (board.clone(), tools.clone(), action);
                        paths.insert(state.clone(), Some(prev));
                        frontier.push_back(state);
                    }
                }
            }
        }
    }
}
type GameState = (Board, Vec<Tool>);
fn rebuild_path(goal: GameState, mut paths: HashMap<GameState, Option<(Board, Vec<Tool>, Tool)>>) -> Vec<(Board, Option<Tool>)> {
    let mut path = vec![(goal.0.clone(), None)];
    let mut state = goal;
    while let Some((board, tools, tool)) = paths.remove(&state).flatten() {
        path.push((board.clone(), Some(tool)));
        state = (board, tools);
    }
    path.reverse();
    path
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
            Tool::Push(fig) => fig.all_transformations().into_iter().map(Tool::Push).collect(),
            Tool::Lift(fig) => fig.all_transformations().into_iter().map(Tool::Lift).collect(),
            Tool::Piece(fig) => fig.all_transformations().into_iter().map(Tool::Piece).collect(),
            Tool::Copy(fig) => fig.all_transformations().into_iter().map(Tool::Copy).collect(),
            Tool::Swap(fig) => fig.all_transformations().into_iter().map(Tool::Swap).collect(),
        }
    }
}

fn main() -> Result<(), &'static str> {
    // let board = Board::from_str("
    //     [122]
    //     [111]
    //     [112]
    // ")?;
    // let goal = Board::from_str("
    //     [121]
    //     [121]
    //     [121]
    // ")?;
    // let tools = vec![
    //     Tool::Push(Push::from_str("(>)")?), // 36
    //     Tool::Push(Push::from_str("(>)")?), // 36
    //     Tool::Push(Push::from_str("(>>)")?), // 24
    //     Tool::Lift(Shape::from_str("(##)")?), // oof
    // ];
    let board = Board::from_str("
        [131]
        [111]
        [113]
    ")?;
    let goal = Board::from_str("
        [211]
        [121]
        [112]
    ")?;
    let tools = vec![
        Tool::Piece(Piece::from_str("(22)(.2)")?),
        Tool::Swap(Shape::from_str("(#.#)")?),
        Tool::Swap(Shape::from_str("(##)")?),
        Tool::Push(Push::from_str("(v<)")?),
    ];

    println!("{board:?}\n{goal:?}\n{tools:?}");
    solve(&board, &goal, &tools);
    Ok(())
}
