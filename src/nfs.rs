use std::io;

use tokio::net::TcpListener;

pub struct NFSServer {}

impl NFSServer {
    pub async fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;

        let context = RPCContext {
            local_port: self.port,
            client_addr: socket.peer_addr().unwrap().to_string(),
            auth: crate::rpc::auth_unix::default(),
            vfs: self.arcfs.clone(),
            mount_signal: self.mount_signal.clone(),
        };

        let (mut message_handler, mut socksend, mut msgrecvchan) =
            SocketMessageHandler::new(&context);
        let _ = socket.set_nodelay(true);

        Ok(Self {})
    }
}
