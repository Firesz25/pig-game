use rand::Rng;
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use thiserror::Error;
use tokio::sync::RwLock;

use super::player::Player;

pub static GAME: LazyLock<RwLock<Game>> = LazyLock::new(|| RwLock::new(Game::new()));

pub struct Game {
    games: HashMap<u128, GameStatus>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub enum GameMessage {
    NewGame,
    JoinGame(JoinGame),
    RoolDice(RoolDice),
}

#[derive(Serialize, ToSchema)]
pub enum GameMessageResponse {
    Success,
    Joined,
    WaitingForPlayer,
    Score((u8, u8)),
    Error(GameError),
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct NewGame {
    pub game_id: u128,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct JoinGame {
    pub player2: u128,
    pub game_id: u128,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RoolDice {
    pub game_id: u128,
    pub player: u128,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RoolDiceResult {
    pub game_id: u128,
    pub player: u128,
    pub dice: u8,
}

pub enum GameStatus {
    Waiting(u128),
    Playing(Player, Player),
}

#[derive(Error, Debug, Serialize, ToSchema)]
pub enum GameError {
    #[error("Game not found")]
    GameNotFound,
    #[error("Game already started")]
    GameAlreadyStarted,
    #[error("internal error")]
    InternalError,
}

impl Game {
    pub fn new() -> Game {
        Game {
            games: HashMap::new(),
        }
    }

    pub fn new_game(&mut self, game_id: u128, player1: u128) {
        self.games.insert(game_id, GameStatus::Waiting(player1));
    }

    pub fn get_game(&self, game_id: u128) -> Result<&GameStatus, GameError> {
        self.games.get(&game_id).ok_or(GameError::GameNotFound)
    }

    pub fn join_to_game(&mut self, game_id: u128, player2: u128) -> Result<(), GameError> {
        if let Some(status) = self.games.get_mut(&game_id) {
            match status {
                GameStatus::Waiting(player1) => {
                    *status = GameStatus::Playing(Player::new(*player1), Player::new(player2));
                    Ok(())
                }
                _ => Err(GameError::GameAlreadyStarted),
            }
        } else {
            Err(GameError::GameNotFound)
        }
    }

    pub fn leave_game(&mut self, game_id: u128, player: u128) -> Result<(), GameError> {
        if let Some(status) = self.games.get_mut(&game_id) {
            match status {
                GameStatus::Waiting(player1) => {
                    if *player1 == player {
                        self.games.remove(&game_id);
                        return Ok(());
                    }
                }
                GameStatus::Playing(player1, player2) => {
                    if *player1 == player || *player2 == player {
                        self.games.remove(&game_id);
                        return Ok(());
                    }
                }
            }
        }
        Err(GameError::GameNotFound)
    }

    pub fn rool_dice(&mut self, game_id: u128, player: u128) -> Result<(u8, u8), GameError> {
        if let Some(status) = self.games.get_mut(&game_id) {
            match status {
                GameStatus::Playing(player1, player2) => {
                    let num: u8 = rand::thread_rng().gen_range(1..=6);
                    if player1 == &player {
                        player1.score += num;
                        return Ok((player1.score, num));
                    } else if player2 == &player {
                        player2.score += num;
                        return Ok((player2.score, num));
                    }
                }
                _ => return Err(GameError::InternalError),
            }
        }
        Err(GameError::GameNotFound)
    }
}
