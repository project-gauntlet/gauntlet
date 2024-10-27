use std::time::Duration;

use tokio::sync::{mpsc, oneshot};
use tokio::time::error::Elapsed;

#[derive(Debug)]
pub enum RequestError {
    TimeoutError,
}

impl From<Elapsed> for RequestError {
    fn from(_: Elapsed) -> RequestError {
        RequestError::TimeoutError
    }
}

pub type Payload<Req, Res> = (Req, Responder<Res>);

#[derive(Debug)]
pub struct ResponseReceiver<Res> {
    pub(crate) response_receiver: Option<oneshot::Receiver<Res>>,
}

impl<Res> ResponseReceiver<Res> {
    pub(crate) fn new(response_receiver: oneshot::Receiver<Res>) -> Self {
        Self {
            response_receiver: Some(response_receiver),
        }
    }

    pub async fn recv(&mut self) -> Res {
        self.response_receiver.take()
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
    fn new(
        request_sender: mpsc::UnboundedSender<Payload<Req, Res>>,
    ) -> Self {
        RequestSender {
            request_sender,
        }
    }

    pub fn send(&self, request: Req) -> ResponseReceiver<Res> {
        let (response_sender, response_receiver) = oneshot::channel::<Res>();
        let responder = Responder::new(response_sender);
        let payload = (request, responder);
        self.request_sender.send(payload).expect("the other side is closed");
        ResponseReceiver::new(response_receiver)
    }

    pub async fn send_receive(&self, request: Req) -> Result<Res, RequestError> {
        let mut receiver = self.send(request);

        let duration = Duration::from_secs(30);

        let result = tokio::time::timeout(duration, receiver.recv()).await?;

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
        self.request_receiver.recv()
            .await
            .expect("the other side of a channel was dropped")
    }
}

impl<Res: std::fmt::Debug> Responder<Res> {
    fn new(response_sender: oneshot::Sender<Res>) -> Self {
        Self { response_sender }
    }

    pub fn respond(self, response: Res) {
        self.response_sender.send(response).expect("the receiver was closed")
    }
}

#[derive(Debug)]
pub struct Responder<Res> {
    response_sender: oneshot::Sender<Res>,
}

pub fn channel<Req: std::fmt::Debug, Res: std::fmt::Debug>() -> (RequestSender<Req, Res>, RequestReceiver<Req, Res>) {
    let (sender, receiver) = mpsc::unbounded_channel::<Payload<Req, Res>>();
    let request_sender = RequestSender::new(sender);
    let request_receiver = RequestReceiver::new(receiver);
    (request_sender, request_receiver)
}
