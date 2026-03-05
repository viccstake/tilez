use std::marker::PhantomData;

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use serde::{Serialize, de::DeserializeOwned};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::error::{Error, Result};

pub struct Session<In, Out> {
    framed: Framed<TcpStream, LengthDelimitedCodec>,
    _marker: PhantomData<(In, Out)>,
}

impl<In, Out> Session<In, Out>
where
    In: DeserializeOwned,
    Out: Serialize,
{
    pub fn new(stream: TcpStream) -> Self {
        let codec = LengthDelimitedCodec::new();
        let framed = Framed::new(stream, codec);

        Self {
            framed,
            _marker: PhantomData,
        }
    }

    /// Receive a typed message
    pub async fn recv(&mut self) -> Result<Option<In>> {
        match self.framed.next().await {
            Some(Ok(bytes)) => {
                let msg = bincode::deserialize::<In>(&bytes).map_err(Error::Deserialize)?;
                Ok(Some(msg))
            }
            Some(Err(e)) => Err(Error::Io(e)),
            None => Ok(None), // connection closed
        }
    }

    /// Send a typed message
    pub async fn send(&mut self, msg: &Out) -> Result<()> {
        let bytes = bincode::serialize(msg).map_err(Error::Serialize)?;

        self.framed
            .send(Bytes::from(bytes))
            .await
            .map_err(Error::Io)
    }
}
