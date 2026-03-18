// Lightweight Tokio-based WebSocket server.
// Only runs when launched with `--tablet`.
//
// One connection at a time (I have only one iPad).
// Receives StrokeBatch / StrokeEnd messages, forwards them via an mpsc
// channel to the GUI thread, which applies them to the AnnotationStore.

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

use crate::protocol::{
    TabletMessage, LaptopMessage, decode_tablet_msg, encode_laptop_msg,
};

/// Tablet Server -> GUI Thread
#[derive(Debug)]
pub enum TabletEvent {
    StrokeBatch(crate::protocol::StrokeBatch),
    StrokeEnd(crate::protocol::StrokeEnd),
}

/// Spawn server on background tokio thread
/// Returns the receiving end of the event channel.
/// Call it once from app::new() when --tablet is set.
pub fn spawn(port: u16) -> mpsc::UnboundedReceiver<TabletEvent> {
    let (tx, rx) = mpsc::unbounded_channel::<TabletEvent>();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async move {
            if let Err(e) = run_server(port, tx).await {
                error!("tablet server error: {e}");
            }
        });
    });

    rx
}

async fn run_server(port: u16, tx: mpsc::UnboundedSender<TabletEvent>) -> Result<()> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;
    info!("tablet server listening on ws://{addr}");

    loop {
        let (stream, peer) = listener.accept().await?;
        info!("tablet connected from {peer}");
        let tx2 = tx.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, tx2).await {
                warn!("tablet connection closed: {e}");
            }
        });
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream, tx: mpsc::UnboundedSender<TabletEvent>,
) -> Result<()> {

    let ws = accept_async(stream).await?;
    let (mut write, mut read) = ws.split();

    while let Some(msg) = read.next().await {
        let msg = msg?;
        match  msg {
            Message::Binary(bytes) => {
                match crate::protocol::decode_tablet_msg(&bytes) {
                    Ok(TabletMessage::StrokeBatch(b)) => {
                        let _ = tx.send(TabletEvent::StrokeBatch(b));
                    }
                    Ok(TabletMessage::StrokeEnd(e)) => {
                        let _ = tx.send(TabletEvent::StrokeEnd(e));
                    }
                    Ok(TabletMessage::Ping) => {
                        let pong = encode_laptop_msg(&LaptopMessage::Pong)?;
                        write.send(Message::Binary(pong.into())).await?;
                    }
                    Err(e) => {
                        warn!("bad message from tablet: {e}");
                    }
                }
            }
            Message::Close(_) => break,
            _ => {} // ignore text / ping frames
        }
    }
    Ok(())
}
