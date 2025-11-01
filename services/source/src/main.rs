mod http;
mod protocol;
mod pty;
mod server;

use axum::{
    extract::Path,
    response::IntoResponse,
    routing::get,
    Router,
};
use clap::Parser;
use http::static_handler;
use server::{websocket::ws_handler, AppState, Config};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "ttyd-rust")]
#[command(about = "Share your terminal over the web", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "7681")]
    port: u16,

    #[arg(short, long, default_value = "0.0.0.0")]
    interface: String,

    #[arg(short = 'W', long)]
    writable: bool,

    #[arg(short, long)]
    credential: Option<String>,

    #[arg(short = 'w', long)]
    cwd: Option<String>,

    #[arg(short = 'O', long)]
    check_origin: bool,

    #[arg(short, long, default_value = "0")]
    max_clients: usize,

    #[arg(short, long)]
    once: bool,

    #[arg(trailing_var_arg = true)]
    command: Vec<String>,
}

async fn static_file_handler_path(Path(path): Path<String>) -> impl IntoResponse {
    static_handler(&path).await
}

async fn static_file_handler_root() -> impl IntoResponse {
    static_handler("/xterm.min.css").await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ttyd_rust=info,tower_http=info".into()),
        )
        .init();

    let args = Args::parse();

    let config = Config {
        port: args.port,
        interface: args.interface.clone(),
        command: if args.command.is_empty() {
            #[cfg(unix)]
            {
                vec!["bash".to_string()]
            }
            #[cfg(windows)]
            {
                vec!["cmd.exe".to_string()]
            }
        } else {
            args.command
        },
        cwd: args.cwd,
        credential: args.credential,
        writable: args.writable,
        check_origin: args.check_origin,
        max_clients: args.max_clients,
        once: args.once,
    };

    info!("Starting ttyd-rust server");
    info!("Command: {:?}", config.command);
    info!("Port: {}", config.port);
    info!(
        "Writable: {}",
        if config.writable { "true" } else { "false" }
    );

    let state = Arc::new(AppState::new(config.clone()));

    let app = Router::new()
        .route("/", get(http::index_handler))
        .route("/token", get(http::token_handler))
        .route("/ws", get(ws_handler))
        .route("/js/*path", get(static_file_handler_path))
        .route("/css/*path", get(static_file_handler_path))
        .route("/xterm.min.css", get(static_file_handler_root))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", config.interface, config.port).parse()?;
    info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
