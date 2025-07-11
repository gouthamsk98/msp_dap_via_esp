use socketioxide::{ extract::{ AckSender, Data, SocketRef }, SocketIo };
use serde_json::Value;
use tracing::info;
use std::sync::{ Arc, Mutex };

use crate::{ loader, models::CommandResponse };

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
    let loader = Arc::new(
        Mutex::new(loader::SerialLoader::new(None, 115200).expect("Failed to create SerialLoader"))
    );
    register_debugger_handlers(&socket, Arc::clone(&loader));
    check_port_connection(socket.clone(), Arc::clone(&loader));
}
fn check_port_connection(socket: SocketRef, loader: Arc<Mutex<loader::SerialLoader>>) {
    tokio::spawn(async move {
        let mut last_status = None;
        loop {
            // Check if the port is connected
            let connected = loader::is_device_connected(loader::TARGET_PID);

            if last_status != Some(connected) {
                if connected {
                    match loader.lock() {
                        Ok(mut loader_guard) => {
                            match loader_guard.reconnect() {
                                Ok(_) => {
                                    info!("Device connected successfully");
                                    socket.emit("device-connected", &Value::Bool(true)).ok();
                                }
                                Err(e) => {
                                    info!("Failed to reconnect device: {}", e);
                                    socket.emit("device-connected", &Value::Bool(false)).ok();
                                }
                            }
                        }
                        Err(e) => {
                            info!("Failed to acquire loader lock for reconnect: {}", e);
                            socket.emit("device-connected", &Value::Bool(false)).ok();
                        }
                    }
                } else {
                    match loader.lock() {
                        Ok(mut loader_guard) => {
                            let _ = loader_guard.close();
                            info!("Device disconnected");
                            socket.emit("device-connected", &Value::Bool(false)).ok();
                        }
                        Err(e) => {
                            info!("Failed to acquire loader lock for close: {}", e);
                        }
                    }
                }

                last_status = Some(connected);
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
    });
}
fn register_debugger_handlers(socket: &SocketRef, loader: Arc<Mutex<loader::SerialLoader>>) {
    socket.on(
        "connect",
        |Data::<Value>(data), ack: AckSender| {
            //get the port path based on pid

            // info!(?data, "Connect event received");
            // ack.send(&data).ok();
        }
    );

    let loader_clone = Arc::clone(&loader);
    socket.on("halt", move |ack: AckSender| {
        let loader_clone = Arc::clone(&loader_clone);
        tokio::spawn(async move {
            info!("Halt command received");
            // Here you would call the halt function from your loader
            match loader_clone.lock() {
                Ok(mut loader) => {
                    match loader.halt() {
                        Ok(_) => {
                            let response = CommandResponse {
                                success: true,
                                message: "Halted".to_string(),
                                command: "halt".to_string(),
                                args: vec![],
                            };
                            ack.send(&response).ok();
                        }
                        Err(e) => {
                            let response = CommandResponse {
                                success: false,
                                message: format!("Error: {}", e),
                                command: "halt".to_string(),
                                args: vec![],
                            };
                            info!("Failed to halt the loader: {}", e);
                            ack.send(&response).ok();
                        }
                    }
                }
                Err(e) => {
                    info!("Failed to acquire loader lock: {}", e);
                    let response = CommandResponse {
                        success: false,
                        message: "Error: Failed to acquire loader lock".to_string(),
                        command: "halt".to_string(),
                        args: vec![],
                    };
                    ack.send(&response).ok();
                }
            }
        });
    });

    socket.on("resume", |ack: AckSender| {
        info!("Resume command received");
        tokio::spawn(async move {
            // Here you would call the resume function from your loader
            match loader.lock() {
                Ok(mut loader) => {
                    match loader.resume() {
                        Ok(_) => {
                            let response = CommandResponse {
                                success: true,
                                message: "Resumed".to_string(),
                                command: "resume".to_string(),
                                args: vec![],
                            };
                            ack.send(&response).ok();
                        }
                        Err(e) => {
                            let response = CommandResponse {
                                success: false,
                                message: format!("Error: {}", e),
                                command: "resume".to_string(),
                                args: vec![],
                            };
                            info!("Failed to resume the loader: {}", e);
                            ack.send(&response).ok();
                        }
                    }
                }
                Err(e) => {
                    info!("Failed to acquire loader lock for resume: {}", e);
                    let response = CommandResponse {
                        success: false,
                        message: "Error: Failed to acquire loader lock".to_string(),
                        command: "resume".to_string(),
                        args: vec![],
                    };
                    ack.send(&response).ok();
                }
            }
        });
    });

    socket.on("write-word", |Data::<Value>(data), ack: AckSender| {
        if let (Some(address), Some(value)) = (data.get("address"), data.get("value")) {
            if let (Some(address_u32), Some(value_u32)) = (address.as_u64(), value.as_u64()) {
                info!(?address_u32, ?value_u32, "Write word command received");
                // Here you would call the write_word function from your loader
            }
        }
    });
}
