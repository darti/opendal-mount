use std::{collections::HashMap, io, net::IpAddr, sync::Arc};

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

        let service = NFSService::new(fs);
        let recorded_service = service.clone();

        let addr = ipstr.to_owned();

        let s = tokio::spawn(async move { service.handle(addr).await });

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

    pub async fn file_systems(&self) -> Vec<Uuid> {
        self.inner.lock().await.services.keys().cloned().collect()
    }
}
