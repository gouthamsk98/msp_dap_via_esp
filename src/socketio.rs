use socketioxide::{ extract::{ AckSender, Data, SocketRef }, SocketIo };
use serde_json::Value;
use tracing::info;

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
    tokio::spawn(async move {
        loop {
            // This is where you would handle the main loop of your debugger
            // For example, you might want to listen for commands or events
            // and process them accordingly.
            // This is a placeholder for your main loop logic.
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
