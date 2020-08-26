mod board;
use board::{Board, Color, Piece, PieceKind, Position};
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
    captured_pieces: Vec<Piece>,
}

impl ChessEngine {
    pub fn new(setup: BoardSetup) -> ChessEngine {
        let board = match setup {
            BoardSetup::Basic => {
                let mut b = Board::new(8);
                //setup pawns
                b.fill(Some(6), None, Piece::new(PieceKind::Pawn, Color::Black));
                b.fill(Some(1), None, Piece::new(PieceKind::Pawn, Color::White));

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
                b.populate(map);
                b
            }
        };
        ChessEngine {
            board,
            captured_pieces: Vec::new(),
        }
    }

    pub fn possible_moves(&self, p: &Position) -> Result<Vec<Position>, String> {
        let mut solution = vec![];
        match self.board.get_space(p)? {
            Option::None => (),
            Option::Some(Piece { kind, color }) => {
                let pattern = self.get_move_pattern(*kind);
                for diff in pattern.move_enumerations.iter() {
                    // add logic for repeatedly adding
                    self.apply_move(&mut solution, p, diff, color, pattern.is_repeatable)
                }
            }
        }
        Ok(solution)
    }

    pub fn execute_move(&mut self, from: &Position, to: &Position) -> Result<(), String> {
        let possibilities = self.possible_moves(from)?;
        if possibilities.contains(to) {
            Ok(self.board.move_piece(from, to)?)
        } else {
            Err("You cannot move to this space".to_string())
        }
    }

    fn apply_move(
        &self,
        sink: &mut Vec<Position>,
        pos: &Position,
        diff: &Position,
        color: &Color,
        repeat: bool,
    ) {
        let check_pos = Position::add(pos, diff);
        println!("Checking Position");
        println!("{:?}", check_pos);
        if check_pos.row < 0 || check_pos.row >= 8 || check_pos.row < 0 || check_pos.row >= 8 {
            return;
        }
        if let Ok(space) = self.board.get_space(&check_pos) {
            match space {
                Option::None => {
                    sink.push(check_pos);
                    if repeat {
                        self.apply_move(sink, &check_pos, diff, color, repeat);
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

    // make into a static map
    // should return a struct that contains a list of patterns, and a boolean of whether they are repeatable within a move
    fn get_move_pattern(&self, kind: PieceKind) -> MovePattern {
        match kind {
            PieceKind::King => {
                // more complex with Castle case
                let moves = ChessEngine::expand_with_inverses(vec![
                    Position::new(0, 1),
                    Position::new(1, 0),
                    Position::new(1, 1),
                ]);
                MovePattern::new(false, moves)
            }
            PieceKind::Queen => {
                let moves = ChessEngine::expand_with_inverses(vec![
                    Position::new(0, 1),
                    Position::new(1, 0),
                    Position::new(1, 1),
                ]);
                MovePattern::new(true, moves)
            }
            PieceKind::Rook => {
                let moves = ChessEngine::expand_with_inverses(vec![
                    Position::new(0, 1),
                    Position::new(1, 0),
                ]);
                MovePattern::new(true, moves)
            }
            PieceKind::Bishop => {
                MovePattern::new(true, Position::new(1, 1).yield_all_inverse_positions())
            }
            PieceKind::Knight => {
                let moves = ChessEngine::expand_with_inverses(vec![
                    Position::new(1, 2),
                    Position::new(2, 1),
                ]);
                MovePattern::new(false, moves)
            }
            PieceKind::Pawn => {
                // irregular with different for move into open space vs attacking
                MovePattern::new(
                    false,
                    vec![
                        Position::new(1, 0),
                        Position::new(1, -1),
                        Position::new(1, 1),
                    ],
                )
            }
        }
    }

    fn expand_with_inverses(positions: Vec<Position>) -> Vec<Position> {
        let mut expanded = Vec::new();
        for pos in positions.iter() {
            expanded.append(&mut pos.yield_all_inverse_positions());
        }
        expanded
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basic_board() -> Result<(), String> {
        let engine = ChessEngine::new(BoardSetup::Basic);
        let pos = Position::new(0, 1);
        let res = engine.board.get_space(&pos);
        let op = res?;

        assert_eq!(
            Some(Piece {
                kind: PieceKind::Knight,
                color: Color::White
            }),
            *op
        );
        Ok(())
    }

    #[test]
    fn test_move_piece() -> Result<(), String> {
        let mut engine = ChessEngine::new(BoardSetup::Basic);
        let from = Position::new(1, 1);
        let to = Position::new(2, 1);
        engine.execute_move(&from, &to);

        let op_to = engine.board.get_space(&to)?;
        let op_from = engine.board.get_space(&from)?;
        assert_eq!(
            Some(Piece {
                kind: PieceKind::Pawn,
                color: Color::White
            }),
            *op_to
        );
        assert_eq!(None, *op_from);
        Ok(())
    }

    #[test]
    fn test_possible_moves() -> Result<(), String> {
        let engine = ChessEngine::new(BoardSetup::Basic);
        let pos = Position::new(1, 1);
        let possibilities = engine.possible_moves(&pos)?;

        println!("{:?}", possibilities);
        assert_eq!(possibilities.len(), 2);
        Ok(())
    }
}
