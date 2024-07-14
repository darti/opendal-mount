use std::{collections::HashMap, io};

use nfsserve::tcp::{NFSTcp, NFSTcpListener};
use opendal::Operator;
use uuid::Uuid;

use crate::OpendalFs;

#[derive(Default)]
pub struct NFSServer {
    listeners: HashMap<Uuid, NFSTcpListener<OpendalFs>>,
}

impl NFSServer {
    pub async fn register(&mut self, ipstr: &str, op: Operator) -> io::Result<Uuid> {
        let fs = OpendalFs::new(op);

        let listener = NFSTcpListener::bind(ipstr, fs).await?;
        listener.handle().await?;

        let id = Uuid::new_v4();
        self.listeners.insert(id, listener);

        Ok(id)
    }

    pub async fn unregister(&mut self, id: &Uuid) -> bool {
        let listener = self.listeners.remove(id);

        if let Some(listener) = listener {
            let task = listener.stop().await;
            task.wait().await;

            true
        } else {
            false
        }
    }

    pub fn file_systems(&self) -> Vec<Uuid> {
        self.listeners.keys().cloned().collect()
    }
}
