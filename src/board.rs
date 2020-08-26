use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryInto;
use std::mem;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    Black,
    White,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
}

impl Piece {
    pub fn new(kind: PieceKind, color: Color) -> Piece {
        Piece { kind, color }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Position {
    pub row: isize,
    pub col: isize,
}

impl Position {
    pub fn new(row: isize, col: isize) -> Position {
        Position { row, col }
    }

    pub fn add(p1: &Position, p2: &Position) -> Position {
        Position::new(p1.row + p2.row, p1.col + p2.col)
    }

    pub fn yield_all_inverse_positions(&self) -> Vec<Position> {
        let scalars = vec![-1, 1];
        let mut set = HashSet::new();
        for i in scalars.iter() {
            for j in scalars.iter() {
                let row = i * self.row;
                let col = j * self.col;
                set.insert(Position::new(row, col));
            }
        }
        set.iter().map(|x| x.clone()).collect()
    }
}

pub struct Board {
    size: isize,
    board: Vec<Option<Piece>>,
}

impl Board {
    pub fn new(size: isize) -> Board {
        Board {
            size,
            board: vec![Option::None; size.checked_pow(2).unwrap().try_into().unwrap()],
        }
    }

    pub fn fill(&mut self, row: Option<isize>, column: Option<isize>, piece: Piece) {
        let mut update_positions = vec![];
        match row {
            Some(r_index) => match column {
                Some(c_index) => update_positions.push(Position::new(r_index, c_index)),
                None => {
                    for i in 0..self.size - 1 {
                        update_positions.push(Position::new(r_index, i));
                    }
                }
            },
            None => match column {
                Some(c_index) => {
                    for i in 0..self.size - 1 {
                        update_positions.push(Position::new(i, c_index));
                    }
                }
                None => (),
            },
        }
        for pos in update_positions.iter() {
            self.set_space(pos, Some(piece));
        }
    }

    pub fn populate(&mut self, setup_map: HashMap<Position, Piece>) {
        for (pos, piece) in &setup_map {
            self.set_space(pos, Some(*piece));
        }
    }

    pub fn get_space(&self, p: &Position) -> Result<&Option<Piece>, String> {
        match self.board.get(self.calculate_element_index(p)?) {
            Option::None => Err("Index out of bounds on the chess board".to_string()),
            Option::Some(space) => Ok(space),
        }
    }

    pub fn move_piece(&mut self, from: &Position, to: &Position) -> Result<(), String> {
        let from_space = self.get_space(from)?;
        let to_space = self.get_space(to)?;

        match from_space {
            Option::None => Err("The from space does not contain a piece to move".to_string()),
            Option::Some(_) => match to_space {
                Option::None => {
                    self.swap_spaces(from, to)?;
                    self.set_space(from, Option::None)?;
                    Ok(())
                }
                Option::Some(_) => {
                    self.swap_spaces(from, to)?;
                    self.set_space(from, Option::None)?;
                    Ok(())
                }
            },
        }
    }

    fn calculate_element_index(&self, p: &Position) -> Result<usize, String> {
        let index = if p.row >= 0 && p.col >= 0 {
            p.row * self.size + p.col
        } else {
            (self.size - p.row) * self.size + self.size -1 - p.col
        };

        if index < self.size * self.size {
            Ok(index.try_into().unwrap())
        } else {
            Err(format!(
                "Neither the position's horizontal or vertical component can exceed {}",
                self.size
            ))
        }
    }

    fn lift_piece(&mut self, p: &Position) -> Result<Option<Piece>, String> {
        self.set_space(p, Option::None)
    }

    fn set_space(&mut self, p: &Position, piece: Option<Piece>) -> Result<Option<Piece>, String> {
        let index = self.calculate_element_index(p)?;
        match self.board.get_mut(index) {
            Option::None => Err("Index out of bounds".to_string()),
            Option::Some(p) => Ok(mem::replace(p, piece)),
        }
    }

    fn swap_spaces(&mut self, p1: &Position, p2: &Position) -> Result<(), String> {
        let p_1 = self.lift_piece(p1)?;
        let p_2 = self.set_space(p2, p_1)?;
        self.set_space(p1, p_2)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_board() -> Result<(),String>{
        let mut b = Board::new(8);

        let pos = Position::new(0,1);
        let mut op = b.get_space(&pos)?;
        assert_eq!(None, *op);

        b.fill(None, Some(1), Piece::new(PieceKind::Pawn, Color::White));
        
        op = b.get_space(&pos)?;
        assert_eq!(Some(Piece{kind: PieceKind::Pawn, color: Color::White}), *op);

        b.set_space(&pos, None);
        op = b.get_space(&pos)?;
        assert_eq!(None, *op);

        Ok(())
    }
}
