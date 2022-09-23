use axum::extract::ws::{Message, WebSocket};
use log::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::config::get_dst_addr;

pub async fn handle_client(mut client: WebSocket) {
    let dst = get_dst_addr();
    let server_stream = TcpStream::connect(dst).await;
    info!("Connect to remote {:#?}", server_stream);

    if server_stream.is_err() {
        error!("Connect to remote failed {:#?}", server_stream);
        let _ = client
            .send(Message::Text(format!("{:#?}", server_stream)))
            .await;
        return;
    }

    let mut server_stream = server_stream.unwrap();

    let mut buf = [0u8; 17000]; // the max ssl record should be 16384 by default

    loop {
        tokio::select! {
            res  = client.recv() => {
                if let Some(msg) = res {
                    if let Ok(Message::Binary(msg)) = msg {
                        let _ = server_stream.write_all(&msg).await;
                    }
                } else {
                    info!("Client close");
                    return;
                }
            },
            res  = server_stream.read(&mut buf) => {
                let n = res.unwrap();
                info!("Recv {}", n);
                if 0 != n {
                    debug!("Recv {}", n);
                    let _ = client.send(Message::Binary(buf[..n].to_vec())).await;
                } else {
                    return ;
                }
            },
        }
    }

    // if let Some(msg) = client.recv().await {
    //     if let Ok(msg) = msg {
    //         match msg {
    //             Message::Text(t) => {
    //                 println!("client sent str: {:?}", t);
    //             }
    //             Message::Binary(_) => {
    //                 println!("client sent binary data");
    //             }
    //             Message::Ping(_) => {
    //                 println!("socket ping");
    //             }
    //             Message::Pong(_) => {
    //                 println!("socket pong");
    //             }
    //             Message::Close(_) => {
    //                 println!("client disconnected");
    //                 return;
    //             }
    //         }
    //     } else {
    //         println!("client disconnected");
    //         return;
    //     }
    // }

    // loop {
    //     if client
    //         .send(Message::Text(String::from("Hi!")))
    //         .await
    //         .is_err()
    //     {
    //         println!("client disconnected");
    //         return;
    //     }
    //     tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    // }
}