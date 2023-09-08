use std::future::Future;
use std::net::{IpAddr, Ipv4Addr};

use port_check;
use rocket::futures::TryFutureExt;
use rocket::response::stream::{Event, EventStream};
use rocket::{
    fs::FileServer, futures::channel::mpsc::Receiver, get, routes, Error, Ignite, Rocket, Shutdown,
    State,
};
use serde::{Deserialize, Serialize};
use tokio::{
    select,
    sync::{
        broadcast::{error::RecvError, Sender},
        mpsc,
    },
};

use crate::{
    message::{Message, MessageData, MessageType},
    CONFIG,
};

pub async fn run(input_sender: Sender<Message>, port: u16) {
    let config = rocket::Config {
        port,
        address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        ..rocket::Config::release_default()
    };

    #[get("/")]
    fn index() -> &'static str {
        "Hello, world!"
    }

    #[get("/events")]
    async fn events(sender: &State<Sender<Message>>, mut end: Shutdown) -> EventStream![] {
        let config = unsafe { CONFIG.lock().unwrap().clone() };
        let mut rx = sender.subscribe();
        EventStream! {
            // 连接上后首先发送config
            yield Event::json(&Message {
                r#type: MessageType::Config,
                data: MessageData::ConfigMessage(config),
            });
            // 然后循环接收发送msg
            loop {
                let msg = select! {
                    msg = rx.recv() => match msg {
                        Ok(msg) => msg,
                        Err(RecvError::Closed) => break,
                        Err(RecvError::Lagged(_)) => continue,
                    },
                    _ = &mut end => {
                        rx.resubscribe();
                        break;
                    },
                };
                yield Event::json(&msg);
            }
        }
    }

    let _ = rocket::custom(&config)
        .manage(input_sender)
        .mount("/", routes![index, events])
        .launch()
        .await;
}
