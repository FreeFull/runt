use std::thread;

use futures::prelude::*;
use futures::sync::mpsc::{unbounded, UnboundedSender};
use futures::sync::oneshot;
use url::Url;

use super::{Data, Error, Fetcher};

pub type FetcherRequest = (Url, oneshot::Sender<(Url, Result<Data, Error>)>);

pub fn spawn() -> UnboundedSender<FetcherRequest> {
    let (request_tx, request_rx) = unbounded();
    thread::spawn(move || {
        tokio::run(
            futures::lazy(move || {
                let fetcher = Fetcher::new().unwrap();
                request_rx.for_each(move |(url, result): FetcherRequest| {
                    fetcher
                        .clone()
                        .get_with_redirect(url.clone(), 30)
                        .then(move |response| {
                            let _ = result.send((url, response));
                            Ok(())
                        })
                })
            })
            .then(|_| Ok(())),
        )
    });
    request_tx
}

pub fn make_request(url: Url, request_tx: &UnboundedSender<FetcherRequest>) -> Receiver {
    let (result_tx, result_rx) = oneshot::channel();
    // If the Fetcher thread dies, that's a fatal error.
    request_tx
        .unbounded_send((url, result_tx))
        .expect("Failed send to Fetcher thread");
    Receiver(result_rx)
}

pub struct Receiver(oneshot::Receiver<(Url, Result<Data, Error>)>);

impl Receiver {
    pub fn wait(self) -> (Url, Result<Data, Error>) {
        self.0.wait().expect("Failed receive from Fetcher thread")
    }
}
