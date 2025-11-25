use crate::handler::main_handler;
use axum::Router;

mod handler;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    if dotenvy::dotenv().is_err() {
        println!("No .env found, using defaults");
    }

    let state = state::GlobalState::new().await?;
    let bind_addr = state.config.bind_addr;

    // build our application with a single route
    let app = Router::new().fallback(main_handler).with_state(state);

    let socket = tokio::net::TcpSocket::new_v4()?;

    socket.set_reuseaddr(true)?;
    assert!(socket.reuseaddr().unwrap());
    socket.bind(bind_addr)?;

    let listener = socket.listen(1024)?;

    println!("ws2http listening on {bind_addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
