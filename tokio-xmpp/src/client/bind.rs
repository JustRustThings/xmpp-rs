use futures::{sink, Async, Future, Poll, Stream};
use std::convert::TryFrom;
use std::mem::replace;
use tokio_io::{AsyncRead, AsyncWrite};
use xmpp_parsers::Jid;
use xmpp_parsers::bind::{BindQuery, BindResponse};
use xmpp_parsers::iq::{Iq, IqType};

use crate::xmpp_codec::Packet;
use crate::xmpp_stream::XMPPStream;
use crate::{Error, ProtocolError};

const NS_XMPP_BIND: &str = "urn:ietf:params:xml:ns:xmpp-bind";
const BIND_REQ_ID: &str = "resource-bind";

pub enum ClientBind<S: AsyncWrite> {
    Unsupported(XMPPStream<S>),
    WaitSend(sink::Send<XMPPStream<S>>),
    WaitRecv(XMPPStream<S>),
    Invalid,
}

impl<S: AsyncWrite> ClientBind<S> {
    /// Consumes and returns the stream to express that you cannot use
    /// the stream for anything else until the resource binding
    /// req/resp are done.
    pub fn new(stream: XMPPStream<S>) -> Self {
        match stream.stream_features.get_child("bind", NS_XMPP_BIND) {
            None =>
            // No resource binding available,
            // return the (probably // usable) stream immediately
            {
                ClientBind::Unsupported(stream)
            }
            Some(_) => {
                let resource;
                if let Jid::Full(jid) = stream.jid.clone() {
                    resource = Some(jid.resource);
                } else {
                    resource = None;
                }
                let iq = Iq::from_set(BIND_REQ_ID, BindQuery::new(resource));
                let send = stream.send_stanza(iq);
                ClientBind::WaitSend(send)
            }
        }
    }
}

impl<S: AsyncRead + AsyncWrite> Future for ClientBind<S> {
    type Item = XMPPStream<S>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let state = replace(self, ClientBind::Invalid);

        match state {
            ClientBind::Unsupported(stream) => Ok(Async::Ready(stream)),
            ClientBind::WaitSend(mut send) => match send.poll() {
                Ok(Async::Ready(stream)) => {
                    replace(self, ClientBind::WaitRecv(stream));
                    self.poll()
                }
                Ok(Async::NotReady) => {
                    replace(self, ClientBind::WaitSend(send));
                    Ok(Async::NotReady)
                }
                Err(e) => Err(e)?,
            },
            ClientBind::WaitRecv(mut stream) => match stream.poll() {
                Ok(Async::Ready(Some(Packet::Stanza(stanza)))) => match Iq::try_from(stanza) {
                    Ok(iq) => {
                        if iq.id == BIND_REQ_ID {
                            match iq.payload {
                                IqType::Result(payload) => {
                                    payload
                                        .and_then(|payload| BindResponse::try_from(payload).ok())
                                        .map(|bind| stream.jid = bind.into());
                                    Ok(Async::Ready(stream))
                                }
                                _ => Err(ProtocolError::InvalidBindResponse)?,
                            }
                        } else {
                            Ok(Async::NotReady)
                        }
                    }
                    _ => Ok(Async::NotReady),
                },
                Ok(Async::Ready(_)) => {
                    replace(self, ClientBind::WaitRecv(stream));
                    self.poll()
                }
                Ok(Async::NotReady) => {
                    replace(self, ClientBind::WaitRecv(stream));
                    Ok(Async::NotReady)
                }
                Err(e) => Err(e)?,
            },
            ClientBind::Invalid => unreachable!(),
        }
    }
}
