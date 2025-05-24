use std::time::Duration;

use anyhow::Error;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::time::error::Elapsed;
use tonic::Code;

#[derive(Error, Debug, Clone)]
pub enum RequestError {
    #[error("The other side has not managed to process request in a timely manner")]
    Timeout,
    #[error("The other side has dropped the oneshot prematurely")]
    OtherSideWasDropped,
    #[error("Error: {display:?}")]
    Other { display: String },
}

pub type RequestResult<V> = Result<V, RequestError>;

impl From<Elapsed> for RequestError {
    fn from(_: Elapsed) -> RequestError {
        RequestError::Timeout
    }
}

impl From<anyhow::Error> for RequestError {
    fn from(error: Error) -> RequestError {
        RequestError::Other {
            display: format!("{}", error),
        }
    }
}

impl From<tonic::Status> for RequestError {
    fn from(error: tonic::Status) -> RequestError {
        match error.code() {
            Code::Ok => unreachable!(),
            Code::DeadlineExceeded => RequestError::Timeout,
            _ => {
                RequestError::Other {
                    display: format!("{}", error.message()),
                }
            }
        }
    }
}

impl From<prost::UnknownEnumValue> for RequestError {
    fn from(error: prost::UnknownEnumValue) -> RequestError {
        RequestError::Other {
            display: format!("{}", error),
        }
    }
}

pub type Payload<Req, Res> = (Req, Responder<Res>);

#[derive(Debug)]
pub struct ResponseReceiver<Res> {
    pub(crate) response_receiver: Option<oneshot::Receiver<anyhow::Result<Res>>>,
}

impl<Res> ResponseReceiver<Res> {
    pub(crate) fn new(response_receiver: oneshot::Receiver<anyhow::Result<Res>>) -> Self {
        Self {
            response_receiver: Some(response_receiver),
        }
    }

    pub async fn recv(&mut self) -> anyhow::Result<Res> {
        self.response_receiver
            .take()
            .expect("recv was called second time")
            .await
            .expect("oneshot was dropped before sending")
    }
}

#[derive(Debug)]
pub struct RequestSender<Req, Res> {
    request_sender: mpsc::UnboundedSender<Payload<Req, Res>>,
}

impl<Req: std::fmt::Debug, Res: std::fmt::Debug> RequestSender<Req, Res> {
    fn new(request_sender: mpsc::UnboundedSender<Payload<Req, Res>>) -> Self {
        RequestSender { request_sender }
    }

    pub fn send(&self, request: Req) -> RequestResult<ResponseReceiver<Res>> {
        let (response_sender, response_receiver) = oneshot::channel::<anyhow::Result<Res>>();
        let responder = Responder::new(response_sender);
        let payload = (request, responder);
        self.request_sender
            .send(payload)
            .map_err(|_err| RequestError::OtherSideWasDropped)?;
        Ok(ResponseReceiver::new(response_receiver))
    }

    pub async fn send_receive(&self, request: Req) -> RequestResult<Res> {
        let mut receiver = self.send(request)?;

        let duration = Duration::from_secs(30);

        let result = tokio::time::timeout(duration, receiver.recv()).await?.map_err(|err| {
            RequestError::Other {
                display: format!("{}", err),
            }
        })?;

        Ok(result)
    }
}

impl<Req, Res> Clone for RequestSender<Req, Res> {
    fn clone(&self) -> Self {
        RequestSender {
            request_sender: self.request_sender.clone(),
        }
    }
}

#[derive(Debug)]
pub struct RequestReceiver<Req, Res> {
    request_receiver: mpsc::UnboundedReceiver<Payload<Req, Res>>,
}

impl<Req, Res> RequestReceiver<Req, Res> {
    fn new(receiver: mpsc::UnboundedReceiver<Payload<Req, Res>>) -> Self {
        RequestReceiver {
            request_receiver: receiver,
        }
    }

    pub async fn recv(&mut self) -> Payload<Req, Res> {
        self.request_receiver
            .recv()
            .await
            .expect("the other side of a channel was dropped")
    }
}

impl<Res: std::fmt::Debug> Responder<Res> {
    fn new(response_sender: oneshot::Sender<anyhow::Result<Res>>) -> Self {
        Self { response_sender }
    }

    pub fn respond(self, response: anyhow::Result<Res>) {
        self.response_sender.send(response).expect("the receiver was closed")
    }
}

#[derive(Debug)]
pub struct Responder<Res> {
    response_sender: oneshot::Sender<anyhow::Result<Res>>,
}

pub fn channel<Req: std::fmt::Debug, Res: std::fmt::Debug>() -> (RequestSender<Req, Res>, RequestReceiver<Req, Res>) {
    let (sender, receiver) = mpsc::unbounded_channel::<Payload<Req, Res>>();
    let request_sender = RequestSender::new(sender);
    let request_receiver = RequestReceiver::new(receiver);
    (request_sender, request_receiver)
}
