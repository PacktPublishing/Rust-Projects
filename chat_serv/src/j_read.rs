use failure::Error;
use futures::try_ready;
use tokio::prelude::*;

enum Rding {
    Len(usize),
    Val(usize, Vec<u8>),
}

pub struct JRead<R: AsyncRead> {
    buf: [u8; 500],
    buflen: usize,
    ptr: usize,
    reading: Rding,
    r: R,
}

impl<R: AsyncRead> JRead<R> {
    pub fn new(r: R) -> Self {
        JRead {
            buf: [0; 500],
            buflen: 0,
            ptr: 0,
            reading: Rding::Len(0),
            r,
        }
    }
}

impl<R: AsyncRead> Stream for JRead<R> {
    type Item = String;
    type Error = Error;
    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        loop {
            if self.ptr == self.buflen {
                self.buflen = try_ready! { self.r.poll_read(&mut self.buf)};
                self.ptr = 0;
            }
            if self.buflen == 0 {
                return Ok(Async::Ready(None));
            }
            match self.reading {
                Rding::Len(ref mut nb) => {
                    match self.buf[self.ptr] {
                        b':' => self.reading = Rding::Val(*nb, Vec::new()),
                        v if v >= b'0' && v <= b'9' => {
                            *nb = *nb * 10 + ((v - 48) as usize);
                        }
                        _ => {}
                    }
                    self.ptr += 1;
                }
                Rding::Val(n, ref mut v) => {
                    let p_dist = std::cmp::min(self.ptr + n - v.len(), self.buflen);
                    v.append(&mut self.buf[self.ptr..p_dist].to_vec());
                    self.ptr = p_dist;
                    if v.len() == n {
                        let res = String::from_utf8(v.clone())?;
                        self.reading = Rding::Len(0);
                        return Ok(Async::Ready(Some(res)));
                    }
                }
            }
        }
    }
}
