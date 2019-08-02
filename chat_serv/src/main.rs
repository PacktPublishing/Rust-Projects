use tokio::net::TcpListener;
use tokio::prelude::*;
use serde_derive::*;
use chatbox::{ChatBox,Request};
use tokio_channel::{mpsc,oneshot};

mod j_read;
mod j_write;


#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Message{
    name:String,
    tx:String,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ServerMessage{
    mess:Option<Message>,
    since:Option<usize>,

}

fn main() {
    let addr = "127.0.0.1:8088".parse().unwrap();
    let lis = TcpListener::bind(&addr).expect("could not bind address");

    let (ch_box,ch_s) = ChatBox::new();


    let fut = lis.incoming().for_each(move |sock|{
        let ch_s = ch_s.clone();
        let (sock_r,sock_w) = sock.split();
        let (fin_s,fin_r) = mpsc::channel(10);
        let write_f = j_write::JWrite::new(fin_r,sock_w);
        tokio::spawn(write_f);
        let rd = j_read::JRead::new(sock_r).for_each(move |s| {
            let v:ServerMessage = serde_json::from_str(&s)?;
            println!("recieved:{:?}",v);
            if let Some(m) = v.mess{
                let f = ch_s.clone()
                        .send(Request::Put(m))
                        .map(|_|())
                        .map_err(|_|println!("could not send message to chatbox"));
                tokio::spawn(f);
            }

            if let Some(n) = v.since{
                let (os_s,os_r) = oneshot::channel();
                let fc = fin_s.clone();
                let f = ch_s.clone()
                        .send(Request::Since(n,os_s))
                        .map_err(|_|println!("could not send since to chatbox"))
                        .and_then(|_|os_r.map_err(|_|println!("could not get from chatbox oneshot")))
                        .and_then(move |v| fc.send(v).map_err(|_|println!("could not send to fin_c")))
                        .map(|_|());
                tokio::spawn(f);
            }
            Ok(())
        }).map_err(|_|());
        tokio::spawn(rd);
        Ok(())
    }).map_err(|e|println!("Listening Err :{:?}" ,e));

    tokio::run(fut.join(ch_box).map(|_|()));

    println!("Hello, world!");
}
