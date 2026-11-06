use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade, Message},
    response::Response,
};
use futures::{StreamExt, SinkExt};


use crate::gateway::GatewayState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<GatewayState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_ws(socket, state.tx.clone()))
}

async fn handle_ws(socket: WebSocket, tx: tokio::sync::broadcast::Sender<String>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(_))) = receiver.next().await {}
    });

    let _ = tokio::join!(send_task, recv_task);
}
