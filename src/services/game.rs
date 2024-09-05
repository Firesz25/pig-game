use crate::dtos::game::*;
use futures_util::{SinkExt, StreamExt};
use salvo::{
    session::{Session, SessionDepotExt},
    websocket::{Message, WebSocket},
    Depot,
};

fn to_json(message: GameMessageResponse) -> String {
    serde_json::to_string(&message).unwrap()
}

pub fn get_or_init_session(dep: &mut Depot) -> u128 {
    if let Some(session) = dep.session_mut() {
        if let Some(username) = session.get::<u128>("username") {
            return username;
        }
    }
    let mut ses = Session::new();
    let username = ulid::Ulid::new().0;
    ses.insert("username", username).unwrap();
    dep.set_session(ses);
    username
}

pub async fn web_socket_handle(ws: WebSocket, user: u128, game_id: u128) {
    let (mut tx, mut rx) = ws.split();
    match GAME.read().await.get_game(game_id) {
        Ok(s) => match s {
            GameStatus::Waiting(player1) => {
                if *player1 == user {
                    tx.send(Message::text(to_json(
                        GameMessageResponse::WaitingForPlayer,
                    )))
                    .await
                    .unwrap();
                } else {
                    match GAME.write().await.join_to_game(game_id, user) {
                        Ok(_) => {
                            tx.send(Message::text(to_json(GameMessageResponse::Joined)))
                                .await
                                .unwrap();
                        }
                        Err(e) => {
                            tx.send(Message::text(to_json(GameMessageResponse::Error(e))))
                                .await
                                .unwrap();
                        }
                    }
                }
            }
            GameStatus::Playing(player1, player2) => {
                if player1 == &user || player2 == &user {
                    tx.send(Message::text(to_json(GameMessageResponse::Success)))
                        .await
                        .unwrap();
                } else {
                    tx.send(Message::text(to_json(GameMessageResponse::Error(
                        GameError::InternalError,
                    ))))
                    .await
                    .unwrap();
                    tx.close().await.unwrap();
                }
            }
        },
        Err(e) => {
            tx.send(Message::text(to_json(GameMessageResponse::Error(e))))
                .await
                .unwrap();
            return;
        }
    }
    while let Some(msg) = rx.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("websocket error: {:?}", e);
                return;
            }
        };

        if msg.is_close() {
            GAME.write().await.leave_game(game_id, user).unwrap();
            tx.close().await.unwrap();
            return;
        }

        match serde_json::from_slice::<RoolDice>(msg.as_bytes()) {
            Ok(roll) => match GAME.write().await.rool_dice(game_id, roll.player) {
                Ok(score) => {
                    tx.send(Message::text(to_json(GameMessageResponse::Score(score))))
                        .await
                        .unwrap();
                }
                Err(e) => {
                    tx.send(Message::text(to_json(GameMessageResponse::Error(e))))
                        .await
                        .unwrap();
                }
            },
            Err(_e) => {
                tx.send(Message::text(to_json(GameMessageResponse::Error(
                    GameError::InternalError,
                ))))
                .await
                .unwrap();
                tx.close().await.unwrap();
            }
        }
    }
}
