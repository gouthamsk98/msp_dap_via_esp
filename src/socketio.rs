use socketioxide::{ extract::{ AckSender, Data, SocketRef }, SocketIo };
use serde_json::Value;
use tracing::info;

use crate::loader;

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
    register_debugger_handlers(&socket);
    check_port_connection(socket.clone());
}
fn check_port_connection(socket: SocketRef) {
    tokio::spawn(async move {
        let mut last_status = None;
        loop {
            // Check if the port is connected
            let connected = loader::is_device_connected(loader::TARGET_PID);

            if last_status != Some(connected) {
                socket.emit("device-connected", &Value::Bool(connected)).ok();
                last_status = Some(connected);
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
    });
}
fn register_debugger_handlers(socket: &SocketRef) {
    socket.on(
        "connect",
        |Data::<Value>(data), ack: AckSender| {
            //get the port path based on pid

            // info!(?data, "Connect event received");
            // ack.send(&data).ok();
        }
    );

    socket.on("halt", |ack: AckSender| {
        info!("Halt command received");
        // Here you would call the halt function from your loader
    });

    socket.on("resume", |ack: AckSender| {
        info!("Resume command received");
        // Here you would call the resume function from your loader
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
