use super::Active;
use super::Capture;
use std::io;
use tokio::io::unix::AsyncFd;
pub struct AsyncPcap {
    pub inner: AsyncFd<Capture<Active>>, // pcap::Capture device
}

impl AsyncPcap {
    pub fn new(device_name: String) -> io::Result<Self> {
        let cap = Capture::from_device(device_name.as_str())
            .expect("Could not open device")
            .promisc(true)
            .immediate_mode(true)
            .open()
            .expect("Failed to open device correctly")
            .setnonblock()
            .expect("Failed to configure interface as non-block");
        Ok(AsyncPcap {
            inner: AsyncFd::new(cap)?,
        })
    }
    pub async fn read(&mut self, out: &mut Vec<u8>) -> io::Result<usize> {
        loop {
            let mut guard = self.inner.readable_mut().await?;
            match guard.try_io(|inner| match inner.get_mut().next() {
                Ok(pkt) => {
                    *out = pkt.data.to_vec(); // Copy pkt data and return lenght of packet
                                              //println!("{:?}", out);
                    Ok(pkt.header.len as usize)
                }
                Err(_) => Err(std::io::Error::from(std::io::ErrorKind::WouldBlock)),
            }) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            };
        }
    }

    pub async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.inner.writable_mut().await?;
            match guard.try_io(|inner| match inner.get_mut().sendpacket(buf) {
                Ok(_) => Ok(buf.len()),
                Err(_) => Err(std::io::Error::from(std::io::ErrorKind::Other)),
            }) {
                Ok(result) => return result,
                Err(e) => {
                    continue;
                }
            }
        }
    }
}
// TODO: Implement these at some point for tx, rx split compatibility
//use futures::ready;
//impl AsyncRead for AsyncPcap {
//    fn poll_read(
//        self: std::pin::Pin<&mut Self>,
//        cx: &mut std::task::Context<'_>,
//        buf: &mut tokio::io::ReadBuf<'_>,
//    ) -> std::task::Poll<io::Result<()>> {
//        let b = vec![0; buf.capacity()];
//        let self_mut = self.get_mut();
//        loop {
//            let mut guard = ready!(self_mut.inner.poll_read_ready_mut(cx))?; // Check for ready
//            match guard.try_io(|inner| match inner.get_mut().next() {
//                Ok(pkt) => {
//                    buf.put_slice(pkt.data); // Copy pkt data and return lenght of packet
//                    Ok(pkt.header.len)
//                }
//                Err(_) => Err(std::io::Error::from(std::io::ErrorKind::WouldBlock)),
//            }) {
//                Ok(result) => Poll::Ready(result),
//                Err(_would_block) => continue,
//            };
//        }
//    }
//}
//
//impl AsyncWrite for AsyncPcap {
//    fn poll_write(
//        self: std::pin::Pin<&mut Self>,
//        cx: &mut std::task::Context<'_>,
//        buf: &[u8],
//    ) -> std::task::Poll<Result<usize, io::Error>> {
//        let self_mut = self.get_mut();
//        loop {
//            let mut guard = ready!(self_mut.inner.poll_write_ready_mut(cx))?;
//
//            match guard.try_io(|inner| match inner.get_mut().sendpacket(buf) {
//                Ok(_) => Ok(buf.len()),
//                Err(_) => Err(std::io::Error::from(ErrorKind::WouldBlock)),
//            }) {
//                Ok(n) => return Poll::Ready(n),
//                Err(_would_block) => continue,
//            }
//        }
//    }
//    // TODO: implement these properly
//    fn poll_flush(
//        self: std::pin::Pin<&mut Self>,
//        cx: &mut std::task::Context<'_>,
//    ) -> std::task::Poll<Result<(), io::Error>> {
//        unimplemented!("Not implemented");
//        //Poll::Ready(Ok(()))
//    }
//    fn poll_shutdown(
//        self: std::pin::Pin<&mut Self>,
//        cx: &mut std::task::Context<'_>,
//    ) -> std::task::Poll<Result<(), io::Error>> {
//        unimplemented!("Not implemented");
//        //Poll::Ready(Ok(()))
//    }
//}
