use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

pub type Payload<Req, Res> = (Req, Responder<Res>);

#[derive(Debug)]
pub struct ResponseReceiver<Res> {
    pub(crate) response_receiver: Option<oneshot::Receiver<Res>>,
}

impl<Res> ResponseReceiver<Res> {
    pub(crate) fn new(response_receiver: oneshot::Receiver<Res>) -> Self {
        println!("ResponseReceiver");
        Self {
            response_receiver: Some(response_receiver),
        }
    }

    pub async fn recv(&mut self) -> Result<Res, ReceiveError> {
        println!("recv");
        match self.response_receiver.take() {
            Some(response_receiver) => {
                response_receiver.await.map_err(|_| {
                    println!("RecvError 1");

                    ReceiveError::RecvError
                })
            },
            None => {
                println!("RecvError 2");
                Err(ReceiveError::RecvError)
            },
        }
    }
}


#[derive(Debug)]
pub struct RequestSender<Req, Res> {
    request_sender: mpsc::UnboundedSender<Payload<Req, Res>>,
}

impl<Req, Res> RequestSender<Req, Res> {
    fn new(
        request_sender: mpsc::UnboundedSender<Payload<Req, Res>>,
    ) -> Self {
        RequestSender {
            request_sender,
        }
    }

    pub fn send(&self, request: Req) -> Result<ResponseReceiver<Res>, RequestError<Req>> {
        let (response_sender, response_receiver) = oneshot::channel::<Res>();
        let responder = Responder::new(response_sender);
        let payload = (request, responder);
        self.request_sender
            .send(payload)
            .map_err(|payload| RequestError::SendError(payload.0.0))?;
        let receiver = ResponseReceiver::new(response_receiver);
        Ok(receiver)
    }

    pub async fn send_receive(&self, request: Req) -> Result<Res, RequestError<Req>> {
        let mut receiver = self.send(request)?;
        receiver.recv().await.map_err(|err| err.into())
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

    pub async fn recv(&mut self) -> Result<Payload<Req, Res>, RequestError<Req>> {
        match self.request_receiver.recv().await {
            Some(payload) => Ok(payload),
            None => {
                println!("RecvError 3");

                Err(RequestError::RecvError)
            },
        }
    }
}

impl<Res> Responder<Res> {
    fn new(response_sender: oneshot::Sender<Res>) -> Self {
        Self { response_sender }
    }

    pub fn respond(self, response: Res) -> Result<(), RespondError<Res>> {
        self.response_sender.send(response).map_err(RespondError)
    }
}

#[derive(Debug)]
pub struct Responder<Res> {
    response_sender: oneshot::Sender<Res>,
}

#[derive(Error, Debug, Copy, Clone, PartialEq)]
pub enum RequestError<T> {
    #[error("Recv error")]
    RecvError,
    #[error("send error")]
    SendError(T),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RespondError<T>(pub T);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ReceiveError {
    RecvError,
}

impl<T> From<ReceiveError> for RequestError<T> {
    fn from(err: ReceiveError) -> RequestError<T> {
        match err {
            ReceiveError::RecvError => RequestError::RecvError,
        }
    }
}

pub fn channel<Req, Res>() -> (RequestSender<Req, Res>, RequestReceiver<Req, Res>) {
    let (sender, receiver) = mpsc::unbounded_channel::<Payload<Req, Res>>();
    let request_sender = RequestSender::new(sender);
    let request_receiver = RequestReceiver::new(receiver);
    (request_sender, request_receiver)
}
