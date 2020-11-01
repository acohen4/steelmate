use std::collections::HashMap;
use std::collections::HashSet;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Color {
    Black,
    White,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
    pub has_moved: bool,
}

impl Piece {
    pub fn new(kind: PieceKind, color: Color) -> Piece {
        Piece { kind, color, has_moved: false }
    }

    pub fn get_pretty_str(&self) -> String {
        match self.color {
            Color::White => {
                match self.kind {
                    PieceKind::King => String::from("♔"),
                    PieceKind::Queen => String::from("♕"),
                    PieceKind::Rook => String::from("♖"),
                    PieceKind::Bishop => String::from("♗"),
                    PieceKind::Knight => String::from("♘"),
                    PieceKind::Pawn => String::from("♙"),
                }
            }
            Color::Black => {
                match self.kind {
                    PieceKind::King => String::from("♚"),
                    PieceKind::Queen => String::from("♛"),
                    PieceKind::Rook => String::from("♜"),
                    PieceKind::Bishop => String::from("♝"),
                    PieceKind::Knight => String::from("♞"),
                    PieceKind::Pawn => String::from("♟︎"),
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Position {
    pub row: i32,
    pub col: i32,
}

impl Position {
    pub fn new(row: i32, col: i32) -> Position {
        Position { row, col }
    }

    pub fn add(p1: &Position, p2: &Position) -> Position {
        Position::new(p1.row.clone() + p2.row.clone(), p1.col.clone() + p2.col.clone())
    }

    pub fn yield_all_inverse_positions(&self) -> Vec<Position> {
        let scalars = vec![-1, 1];
        let mut set = HashSet::new();
        for i in scalars.iter() {
            for j in scalars.iter() {
                let row = i * self.row.clone();
                let col = j * self.col.clone();
                set.insert(Position::new(row, col));
            }
        }
        set.iter().map(|x| x.clone()).collect()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Board {
    size: i32,
    board: HashMap<Position, Piece>,
}

impl Board {
    pub fn new(size: i32) -> Result<Board, String> {
        if size < 0 {
            Err("cannot have a negative size".to_string())
        } else {
            Ok(Board {
                size: size,
                board: HashMap::new(),
            })
        }
    }

    pub fn get_piece_positions(&self) -> &HashMap<Position, Piece> {
        &self.board
    }

    pub fn fill(&mut self, row: Option<i32>, column: Option<i32>,
                piece: Piece) -> Result<(), String> {
        let mut update_positions = vec![];
        match row {
            Some(r_index) => match column {
                Some(c_index) => update_positions.push(Position::new(r_index, c_index)),
                None => {
                    for i in 0..self.size {
                        update_positions.push(Position::new(r_index, i));
                    }
                }
            },
            None => match column {
                Some(c_index) => {
                    for i in 0..self.size {
                        update_positions.push(Position::new(i, c_index));
                        update_positions.push(Position::new(i, c_index));
                    }
                }
                None => (),
            },
        }
        for pos in update_positions.iter() {
            self.set_space(pos, Some(piece.clone()))?;
        }
        Ok(())
    }

    pub fn populate(&mut self, setup_map: HashMap<Position, Piece>) -> Result<(), String>{
        for (pos, piece) in setup_map {
            self.set_space(&pos, Some(piece))?;
        }
        Ok(())
    }

    pub fn move_piece(&mut self, from: &Position, to: &Position) -> Result<Option<Piece>, String> {
        match self.board.remove(from) {
            Option::None => Err("The from space does not contain a piece to move".to_string()),
            Option::Some(mut from_piece) => {
                from_piece.has_moved = true;
                let res = match self.set_space(to, Some(from_piece))? {
                    Option::None => Ok(Option::None),
                    Option::Some(to_piece) => Ok(Option::Some(to_piece))
                };
                res
            }
        }
    }

    pub fn get_space(&self, p: &Position) -> Result<Option<&Piece>, String> {
        self.validate_position(p)?;
        Ok(self.board.get(p))
    }

    pub fn is_empty_space(&self, p: &Position) -> bool {
        match self.get_space(p) {
            Ok(Option::None) => true,
            _ => false,
        }
    }

    pub fn pretty(&self) -> String {
        let mut res = String::new();
        res.push_str(&format!("{}\n", &self.get_chess_row_boarder_string()));
        for row in 0..self.size {
            let mut row_string = String::from("|");
            for col in 0..self.size {
                let symbol = match self.board.get(&Position::new(row, col)) {
                    Option::None => String::from(" "),
                    Option::Some(piece) => piece.get_pretty_str()
                };
                row_string.push_str("  ");
                row_string.push_str(&symbol);
                if col < self.size {
                    row_string.push_str("  |");
                }
            }
            res.push_str(&format!("{}\n", &row_string));
            res.push_str(&format!("{}\n", &self.get_chess_row_boarder_string()));
        }
        res
    }

    pub fn pretty_print(&self) {
        println!("{}", &self.pretty());
    }

    pub fn get_size(&self) -> i32 {
        self.size
    }

    fn get_chess_row_boarder_string(&self) -> String {
        format!("{}", "------".repeat(self.size as usize))
    }

    fn set_space(&mut self, p: &Position, piece: Option<Piece>) -> Result<Option<Piece>, String> {
        self.validate_position(p)?;
        match piece {
            Option::None => Ok(self.board.remove(p)),
            Option::Some(piece) => Ok(self.board.insert(p.clone(), piece))
        }
    }

    pub fn validate_position(&self, p: &Position) -> Result<(), String> {
        if Board::is_out_of_bounds(p.col, 0, self.size)
            || Board::is_out_of_bounds(p.row, 0, self.size) {
            Err(String::from("Index out of bounds"))
        } else {
            Ok(())
        }
    }

    fn is_out_of_bounds<T: Ord>(val: T, lower: T, upper: T) -> bool {
        val < lower || val >= upper
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_board() -> Result<(), String> {
        let mut b = Board::new(8)?;

        let pos = Position::new(0, 1);
        let mut op = b.get_space(&pos)?;
        assert_eq!(None, op);

        b.fill(None, Some(1), Piece::new(PieceKind::Pawn, Color::White))?;

        op = b.get_space(&pos)?;
        assert_eq!(
            Some(&Piece { kind: PieceKind::Pawn, color: Color::White, has_moved: false }),
            op
        );

        b.set_space(&pos, None)?;
        op = b.get_space(&pos)?;
        assert_eq!(None, op);

        b.fill(Some(1), None, Piece::new(PieceKind::Pawn, Color::Black))?;

        b.pretty_print();

        Ok(())
    }
}
