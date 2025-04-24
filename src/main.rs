use axum::{ extract::Json, routing::{ get, post }, http::StatusCode, Router };
use serde::{ Deserialize, Serialize };
use std::process::Command;
use std::path::{ Path, PathBuf };
use tokio::process::Command as TokioCommand;
use tower_http::cors::{ Any, CorsLayer };
use std::net::SocketAddr;

// Path to the arduino-cli binary
#[cfg(target_os = "linux")]
static ARDUINO_CLI_BINARY: &[u8] = include_bytes!("../resource/linux/arduino-cli"); // Change this if needed
#[cfg(target_os = "windows")]
static ARDUINO_CLI_BINARY: &[u8] = include_bytes!("../resource/windows/arduino-cli.exe"); // Change this if needed
#[cfg(target_os = "macos")]
static ARDUINO_CLI_BINARY: &[u8] = include_bytes!("../resource/macos/arduino-cli"); // Change this if needed
static ARDUINO_CLI_PATH: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

// Function to initialize the arduino-cli binary
fn initialize_arduino_cli() -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let arduino_cli_path = temp_dir.join("arduino-cli-embedded");

    // Write the binary to a temporary location
    std::fs
        ::write(&arduino_cli_path, ARDUINO_CLI_BINARY)
        .expect("Failed to write arduino-cli binary to disk");

    // Make it executable on Unix-like systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs
            ::metadata(&arduino_cli_path)
            .expect("Failed to get metadata")
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&arduino_cli_path, perms).expect("Failed to set permissions");
    }

    arduino_cli_path
}
// Get the path to the arduino-cli binary
fn get_arduino_cli_path() -> &'static PathBuf {
    ARDUINO_CLI_PATH.get_or_init(initialize_arduino_cli)
}

#[tokio::main]
async fn main() {
    // Initialize arduino-cli
    let arduino_cli_path = get_arduino_cli_path();
    println!("Using arduino-cli at: {}", arduino_cli_path.display());

    // Test if arduino-cli works
    let test_result = Command::new(arduino_cli_path).arg("version").output();
    match test_result {
        Ok(output) => {
            if output.status.success() {
                println!("arduino-cli initialized successfully:");
                println!("{}", String::from_utf8_lossy(&output.stdout));
            } else {
                eprintln!("arduino-cli test failed: {}", String::from_utf8_lossy(&output.stderr));
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to execute arduino-cli: {}", e);
            std::process::exit(1);
        }
    }
    // Set up CORS
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    // Build our application with routes
    let app = Router::new()
        .route("/", get(health_check))
        .route("/boards/list", get(list_boards))
        .route("/boards/connected", get(list_connected_boards))
        .route("/cores/list", get(list_cores))
        .route("/cores/install", post(install_core))
        .route("/sketch/compile", post(compile_sketch))
        .route("/sketch/upload", post(upload_sketch))
        .layer(cors);

    // Run our app
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Arduino CLI API server starting on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}

// Handler for health check endpoint
async fn health_check() -> &'static str {
    "Arduino CLI API is running!"
}

// Response structures
#[derive(Serialize)]
struct CommandResponse {
    success: bool,
    output: String,
    error: Option<String>,
}

// Request structures
#[derive(Deserialize)]
struct CoreInstallRequest {
    core_name: String,
}

#[derive(Deserialize)]
struct CompileRequest {
    sketch_path: String,
    fqbn: Option<String>,
}

#[derive(Deserialize)]
struct UploadRequest {
    sketch_path: String,
    port: String,
    fqbn: String,
}

// Helper function to run Arduino CLI commands
async fn run_arduino_command(args: Vec<&str>) -> Result<CommandResponse, StatusCode> {
    let arduino_cli_path = get_arduino_cli_path();

    let output = TokioCommand::new(arduino_cli_path)
        .args(&args)
        .output().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(CommandResponse {
        success: output.status.success(),
        output: stdout,
        error: if stderr.is_empty() {
            None
        } else {
            Some(stderr)
        },
    })
}
// Handler to list all boards
async fn list_boards() -> Result<Json<CommandResponse>, StatusCode> {
    let response = run_arduino_command(vec!["board", "listall", "--format", "json"]).await?;
    Ok(Json(response))
}

// Handler to list connected boards
async fn list_connected_boards() -> Result<Json<CommandResponse>, StatusCode> {
    let response = run_arduino_command(vec!["board", "list", "--format", "json"]).await?;
    Ok(Json(response))
}

// Handler to list installed cores
async fn list_cores() -> Result<Json<CommandResponse>, StatusCode> {
    let response = run_arduino_command(vec!["core", "list", "--format", "json"]).await?;
    Ok(Json(response))
}

// Handler to install a core
async fn install_core(Json(payload): Json<CoreInstallRequest>) -> Result<
    Json<CommandResponse>,
    StatusCode
> {
    let response = run_arduino_command(vec!["core", "install", &payload.core_name]).await?;
    Ok(Json(response))
}

// Handler to compile a sketch
async fn compile_sketch(Json(payload): Json<CompileRequest>) -> Result<
    Json<CommandResponse>,
    StatusCode
> {
    let mut args = vec!["compile"];

    // if let Some(fqbn) = payload.fqbn {
    //     args.push("--fqbn");
    //     args.push(fqbn.clone.as_str());
    // }

    args.push(&payload.sketch_path);

    let response = run_arduino_command(args).await?;
    Ok(Json(response))
}

// Handler to upload a sketch
async fn upload_sketch(Json(payload): Json<UploadRequest>) -> Result<
    Json<CommandResponse>,
    StatusCode
> {
    let args = vec![
        "upload",
        "--port",
        &payload.port,
        "--fqbn",
        &payload.fqbn,
        &payload.sketch_path
    ];

    let response = run_arduino_command(args).await?;
    Ok(Json(response))
}
