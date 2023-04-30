use std::{net::SocketAddr, sync::Arc, time::Duration};

use crate::{Client, Node, NodeService};

pub async fn join_ring<T: Client + Clone + Sync + Send + 'static>(
    node_service: Arc<NodeService<T>>,
    ring: SocketAddr,
    max_retries: u32,
) {
    // TODO: make this configurable
    const WAIT_BETWEEN_RETRIES: Duration = Duration::from_secs(3);
    let mut attempt = 0;
    loop {
        attempt += 1;
        log::info!("{} attempt to join ring: {:?}", attempt, ring);

        let node = Node::new(ring);
        tokio::time::sleep(Duration::from_secs(1)).await;

        if let Ok(_) = node_service.join(node).await {
            log::info!("Joined ring: {:?}", ring);
            break;
        } else {
            if attempt >= max_retries {
                log::error!("Failed to join ring: {:?}", ring);
                panic!("Failed to join ring: {:?}", ring)
            }
        }

        tokio::time::sleep(WAIT_BETWEEN_RETRIES).await;
    }
}

pub fn background_tasks<T: Client + Clone + Sync + Send + 'static>(
    node_service: Arc<NodeService<T>>,
) {
    let service = node_service.clone();

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            if let Err(err) = service.stabilize().await {
                log::error!("Stabilize error: {:?}", err);
            }

            if let Err(err) = service.check_predecessor().await {
                log::error!("Check predecessor error: {:?}", err);
            }

            if let Err(err) = service.reconcile_successors().await {
                log::error!("Reconcile successors error: {:?}", err);
            }

            service.fix_fingers().await;
        }
    });
}
