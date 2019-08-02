use futures::Future;
use std::fmt::Debug;
use tokio::prelude::*;
use tokio_channel::{mpsc, oneshot};

pub enum Request<M> {
    Put(M),
    Since(usize, oneshot::Sender<Vec<M>>),
}

pub struct ChatBox<M> {
    store: Vec<M>,
    ch_r: mpsc::Receiver<Request<M>>,
}

impl<M> ChatBox<M> {
    pub fn new() -> (Self, mpsc::Sender<Request<M>>) {
        let (ch_s, ch_r) = mpsc::channel(10);
        (
            ChatBox {
                store: Vec::new(),
                ch_r,
            },
            ch_s,
        )
    }
}

impl<M: Debug + Clone> Future for ChatBox<M> {
    type Item = ();
    type Error = ();
    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        loop {
            let rq = match { self.ch_r.poll()? } {
                Async::NotReady => return Ok(Async::NotReady),
                Async::Ready(Some(v)) => v,
                Async::Ready(None) => return Ok(Async::Ready(())),
            };
            match rq {
                Request::Put(m) => {
                    println!("got message {:?}", m);
                    self.store.push(m);
                }
                Request::Since(n, ch) => {
                    println!("got request {:?}", n);
                    let res = if n >= self.store.len() {
                        Vec::new()
                    } else {
                        Vec::from(&self.store[n..])
                    };
                    ch.send(res).ok();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::lazy;

    #[test]
    fn it_works() {
        let (ts_s, ts_r) = mpsc::channel(10);
        let f = lazy(move || {
            let (f, ch_s) = ChatBox::new();
            tokio::spawn(f);
            for i in 0..5 {
                let tss = ts_s.clone();
                let ch2 = ch_s.clone();
                let (os_s, os_r) = oneshot::channel();
                let f2 = ch_s
                    .clone()
                    .send(Request::Put(i))
                    .and_then(|_| ch2.send(Request::Since(0, os_s)))
                    .map_err(|e| println!("{:?}", e))
                    .and_then(|_| os_r.map_err(|_| ()))
                    .and_then(move |res| {
                        println!("res {} = {:?}", i, res);
                        tss.send(res)
                            .map_err(move |_| println!("could not send {}", i))
                    })
                    .map(|_| ());
                tokio::spawn(f2);
            }
            Ok(())
        });
        tokio::run(f);

        let mut longest = 0;
        for v in ts_r.wait() {
            longest = std::cmp::max(longest, v.unwrap().len());
        }
        assert_eq!(longest, 5);
    }
}
