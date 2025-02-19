use crate::discord_ipc::DiscordIpc;
use async_net::{unix::UnixStream, Shutdown};
use async_trait::async_trait;
use futures_lite::io::AsyncWriteExt;
use futures_lite::AsyncReadExt;
use crate::utils::get_pipe_pattern;
use serde_json::json;
use std::{error::Error};

// Environment keys to search for the Discord pipe

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[allow(dead_code)]
#[allow(missing_docs)]
/// A wrapper struct for the functionality contained in the
/// underlying [`DiscordIpc`](trait@DiscordIpc) trait.
#[derive(Clone)]
pub struct DiscordIpcClient {
  /// Client ID of the IPC client.
  pub client_id: String,
  pub access_token: String,
  pub connected: bool,
  // Socket ref to the open socket
  pub socket: Option<UnixStream>,
}

impl DiscordIpcClient {

  /// Creates a new `DiscordIpcClient`.
  ///
  /// # Examples
  /// ```
  /// let ipc_client = DiscordIpcClient::new("<some client id>")?;
  /// ```
  pub async fn new(client_id: &str, access_token: &str) -> Result<Self> {    

    let mut client = Self {
      client_id: client_id.to_string(),
      access_token: access_token.to_owned(),
      connected: false,
      socket: None,
    };

    // connect to client
    client.connect().await?;

    // let token = client.access_token;
    // client.login(access_token.to_string()).await.ok();

    Ok(client)
  }


}

#[async_trait]
impl DiscordIpc for DiscordIpcClient {
  async fn connect_ipc(&mut self) -> Result<()> {
    // iterate over the likely places to find the socket
    for i in 0..10 {
      let path = get_pipe_pattern().join(format!("discord-ipc-{}", i));

      match UnixStream::connect(&path).await {
        Ok(socket) => {
          println!("Found socket @ {:?}", path);
          self.socket = Some(socket);
          self.connected = true;
          return Ok(());
        }
        Err(_) => continue,
      }
    }

    Err("Couldn't connect to the Discord IPC socket".into())
  }

  async fn write(&mut self, data: &[u8]) -> Result<()> {
    let socket = self.socket.as_mut().expect("Client not connected");

    socket.write_all(data).await?;

    Ok(())
  }

  async fn read(&mut self, buffer: &mut [u8]) -> Result<()> {
    let socket = self.socket.as_mut().unwrap();
    socket.read_exact(buffer).await?;

    Ok(())
  }

  async fn close(&mut self) -> Result<()> {
    let data = json!({});
    if self.send(data.to_string(), 2).await.is_ok() {}

    let socket = self.socket.as_mut().unwrap();

    socket.flush().await?;
    socket.shutdown(Shutdown::Both)?;

    self.connected = false;

    Ok(())
  }

  fn get_client_id(&self) -> String {
    self.client_id.to_owned()
  }

  fn get_client_instance(&self) -> DiscordIpcClient {
    self.clone()
  }
}
