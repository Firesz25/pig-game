mod dtos;
mod routes;
mod services;

use salvo::prelude::*;
use salvo::session::CookieStore;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let session_handler = SessionHandler::builder(
        CookieStore::new(),
        b"sufli64537246n5cq5c45cq6543^C%#@#^%$C^T%@^%Cdcfscnyfow34c5w34%#$@^RGWxdfzxvsvss",
    )
    .build()
    .unwrap();
    let router = Router::new().hoop(session_handler).push(
        Router::with_path("game")
            .goal(routes::game::game_ws)
            .get(routes::game::new_game),
    );
    let router2;
    if std::env::var("DEVELOPMENT").unwrap() == "true" {
        let doc =
            OpenApi::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")).merge_router(&router);
        router2 = router
            .push(doc.into_router("/api-docs/openapi.json"))
            .push(RapiDoc::new("/api-docs/openapi.json").into_router("/api-docs/rapi-doc"));
    } else {
        router2 = router;
    }

    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(acceptor).serve(router2).await;
}
