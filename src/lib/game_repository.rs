use super::board::Board;
use super::errors::GameError;
use rand::{Rng};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::sync::RwLock;

pub struct GameRepository {
    games: HashMap<u32, RwLock<Game>>
}

impl GameRepository {

    pub fn new() -> GameRepository {
        GameRepository {
            games: HashMap::new()
        }
    }

    pub fn get_latest_game_board(&self, id: u32) -> Result<Board, GameError> {
        let game_lock = self.get_game_lock(id)?;
        let game = game_lock.read().unwrap();
        let board = game.state_history.get(0).unwrap();
        Ok((*board).clone())
    }

    pub fn create_game(&mut self, board: Board) -> Result<u32, String> {
        let id = self.get_unique_id()?;
        let state_history = vec![board];
        self.games.insert(id, RwLock::new(Game {id, state_history}));
        Ok(id)
    }

    pub fn update_game(&self, id: u32, board: &Board) -> Result<(), GameError> {
        let game_lock = self.get_game_lock(id)?;
        let mut game = game_lock.write().unwrap();
        game.append_state(board);
        Ok(())
    }

    fn get_game_lock(&self, id: u32) -> Result<&RwLock<Game>, GameError> {
        match self.games.get(&id) {
            Some(game_lock) => Ok(game_lock),
            None => Err(GameError::DoesNotExist)
        }
    }

    fn get_unique_id(&self) -> Result<u32, String> {
        let attempts = 10;
        for _ in 1..attempts {
            let id = GameRepository::generate_random_id();
            if let None = self.games.get(&id) {
                return Ok(id)
            }
        }
        Err(String::from("Could not generate a Game ID"))
    }

    fn generate_random_id() -> u32 {
        let mut rng = rand::thread_rng();
        rng.gen::<u32>()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    id: u32,
    state_history: Vec<Board>,
}

impl Game {
    pub fn append_state(&mut self, board: &Board) {
        self.state_history.insert(0, (*board).clone())
    }
}
