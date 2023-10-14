struct Figure<T> {
    layout: [T; 9],
    rotatable: bool,
}

type Shape = Figure<bool>;
type Piece = Figure<u8>;
type Push = Figure<usize>;

struct Board {
    state: [u8; 9],
}

enum Tool {
    Push(Push),
    Lift(Shape),
    Piece(Piece),
    Copy(Piece),
    Swap(Shape),
}

fn main() {
    let board = Board { state: [0, 1, 1, 0, 0, 0, 0, 0, 1] };
    let goal = Board { state: [0, 1, 0, 0, 1, 0, 0, 1, 0] };
    let tools = vec![
        Tool::Push(Push { layout: [1, 0, 0, 0, 0, 0, 0, 0, 0], rotatable: true }),
        Tool::Push(Push { layout: [1, 0, 0, 0, 0, 0, 0, 0, 0], rotatable: true }),
        Tool::Push(Push { layout: [1, 1, 0, 0, 0, 0, 0, 0, 0], rotatable: true }),
        Tool::Lift(Shape { layout: [true, true, false, false, false, false, false, false, false], rotatable: true }),
    ];
}
