use crate::sources::interfaces::{IpFuture, IpResult, Source};
use igd;
use std::pin::Pin;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};
use std::thread;

use log::trace;

use std::net::IpAddr;

/// IGD Source of the external ip
///
/// It will try to connect to the local router implementing the IGD interface to obtain the external
/// IP directly from it.
///
/// The feature "igd" must be enabled to use this t(on by default)
#[derive(Debug, Clone)]
pub struct IGD {}

impl IGD {
    pub fn source() -> Box<dyn Source> {
        Box::new(IGD {})
    }
}

impl Source for IGD {
    fn get_ip<'a>(&'a self) -> IpFuture<'a> {
        let (tx, rx) = mpsc::channel();
        let future = IGDFuture {
            rx: rx,
            waker: Arc::new(Mutex::from(None)),
        };
        future.run(tx);
        Box::pin(future)
    }

    fn box_clone(&self) -> Box<dyn Source> {
        Box::new(self.clone())
    }
}

type ThreadOutput = Mutex<IpResult>;

struct IGDFuture {
    rx: mpsc::Receiver<ThreadOutput>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl std::fmt::Display for IGD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IGD")
    }
}

impl IGDFuture {
    pub fn run(&self, tx: mpsc::Sender<ThreadOutput>) {
        let waker = self.waker.clone();
        thread::spawn(move || {
            trace!("IGD Future thread started");
            fn inner() -> IpResult {
                let gateway = igd::search_gateway(Default::default())?;
                let ip = gateway.get_external_ip()?;
                return Ok(IpAddr::from(ip));
            }

            let result = inner();
            log::debug!("IGD task completed: {:?}", result);
            let r = tx.send(ThreadOutput::from(result));
            log::debug!("Send result: {:?}", r);

            if let Some(waker) = waker.lock().unwrap().take() {
                waker.wake();
            }
        });
    }
}

impl std::future::Future for IGDFuture {
    type Output = IpResult;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        let r = self.rx.try_recv();
        match r {
            Err(_) => {
                let mut waker = self.waker.lock().unwrap();
                *waker = Some(cx.waker().clone());
                Poll::Pending
            }
            Ok(x) => Poll::Ready(x.into_inner().unwrap()),
        }
    }
}
