use crate::{
    dtos::game::GAME,
    services::game::{get_or_init_session, web_socket_handle},
};
use salvo::prelude::*;
use ulid::Ulid;

#[endpoint]
pub async fn game_ws(req: &mut Request, res: &mut Response, dep: &mut Depot) {
    let user = get_or_init_session(dep);
    let game_id = req.query("id").unwrap();
    WebSocketUpgrade::new()
        .upgrade(req, res, move |w| web_socket_handle(w, user, game_id))
        .await
        .unwrap();
}

#[endpoint]
pub async fn new_game(res: &mut Response, dep: &mut Depot) {
    let user = get_or_init_session(dep);
    let game_id = Ulid::new().0;
    GAME.write().await.new_game(game_id, user);
    res.render(game_id.to_string());
}
