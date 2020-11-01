use super::board::{Board, Color, Piece, PieceKind, Position};
use std::collections::HashMap;

struct MovePattern {
    is_repeatable: bool,
    move_enumerations: Vec<Position>,
}

impl MovePattern {
    fn new(is_repeatable: bool, move_enumerations: Vec<Position>) -> MovePattern {
        MovePattern {
            is_repeatable,
            move_enumerations,
        }
    }
}

pub enum BoardSetup {
    Basic,
}

pub struct ChessEngine {
    pub board: Board,
}

impl ChessEngine {

    pub fn create_board(setup: BoardSetup) -> Result<Board, String> {
        match setup {
            BoardSetup::Basic => ChessEngine::setup_basic_board()
        }
    }

    pub fn possible_moves(board: &Board, p: &Position) -> Result<Vec<Position>, String> {
        match board.get_space(p)? {
            Option::None => Ok(vec![]),
            Option::Some(Piece { kind: PieceKind::Pawn, color: c, has_moved: hm }) => {
                Ok(ChessEngine::generate_pawn_moves(board, p, *c, *hm))
            },
            Option::Some(Piece { kind: PieceKind::King, color: c, has_moved: hm }) => {
                Ok(ChessEngine::generate_king_moves(board, p, *c, *hm))
            },
            Option::Some(piece) => {
                let mut solutions = vec![];
                let pattern = ChessEngine::get_move_pattern(piece.kind)?;
                for diff in pattern.move_enumerations.iter() {
                    ChessEngine::apply_move(board, &mut solutions, p, diff, &piece.color,
                                    pattern.is_repeatable)
                }
                Ok(solutions)
            }
        }
    }

    fn generate_king_moves(board: &Board, p: &Position, color: Color, has_moved: bool)
        -> Vec<Position> {
        let mut solutions = vec![];
        if !has_moved {
            if ChessEngine::can_side_castle(board, p, true) {
                solutions.push(Position::new(p.row, p.col - 3));
            }
            if ChessEngine::can_side_castle(board, p, false) {
                solutions.push(Position::new(p.row, p.col + 2));
            }
        }
        // do the basic case
        let surroundings: Vec<Position> = ChessEngine::expand_with_inverses(vec![
            Position::new(0, 1),
            Position::new(1, 0),
            Position::new(1, 1),
        ])
            .iter()
            .map(|pos| Position::add(p, pos))
            .collect();

        for pos in surroundings {
            if let Ok(_) = board.get_space(&pos) {
                if ChessEngine::is_enemy_space(board, &pos, color)
                    || board.is_empty_space(&pos) {
                    if !ChessEngine::is_threatened(board, &pos, color) {
                        solutions.push(pos.clone());
                    }
                }
            }
        }
        solutions
    }

    fn can_side_castle(board: &Board, king_pos: &Position, is_left: bool) -> bool {
        let direction = if is_left { -1 } else { 1 };
        let rook_col = if is_left { 0 } else { (board.get_size() as i32) - 1 };
        let mut can_castle =
            match board.get_space(&Position::new(king_pos.row, rook_col)){
                Ok(Option::Some(Piece {kind: _, color: _, has_moved: false})) => true,
                _ => false,
            };
        let mut i = king_pos.col;
        while i > 0 && can_castle {
            i = i + direction;
            match board.get_space(&Position::new(king_pos.row, i)) {
                Ok(Option::Some(_)) => can_castle = false,
                _ => ()
            }
        }
        can_castle
    }

    fn generate_pawn_moves(board: &Board, p: &Position, c: Color, has_moved: bool) -> Vec<Position>{
        // convert color to direction
        let mut solutions = vec![];
        let direction = if c == Color::White { 1 } else { -1 };
        let forward_space = Position::new(direction + p.row, p.col);
        if let Ok(Option::None) = board.get_space(&forward_space) {
            solutions.push(forward_space);
            if has_moved == false {
                let double_forward_space = Position::new(2 * direction + p.row
                                                         , p.col);
                if let Ok(Option::None) = board.get_space(&double_forward_space) {
                    solutions.push(double_forward_space);
                }
            }
        }
        let positive_diagonal_space = Position::new(p.row + direction
                                                    , p.col + 1);
        if ChessEngine::is_enemy_space(board, &positive_diagonal_space, c) {
            solutions.push(positive_diagonal_space);
        }
        let negative_diagonal_space = Position::new(p.row + direction
                                                    , p.col - 1);
        if ChessEngine::is_enemy_space(board, &negative_diagonal_space, c) {
            solutions.push(negative_diagonal_space);
        }
        solutions
    }

    fn is_enemy_space(board: &Board, p: &Position, c: Color) -> bool {
        if let Ok(
            Option::Some(Piece { kind: _, color: move_color, has_moved: _ })
        ) = board.get_space(p) {
            *move_color != c
        } else {
            false
        }
    }

    pub fn execute_move(board: &mut Board, from: &Position, to: &Position)
                        -> Result<Option<Piece>, String> {
        let possibilities = ChessEngine::possible_moves(board, from)?;
        if possibilities.contains(to) {
            // if castle, also move Rook
            if ChessEngine::is_castle(board, from, to) {
                ChessEngine::castle_rook(board, from, to)?;
            }
            Ok(board.move_piece(from, to)?)
        } else {
            Err(String::from("You cannot move to this space"))
        }
    }

    fn apply_move(
        board: &Board,
        sink: &mut Vec<Position>,
        pos: &Position,
        diff: &Position,
        color: &Color,
        repeat: bool,
    ) {
        let check_pos = Position::add(pos, diff);
        println!("Checking Position");
        println!("{:?}", check_pos);
        if let Err(_) = board.validate_position(&check_pos) {
            return;
        }
        if let Ok(space) = board.get_space(&check_pos) {
            match space {
                Option::None => {
                    sink.push(check_pos);
                    if repeat {
                        ChessEngine::apply_move(board, sink, &check_pos, diff, color, repeat);
                    }
                }
                Option::Some(piece) => {
                    if piece.color != *color {
                        sink.push(check_pos);
                    }
                }
            }
        }
    }

    fn get_move_pattern(kind: PieceKind) -> Result<MovePattern, String> {
        match kind {
            PieceKind::King | PieceKind::Pawn => Err(String::from("not supported")),
            PieceKind::Queen => {
                let moves = ChessEngine::expand_with_inverses(vec![
                    Position::new(0, 1),
                    Position::new(1, 0),
                    Position::new(1, 1),
                ]);
                Ok(MovePattern::new(true, moves))
            }
            PieceKind::Rook => {
                let moves = ChessEngine::expand_with_inverses(vec![
                    Position::new(0, 1),
                    Position::new(1, 0),
                ]);
                Ok(MovePattern::new(true, moves))
            }
            PieceKind::Bishop => {
                Ok(MovePattern::new(true, Position::new(1, 1)
                    .yield_all_inverse_positions()))
            }
            PieceKind::Knight => {
                let moves = ChessEngine::expand_with_inverses(vec![
                    Position::new(1, 2),
                    Position::new(2, 1),
                ]);
                Ok(MovePattern::new(false, moves))
            }
        }
    }

    fn expand_with_inverses(positions: Vec<Position>) -> Vec<Position> {
        //positions.iter().map(|pos| pos.yield_all_inverse_positions()).collect()
        let mut expanded = Vec::new();
        for pos in positions.iter() {
            expanded.append(&mut pos.yield_all_inverse_positions());
        }
        expanded
    }

    fn is_castle(board: &Board, from: &Position, to: &Position) -> bool {
        board.get_space(from).unwrap().unwrap().kind == PieceKind::King
            && (from.col - to.col).abs() == 2
    }

    fn castle_rook(board: &mut Board, king_start: &Position, king_dest: &Position) -> Result<(), String> {
        let is_right = king_start.col > king_dest.col;
        let rook_from_col = if is_right {0} else {board.get_size() as i32};
        let rook_to_col = if is_right {king_dest.col - 1} else {king_dest.col + 1};
        ChessEngine::execute_move(board, &Position::new(king_start.row, rook_from_col),
                          &Position::new(king_start.row, rook_to_col))?;
        Ok(())
    }

    fn is_threatened(board: &Board, pos: &Position, color: Color) -> bool {
        let threatened = board.get_piece_positions().iter()
            .filter(|(_, piece)| piece.color != color)
            .flat_map(|(position, _)| ChessEngine::possible_moves(board, position).unwrap())
            .any(|possible_move| possible_move == *pos);
        threatened
    }

    fn setup_basic_board() -> Result<Board, String>{
        let mut b = Board::new(8)?;
        //setup pawns
        b.fill(Some(6), None, Piece::new(PieceKind::Pawn, Color::Black))?;
        b.fill(Some(1), None, Piece::new(PieceKind::Pawn, Color::White))?;

        // populate accepts map of Spot => Piece
        let mut map = HashMap::new();
        // handle rooks
        map.insert(
            Position::new(0, 0),
            Piece::new(PieceKind::Rook, Color::White),
        );
        map.insert(
            Position::new(0, 7),
            Piece::new(PieceKind::Rook, Color::White),
        );
        map.insert(
            Position::new(7, 0),
            Piece::new(PieceKind::Rook, Color::Black),
        );
        map.insert(
            Position::new(7, 7),
            Piece::new(PieceKind::Rook, Color::Black),
        );
        // handle knights
        map.insert(
            Position::new(0, 1),
            Piece::new(PieceKind::Knight, Color::White),
        );
        map.insert(
            Position::new(0, 6),
            Piece::new(PieceKind::Knight, Color::White),
        );
        map.insert(
            Position::new(7, 1),
            Piece::new(PieceKind::Knight, Color::Black),
        );
        map.insert(
            Position::new(7, 6),
            Piece::new(PieceKind::Knight, Color::Black),
        );
        // handle bishops
        map.insert(
            Position::new(0, 2),
            Piece::new(PieceKind::Bishop, Color::White),
        );
        map.insert(
            Position::new(0, 5),
            Piece::new(PieceKind::Bishop, Color::White),
        );
        map.insert(
            Position::new(7, 2),
            Piece::new(PieceKind::Bishop, Color::Black),
        );
        map.insert(
            Position::new(7, 5),
            Piece::new(PieceKind::Bishop, Color::Black),
        );
        // handle queens
        map.insert(
            Position::new(0, 3),
            Piece::new(PieceKind::Queen, Color::White),
        );
        map.insert(
            Position::new(7, 3),
            Piece::new(PieceKind::Queen, Color::Black),
        );
        // handle kings
        map.insert(
            Position::new(0, 4),
            Piece::new(PieceKind::King, Color::White),
        );
        map.insert(
            Position::new(7, 4),
            Piece::new(PieceKind::King, Color::Black),
        );
        b.populate(map)?;
        Ok(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basic_board() -> Result<(), String> {
        let board = ChessEngine::setup_basic_board()?;
        let pos = Position::new(0, 1);
        let res = board.get_space(&pos);
        let op = res?;

        assert_eq!(
            Some(&Piece {
                kind: PieceKind::Knight,
                color: Color::White,
                has_moved: false
            }),
            op
        );
        Ok(())
    }

    #[test]
    fn test_move_piece() -> Result<(), String> {
        let mut board = ChessEngine::setup_basic_board()?;
        let from = Position::new(1, 1);
        let to = Position::new(2, 1);
        ChessEngine::execute_move(&mut board, &from, &to)?;

        let op_to = board.get_space(&to)?;
        let op_from = board.get_space(&from)?;
        assert_eq!(
            Some(&Piece {
                kind: PieceKind::Pawn,
                color: Color::White,
                has_moved: true
            }),
            op_to
        );
        assert_eq!(None, op_from);
        Ok(())
    }

    #[test]
    fn test_possible_moves() -> Result<(), String> {
        let board = ChessEngine::setup_basic_board()?;
        let pos = Position::new(0, 1);
        let possibilities = ChessEngine::possible_moves(&board, &pos)?;

        board.pretty_print();

        println!("{:?}", possibilities);
        assert_eq!(possibilities.len(), 2);
        Ok(())
    }
}
