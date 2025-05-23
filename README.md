# Arduino ESP32 Cloud/Desktop Compiler

A cloud-based compiler service for Arduino ESP32 development that provides a Socket.IO interface to the Arduino CLI.

## Overview

Arduino ESP32 Cloud Compiler is a server application that embeds the Arduino CLI tool and exposes its functionality through a Socket.IO API. This allows web applications and other clients to perform Arduino operations like listing boards, compiling sketches, and uploading to devices without requiring Arduino CLI to be installed on the client machine.

The compiler can be used in two primary ways:

1. As a cloud-based service running on a remote server
2. As a desktop compiler running as a daemon in the background, providing local development environments with Arduino CLI capabilities without manual installation and setup

## Features

- Cross-platform support (Windows, macOS, Linux)
- Built-in Arduino CLI binary (no separate installation required)
- Socket.IO API for real-time communication
- Support for common Arduino operations:
  - List all available boards
  - List connected boards
  - List and install cores
  - Compile sketches
  - Upload sketches to devices

## Installation

### Prerequisites

- Rust and Cargo (for building from source)

### Building from Source

1. Clone the repository:

   ```bash
   git clone https://github.com/gouthamsk98/arduino-esp32-cloud-compiler.git
   cd arduino-esp32-cloud-compiler
   ```

2. Build the project:

   ```bash
   cargo build --release
   ```

3. Run the server:
   ```bash
   cargo run --release
   ```

## Usage

The server runs on port 3000 by default. Once started, clients can connect to it via Socket.IO.

### REST API

- `GET /` - Health check endpoint (returns "alive")

### Socket.IO Events

#### Client to Server Events:

| Event            | Description                       | Parameters                                                                | Response                                           |
| ---------------- | --------------------------------- | ------------------------------------------------------------------------- | -------------------------------------------------- |
| `list-boards`    | List all available Arduino boards | None                                                                      | CommandResponse with JSON data of all boards       |
| `list-connected` | List all connected Arduino boards | None                                                                      | CommandResponse with JSON data of connected boards |
| `list-cores`     | List installed Arduino cores      | None                                                                      | CommandResponse with JSON data of cores            |
| `install-core`   | Install an Arduino core           | `{core: "core_name"}`                                                     | CommandResponse with installation result           |
| `compile-sketch` | Compile an Arduino sketch         | `{sketch_path: "/path/to/sketch", fqbn: "board_name"}`                    | CommandResponse with compilation result            |
| `upload-sketch`  | Upload a sketch to a board        | `{sketch_path: "/path/to/sketch", port: "/dev/port", fqbn: "board_name"}` | CommandResponse with upload result                 |

#### Server to Client Events:

| Event          | Description                 | Data                        |
| -------------- | --------------------------- | --------------------------- |
| `auth`         | Authentication response     | Echo of client auth data    |
| `message-back` | Response to `message` event | Echo of client message data |

### Response Format

All commands return a `CommandResponse` object with the following structure:

```json
{
  "success": boolean,
  "output": "command output string",
  "error": "error message if any (or null)",
  "command": "executed command",
  "args": ["array", "of", "arguments"]
}
```

## Desktop Daemon Usage

The Arduino ESP32 Cloud Compiler can run as a background daemon on your development machine, providing local IDE integrations and tools with Arduino compilation capabilities.

### Setting Up as a Background Service

#### macOS

1. Create a launch agent plist file:

```bash
mkdir -p ~/Library/LaunchAgents
touch ~/Library/LaunchAgents/com.arduino.esp32compiler.plist
```

2. Add the following content to the plist file:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.arduino.esp32compiler</string>
    <key>ProgramArguments</key>
    <array>
        <string>/path/to/arduino-esp32-cloud-compiler</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/arduino-esp32-compiler.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/arduino-esp32-compiler.log</string>
</dict>
</plist>
```

3. Load the service:

```bash
launchctl load ~/Library/LaunchAgents/com.arduino.esp32compiler.plist
```

#### Linux (systemd)

1. Create a systemd service file:

```bash
sudo nano /etc/systemd/system/arduino-esp32-compiler.service
```

2. Add the following content:

```
[Unit]
Description=Arduino ESP32 Cloud Compiler
After=network.target

[Service]
Type=simple
User=your_username
ExecStart=/path/to/arduino-esp32-cloud-compiler
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

3. Enable and start the service:

```bash
sudo systemctl enable arduino-esp32-compiler
sudo systemctl start arduino-esp32-compiler
```

#### Windows

1. Install NSSM (Non-Sucking Service Manager):
   Download from http://nssm.cc/

2. Install the service:

```cmd
nssm install ArduinoESP32Compiler C:\path\to\arduino-esp32-cloud-compiler.exe
nssm start ArduinoESP32Compiler
```

### Connecting to the Local Daemon

When running as a local daemon, you can connect to the service using:

```javascript
const socket = io("http://localhost:3000");
```

This enables your development tools and IDEs to leverage the Arduino CLI functionality without having to manage the Arduino CLI installation or updates.

## Arduino CLI Commands

The service wraps the following Arduino CLI commands:

- `arduino-cli board listall --format json` - List all available boards
- `arduino-cli board list --format json` - List connected boards
- `arduino-cli core list --format json` - List installed cores
- `arduino-cli core install <core>` - Install a specific core
- `arduino-cli compile --fqbn <fqbn> <sketch>` - Compile a sketch
- `arduino-cli upload --port <port> --fqbn <fqbn> <sketch>` - Upload a sketch

## Example Client Usage

Here's an example of how to connect to the service using Socket.IO in JavaScript:

```javascript
import { io } from "socket.io-client";

const socket = io("http://localhost:3000");

socket.on("connect", () => {
  console.log("Connected to server");

  // List all boards
  socket.emit("list-boards", (response) => {
    console.log("Available boards:", response);
  });

  // Compile a sketch
  socket.emit(
    "compile-sketch",
    {
      sketch_path: "/path/to/sketch/sketch.ino",
      fqbn: "esp32:esp32:esp32",
    },
    (response) => {
      console.log("Compilation result:", response);
    }
  );
});
```

## Development

### Project Structure

- `src/main.rs` - Main server entry point
- `src/compiler.rs` - Arduino CLI interface implementation
- `src/models.rs` - Data structures and models
- `src/socketio.rs` - Socket.IO event handlers
- `resource/` - Platform-specific Arduino CLI binaries
