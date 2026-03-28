use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use reqwest::Client;
use serde_json::Value;
use tokio::sync::{Notify, watch};
use tokio::time;

mod track;
use track::Track;

async fn fetch_track(client: &Client, url: &str) -> reqwest::Result<Option<Track>> {
    let data: Value = client.get(url).send().await?.json().await?;
    Ok(data
        .get("recenttracks")
        .and_then(|rt| rt.get("track"))
        .and_then(|tracks| tracks.get(0))
        .and_then(|v| serde_json::from_value(v.clone()).ok()))
}

struct ConnectionGuard {
    connections: Arc<AtomicUsize>,
}

impl ConnectionGuard {
    fn new(connections: Arc<AtomicUsize>, wake: &Notify) -> Self {
        connections.fetch_add(1, Ordering::Relaxed);
        wake.notify_one();
        Self { connections }
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.connections.fetch_sub(1, Ordering::Relaxed);
    }
}

#[derive(Clone)]
struct AppState {
    track_rx: watch::Receiver<Track>,
    connections: Arc<AtomicUsize>,
    wake: Arc<Notify>,
}

async fn poll_lastfm(
    client: Client,
    url: String,
    tx: watch::Sender<Track>,
    connections: Arc<AtomicUsize>,
    wake: Arc<Notify>,
) {
    loop {
        if connections.load(Ordering::Relaxed) == 0 {
            wake.notified().await; // end this wokeness
            continue;
        }

        let mut interval = time::interval(Duration::from_secs(1));
        while connections.load(Ordering::Relaxed) > 0 {
            interval.tick().await;

            let new_track = match fetch_track(&client, &url).await {
                Ok(Some(track)) => track,
                Ok(None) => continue,
                Err(e) => {
                    eprintln!("failed to poll last.fm: {e}");
                    continue;
                }
            };

            tx.send_if_modified(|current| {
                if *current != new_track {
                    *current = new_track;
                    true
                } else {
                    false
                }
            });
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let _guard = ConnectionGuard::new(state.connections, &state.wake);
    let mut rx = state.track_rx;

    let current = rx.borrow_and_update().clone();
    if current != Track::default() && send_track(&mut socket, &current).await.is_err() {
        return;
    }

    loop {
        tokio::select! {
            changed = rx.changed() => {
                if changed.is_err() {
                    break;
                }
                let track = rx.borrow_and_update().clone();
                if send_track(&mut socket, &track).await.is_err() {
                    break;
                }
            }
            _msg = socket.recv() => {
                break;
            }
        }
    }
}

async fn send_track(socket: &mut WebSocket, track: &Track) -> Result<(), axum::Error> {
    let json = serde_json::to_string(track).expect("track serialization should not fail");
    socket.send(Message::Text(json.into())).await
}

#[tokio::main]
async fn main() {
    println!(concat!("🎶 nowplaying v", env!("CARGO_PKG_VERSION")));

    dotenvy::dotenv().ok();

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let lastfm_user = env::var("LASTFM_USER").expect("LASTFM_USER must be set");
    let lastfm_api_key = env::var("LASTFM_API_KEY").expect("LASTFM_API_KEY must be set");
    let lastfm_url = format!(
        "https://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user={lastfm_user}&api_key={lastfm_api_key}&format=json"
    );

    let (tx, rx) = watch::channel(Track::default());
    let connections = Arc::new(AtomicUsize::new(0));
    let wake = Arc::new(Notify::new());

    tokio::spawn(poll_lastfm(
        Client::new(),
        lastfm_url,
        tx,
        connections.clone(),
        wake.clone(),
    ));

    let state = AppState {
        track_rx: rx,
        connections,
        wake,
    };

    let app = Router::new()
        .route(
            "/",
            get(|| async {
                concat!(
                    "🎶 nowplaying v",
                    env!("CARGO_PKG_VERSION"),
                    "\n",
                    "🔌 connect to /ws over websocket\n",
                )
            }),
        )
        .route("/ws", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}"))
        .await
        .expect("failed to bind to address");

    println!("listening on {host}:{port}");
    axum::serve(listener, app).await.unwrap();
}
