use serde_json::Value;
use socketioxide::{ extract::{ AckSender, Data, SocketRef }, SocketIo };
use tracing::info;
use crate::models::*;
use crate::compiler::run_arduino_command;

pub fn on_connect(socket: SocketRef, Data(data): Data<Value>) {
    info!(ns = socket.ns(), ?socket.id, "Socket.IO connected");
    socket.emit("auth", &data).ok();

    socket.on("message", |Data::<Value>(data), socket: SocketRef| {
        info!(?data, "Received event:");
        socket.emit("message-back", &data).ok();
    });

    socket.on("message-with-ack", |Data::<Value>(data), ack: AckSender| {
        info!(?data, "Received event");
        ack.send(&data).ok();
    });
    // Specific commands for common Arduino CLI operations
    register_arduino_handlers(&socket);
}

// Register specific handlers for common Arduino CLI operations
fn register_arduino_handlers(socket: &SocketRef) {
    // List all available boards
    socket.on("list-boards", |ack: AckSender| {
        tokio::spawn(async move {
            let command = ArduinoCommand {
                command: "board".to_string(),
                args: vec!["listall".to_string(), "--format".to_string(), "json".to_string()],
            };

            let response = run_arduino_command(&command).await;
            ack.send(&response).ok();
        });
    });

    // List connected boards
    socket.on("list-connected", |ack: AckSender| {
        tokio::spawn(async move {
            let command = ArduinoCommand {
                command: "board".to_string(),
                args: vec!["list".to_string(), "--format".to_string(), "json".to_string()],
            };

            let response = run_arduino_command(&command).await;
            ack.send(&response).ok();
        });
    });

    // List installed cores
    socket.on("list-cores", |ack: AckSender| {
        tokio::spawn(async move {
            let command = ArduinoCommand {
                command: "core".to_string(),
                args: vec!["list".to_string(), "--format".to_string(), "json".to_string()],
            };

            let response = run_arduino_command(&command).await;
            ack.send(&response).ok();
        });
    });

    // Install a core
    socket.on("install-core", |Data::<Value>(data), ack: AckSender| {
        let core_name = match data.get("core").and_then(|v| v.as_str()) {
            Some(name) => name.to_string(),
            None => {
                let error_response = CommandResponse {
                    success: false,
                    output: String::new(),
                    error: Some("Missing core name".to_string()),
                    command: "core".to_string(),
                    args: vec!["install".to_string()],
                };
                ack.send(&error_response).ok();
                return;
            }
        };

        tokio::spawn(async move {
            let command = ArduinoCommand {
                command: "core".to_string(),
                args: vec!["install".to_string(), core_name],
            };

            let response = run_arduino_command(&command).await;
            ack.send(&response).ok();
        });
    });

    // Compile a sketch
    socket.on("compile-sketch", |Data::<Value>(data), ack: AckSender| {
        // Extract sketch path and optional FQBN
        let sketch_path = match data.get("sketch_path").and_then(|v| v.as_str()) {
            Some(path) => path.to_string(),
            None => {
                let error_response = CommandResponse {
                    success: false,
                    output: String::new(),
                    error: Some("Missing sketch path".to_string()),
                    command: "compile".to_string(),
                    args: vec![],
                };
                ack.send(&error_response).ok();
                return;
            }
        };

        let mut args = vec![];

        // Add FQBN if provided
        if let Some(fqbn) = data.get("fqbn").and_then(|v| v.as_str()) {
            args.push("--fqbn".to_string());
            args.push(fqbn.to_string());
        }

        args.push(sketch_path);

        tokio::spawn(async move {
            let command = ArduinoCommand {
                command: "compile".to_string(),
                args,
            };

            let response = run_arduino_command(&command).await;
            ack.send(&response).ok();
        });
    });

    // Upload a sketch
    socket.on("upload-sketch", |Data::<Value>(data), ack: AckSender| {
        let sketch_path = match data.get("sketch_path").and_then(|v| v.as_str()) {
            Some(path) => path.to_string(),
            None => {
                let error_response = CommandResponse {
                    success: false,
                    output: String::new(),
                    error: Some("Missing sketch path".to_string()),
                    command: "upload".to_string(),
                    args: vec![],
                };
                ack.send(&error_response).ok();
                return;
            }
        };

        let port = match data.get("port").and_then(|v| v.as_str()) {
            Some(port) => port.to_string(),
            None => {
                let error_response = CommandResponse {
                    success: false,
                    output: String::new(),
                    error: Some("Missing port".to_string()),
                    command: "upload".to_string(),
                    args: vec![],
                };
                ack.send(&error_response).ok();
                return;
            }
        };

        let fqbn = match data.get("fqbn").and_then(|v| v.as_str()) {
            Some(fqbn) => fqbn.to_string(),
            None => {
                let error_response = CommandResponse {
                    success: false,
                    output: String::new(),
                    error: Some("Missing FQBN".to_string()),
                    command: "upload".to_string(),
                    args: vec![],
                };
                ack.send(&error_response).ok();
                return;
            }
        };

        let args = vec!["--port".to_string(), port, "--fqbn".to_string(), fqbn, sketch_path];

        tokio::spawn(async move {
            let command = ArduinoCommand {
                command: "upload".to_string(),
                args,
            };

            let response = run_arduino_command(&command).await;
            ack.send(&response).ok();
        });
    });
}
