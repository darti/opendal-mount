use std::{
    collections::HashMap,
    io,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use log::debug;
use nfsserve::service::NFSService;
use opendal::Operator;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::OpendalFs;

#[derive(Clone, Default)]
pub struct NFSServer {
    inner: Arc<Mutex<NFSInner>>,
}

#[derive(Default)]
struct NFSInner {
    services: HashMap<Uuid, NFSService<OpendalFs>>,
}

impl NFSServer {
    pub async fn register(&self, ipstr: &str, op: Operator) -> io::Result<Uuid> {
        debug!("Starting nfs service on {}", ipstr);
        let fs = OpendalFs::new(op);

        let service = NFSService::new(fs, ipstr).await?;
        let recorded_service = service.clone();

        let _s = tokio::spawn(async move { service.handle().await });

        let id = Uuid::new_v4();
        self.inner
            .lock()
            .await
            .services
            .insert(id, recorded_service);

        Ok(id)
    }

    pub async fn unregister(&self, id: &Uuid) -> bool {
        let listener = self.inner.lock().await.services.remove(id);

        if let Some(listener) = listener {
            let task = listener.stop().await;
            task.wait().await;

            true
        } else {
            false
        }
    }

    pub async fn local_addr(&self, id: &Uuid) -> Option<SocketAddr> {
        self.inner
            .lock()
            .await
            .services
            .get(id)
            .map(|s| s.local_addr())
    }

    pub async fn file_systems(&self) -> Vec<Uuid> {
        self.inner.lock().await.services.keys().cloned().collect()
    }
}
