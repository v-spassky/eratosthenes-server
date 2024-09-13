use crate::app_context::AppContext;
use crate::http::config::CORS_POLICY;
use crate::http::query_params::PasscodeQueryParam;
use crate::rooms::handlers::ws::RoomWsHandler;
use crate::storage::rooms::HashMapRoomsStorage;
use std::net::SocketAddr;
use warp::reply::Reply;
use warp::{ws::Ws, Filter, Rejection};

pub fn room_chat(
    app_context: &AppContext<HashMapRoomsStorage>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("rooms")
        .and(warp::ws())
        .and(warp::path::param::<String>())
        .and(warp::query::<PasscodeQueryParam>())
        .and(warp::addr::remote())
        .map({
            let app_context = app_context.clone();
            move |ws: Ws,
                  room_id,
                  query_params: PasscodeQueryParam,
                  client_ip: Option<SocketAddr>| {
                let app_context = app_context.clone();
                ws.on_upgrade(move |socket| async move {
                    RoomWsHandler::new(
                        app_context,
                        room_id,
                        client_ip,
                        query_params.passcode,
                        socket,
                    )
                    .await
                    .on_user_connected()
                    .await
                })
            }
        })
        .with(CORS_POLICY.clone())
}
