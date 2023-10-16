mod tools;

use std::collections::{HashMap, VecDeque};
use std::iter::zip;
use crate::tools::*;

pub(crate) fn try_into_array<I: Iterator, const N: usize>(mut it: I) -> Result<[I::Item; N], ()> {
    // it'd be cool if we could skip allocating the vec here,
    // but it's fine.
    let vec: Vec<_> = it.by_ref().take(N).collect();
    if it.next().is_some() {
        return Err(());
    }
    vec.try_into().map_err(|_| ())
}

fn swap_remove_each<T: Clone>(list: &[T]) -> impl Iterator<Item=(T, Vec<T>)> + '_ {
    (0..list.len()).map(|idx| {
        let mut list = list.to_vec();
        let elem = list.swap_remove(idx);
        (elem, list)
    })
}

fn solve(board: &Board, goal: &Board, tools: &[Tool]) -> Result<Vec<(Board, Tool)>, ()> {
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
            return Ok(rebuild_path((board, tools), paths));
        }
        for (tool, next_tools) in swap_remove_each(&tools) {
            for action in tool.all_transformations() {
                match &action {
                    Tool::Push(fig) => {
                        let next_board = board.apply_push(fig);
                        let state = (next_board, next_tools.clone());
                        let prev = (board.clone(), tools.clone(), action);
                        paths.insert(state.clone(), Some(prev));
                        frontier.push_back(state);
                    }
                    Tool::Lift(fig) => {
                        let (next_board, piece) = board.apply_lift(fig);
                        let mut next_tools = next_tools.clone();
                        next_tools.push(Tool::Piece(piece));
                        let state = (next_board, next_tools);
                        let prev = (board.clone(), tools.clone(), action);
                        paths.insert(state.clone(), Some(prev));
                        frontier.push_back(state);
                    }
                    Tool::Piece(fig) => {
                        let next_board = board.apply_piece(fig);
                        let state = (next_board, next_tools.clone());
                        let prev = (board.clone(), tools.clone(), action);
                        paths.insert(state.clone(), Some(prev));
                        frontier.push_back(state);
                    },
                    Tool::Copy(fig) => {
                        let next_board = board.apply_copy(fig);
                        let state = (next_board, next_tools.clone());
                        let prev = (board.clone(), tools.clone(), action);
                        paths.insert(state.clone(), Some(prev));
                        frontier.push_back(state);
                    }
                    Tool::Swap(fig) => {
                        let next_board = board.apply_swap(fig);
                        let state = (next_board, next_tools.clone());
                        let prev = (board.clone(), tools.clone(), action);
                        paths.insert(state.clone(), Some(prev));
                        frontier.push_back(state);
                    }
                }
            }
        }
    }
    Err(())
}
type GameState = (Board, Vec<Tool>);
fn rebuild_path(goal: GameState, mut paths: HashMap<GameState, Option<(Board, Vec<Tool>, Tool)>>) -> Vec<(Board, Tool)> {
    let mut path = Vec::new();
    let mut state = goal;
    while let Some((board, tools, tool)) = paths.remove(&state).flatten() {
        path.push((board.clone(), tool));
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
    Copy(CopyPaste),
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

// Assumes s1 is uniform width, because that's our use case.
fn inline_multiline_strs(s1: &str, s2: &str) -> String {
    let mut out = String::new();
    for (a, b) in zip(s1.lines(), s2.lines()) {
        out.push_str(a);
        out.push_str("   ");
        out.push_str(b);
        out.push('\n');
    }
    out
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

    let solution = solve(&board, &goal, &tools).unwrap();
    for (board, tool) in solution {
        let step = inline_multiline_strs(&format!("{board:?}"), &format!("{tool:?}"));
        println!("{step}");
    }
    println!("{goal:?}");
    Ok(())
}
