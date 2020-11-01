use super::board::{Board, Piece, Position};
use super::engine::{ChessEngine, BoardSetup};
use super::game_repository::GameRepository;
use super::errors::GameError;

use std::collections::HashMap;
use std::sync::RwLock;

pub struct GameController {
    game_repository: RwLock<GameRepository>,
}

impl GameController {

    pub fn new(game_repository: GameRepository) -> GameController {
        GameController {game_repository: RwLock::new(game_repository)}
    }

    pub fn start_game(&self) -> Result<String, String> {
        let board = match ChessEngine::create_board(BoardSetup::Basic) {
            Ok(b) => b,
            Err(msg) => return Err(msg)
        };
        board.pretty_print();
        let game_repo = &mut *self.game_repository.write().unwrap();
        match game_repo.create_game(board) {
            Ok(id) => Ok(id.to_string()),
            Err(msg) => Err(msg),
        }
    }

    pub fn get_game(&self, id: u32) -> Result<String, GameError> {
        let game_repo: &GameRepository = & *self.game_repository.read().unwrap();
        let board = game_repo.get_latest_game_board(id)?;
        match serde_json::to_string(&GameController::get_board_external_rep(&board)) {
            Ok(board_repr) => Ok(board_repr),
            Err(msg) => Err(GameError::Internal(msg.to_string()))
        }
    }

    pub fn play_move(&self, id: u32, src: String,
                     dest: String) -> Result<(), GameError> {
        // validate
        let possible_moves = self.get_piece_move_options_helper(id, &src)?;
        let allowed = possible_moves.iter().any(|m| *m == dest);
        if !allowed {
            return Err(GameError::NotAllowed);
        }

        // execute move
        let game_repo: &GameRepository = & *self.game_repository.read().unwrap();
        let mut board = game_repo.get_latest_game_board(id).unwrap();
        let board_size = board.get_size();
        let src_pos = GameController::convert_space_name_to_position(board_size, &src);
        let dest_pos = GameController::convert_space_name_to_position(board_size, &dest);
        ChessEngine::execute_move(&mut board, &src_pos, &dest_pos);
        game_repo.update_game(id, &board)?;
        Ok(())
    }

    pub fn get_piece_move_options(&self, id: u32, pos_str: &String) -> Result<String, GameError> {

        let moves = self.get_piece_move_options_helper(id, pos_str)?;
        match serde_json::to_string(&moves) {
            Ok(res) => Ok(res),
            Err(err) => Err(GameError::Internal(err.to_string())),
        }
    }

    fn get_piece_move_options_helper(&self, id: u32, pos_str: &String) -> Result<Vec<String>, GameError>{
        let game_repo: &GameRepository = & *self.game_repository.read().unwrap();
        let board = game_repo.get_latest_game_board(id)?;
        let size = board.get_size();
        let position = GameController::convert_space_name_to_position(size, pos_str);
        Ok(
            ChessEngine::possible_moves(&board, &position).unwrap()
            .iter()
            .map(|p| GameController::convert_position_to_space_name(size, p))
            .collect()
        )
    }

    pub fn get_board_external_rep(board: &Board) -> HashMap<String, Piece> {
        let size = board.get_size();
        board.get_piece_positions().iter()
            .map(|(pos, piece)| (GameController::convert_position_to_space_name(size, pos), piece.clone()))
            .collect()
    }

    pub fn convert_position_to_space_name(board_size: i32, position: &Position) -> String {
        let letter = std::char::from_u32((position.col + 65) as u32).unwrap();
        let digit = board_size - position.row;
        format!("{}{}", letter, &digit.to_string())
    }

    pub fn convert_space_name_to_position(board_size: i32,name: &String) -> Position {
        let col = (name.chars().nth(0).unwrap() as u32) as i32 - 65;
        let row = (board_size as i32) - name.chars().nth(1).unwrap().to_string().parse::<i32>().unwrap();
        Position::new(row, col)
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_convert_position_to_space_name() -> Result<(), String> {
        let mut b = Board::new(8)?;
        let pos = Position::new(0, 1);
        assert_eq!(GameController::convert_position_to_space_name(b.get_size(), &pos),
                   String::from("B8"));
        Ok(())
    }

    #[test]
    fn test_convert_space_name_to_position() -> Result<(), String> {
        let mut b = Board::new(8)?;
        let pos_str = String::from("B8");
        assert_eq!(GameController::convert_space_name_to_position(b.get_size(), &pos_str),
                   Position::new( 0, 1));

        Ok(())
    }

    #[test]
    fn test_convert_space_name_to_position_0th() -> Result<(), String> {
        let mut b = Board::new(8)?;
        let pos_str = String::from("E0");
        assert_eq!(GameController::convert_space_name_to_position(b.get_size(), &pos_str),
                   Position::new( 0, 1));

        Ok(())
    }
}