use std::io;
use std::marker::PhantomData;

use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use futures::{SinkExt, StreamExt};
use bytes::Bytes;
use serde::{Serialize, de::DeserializeOwned};

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
    pub async fn recv(&mut self) -> io::Result<Option<In>> {
        match self.framed.next().await {
            Some(Ok(bytes)) => {
                let msg = bincode::deserialize::<In>(&bytes)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Ok(Some(msg))
            }
            Some(Err(e)) => Err(e),
            None => Ok(None), // connection closed
        }
    }

    /// Send a typed message
    pub async fn send(&mut self, msg: &Out) -> io::Result<()> {
        let bytes = bincode::serialize(msg)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.framed.send(Bytes::from(bytes)).await
    }
}