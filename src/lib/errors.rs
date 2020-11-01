
#[derive(Debug)]
pub enum GameError {
    DoesNotExist,
    NotAllowed,
    Internal(String),
}
