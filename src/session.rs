use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Session<L: GameLogic> {
    reader: tokio::io::ReadHalf<TcpStream>,
    writer: tokio::io::WriteHalf<TcpStream>,
    logic: L,
}

pub trait GameLogic {
    type Message;

    fn on_message(&mut self, msg: Self::Message) -> Option<Self::Message>;
}

impl<L: GameLogic> Session<L>
where
    L::Message: From<Vec<u8>> + Into<Vec<u8>>,
{
    pub fn new(stream: TcpStream, logic: L) -> Self {
        let (reader, writer) = tokio::io::split(stream);
        Self { reader, writer, logic }
    }

    pub async fn run(mut self) -> tokio::io::Result<()> {
        let mut buffer = vec![0u8; 1024];

        loop {
            let n = self.reader.read(&mut buffer).await?;

            if n == 0 {
                break; // connection closed
            }

            let msg = L::Message::from(buffer[..n].to_vec());

            if let Some(response) = self.logic.on_message(msg) {
                let bytes: Vec<u8> = response.into();
                self.writer.write_all(&bytes).await?;
            }
        }

        Ok(())
    }
}