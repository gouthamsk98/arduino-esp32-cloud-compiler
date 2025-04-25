use axum::routing::get;
use socketioxide::SocketIo;
use tracing::info;
use tracing_subscriber::FmtSubscriber;
use arduino_esp32_cloud_compiler::socketio::on_connect;
use arduino_esp32_cloud_compiler::compiler::{ get_arduino_cli_path, health_check };

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;
    // Health check for arduino-cli
    match health_check() {
        true => info!("arduino-cli initialized successfully"),
        false => {
            info!("arduino-cli test failed");
            std::process::exit(1);
        }
    }

    let (layer, io) = SocketIo::new_layer();

    io.ns("/", on_connect);
    io.ns("/custom", on_connect);

    let app = axum::Router
        ::new()
        .route(
            "/",
            get(|| async { "alive" })
        )
        .layer(layer);

    info!("Starting server");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
