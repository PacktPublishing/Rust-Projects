use futures::future::Future;
use futures::try_ready;
use serde::Serialize;
use tokio::prelude::*;

pub struct JWrite<S, W> {
    in_s: S,
    out_w: W,
    buff: Option<Vec<u8>>,
}

impl<S, W> JWrite<S, W> {
    pub fn new(in_s: S, out_w: W) -> Self {
        JWrite {
            in_s,
            out_w,
            buff: None,
        }
    }
}

impl<S, I, W> Future for JWrite<S, W>
where
    S: Stream<Item = I, Error = ()>,
    I: Serialize,
    W: AsyncWrite,
{
    type Item = ();
    type Error = ();
    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        loop {
            match self.buff {
                None => {
                    let i = match try_ready! {self.in_s.poll()} {
                        Some(v) => v,
                        None => return Ok(Async::Ready(())),
                    };
                    self.buff = serde_json::to_string(&i)
                        .map(|v| v.as_bytes().to_vec())
                        .ok();
                }
                Some(ref mut v) => {
                    let n = try_ready! {self.out_w.poll_write(v).map_err(|_| println!("could not write out"))};
                    if n == v.len() {
                        self.buff = None;
                    } else {
                        self.buff = Some(v.split_off(n));
                    }
                }
            }
        }
    }
}
