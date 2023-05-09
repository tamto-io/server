use crate::client::{ClientError, MockClient};
use crate::service::tests::{self, ExpectationExt};
use crate::service::tests::{get_lock, MTX};
use crate::{Node, NodeId, NodeService};
use mockall::predicate;
use std::net::SocketAddr;

#[tokio::test]
async fn stabilize_when_predecessor_is_between_node_and_successor_then_set_set_the_it_as_new_successor(
) {
    let _m = get_lock(&MTX);
    let ctx = MockClient::init_context();

    ctx.expect().returning(|addr: SocketAddr| {
        let mut client = MockClient::new();
        if addr.port() == 42016 {
            client
                .expect_predecessor()
                .times(1)
                .returning(|| Ok(Some(tests::node(12))));
        }

        if addr.port() == 42012 {
            client
                .expect_notify()
                .with(predicate::function(|n: &Node| n.id == NodeId(8)))
                .times(1)
                .returning(|_| Ok(()));
        }
        client
    });

    let service: NodeService<MockClient> =
        NodeService::with_id(8, SocketAddr::from(([127, 0, 0, 1], 42001)), 3);
    service.store.db().set_successor(tests::node(16));

    assert_eq!(service.store.db().successor().id, NodeId(16));
    let result = service.stabilize().await;
    assert!(result.is_ok());

    assert_eq!(service.store.db().successor().id, NodeId(12));
}

#[tokio::test]
async fn when_predecessor_is_not_between_node_and_successor_then_the_old_one_should_be_kept() {
    let _m = get_lock(&MTX);
    let ctx = MockClient::init_context();

    ctx.expect().returning(|addr: SocketAddr| {
        let mut client = MockClient::new();
        if addr.port() == 42016 {
            client
                .expect_predecessor()
                .returning(|| Ok(Some(tests::node(1))));
            client
                .expect_notify()
                .with(predicate::function(|n: &Node| n.id == NodeId(8)))
                .returning(|_| Ok(()));
        }
        client
    });

    let service: NodeService<MockClient> =
        NodeService::with_id(8, SocketAddr::from(([127, 0, 0, 1], 42001)), 3);
    service.store.db().set_successor(tests::node(16));

    assert_eq!(service.store.db().successor().id, NodeId(16));
    let result = service.stabilize().await;
    assert!(result.is_ok());

    assert_eq!(service.store.db().successor().id, NodeId(16));
}

#[test]
fn when_getting_predecessor_fails_then_nothing_should_be_updated() {
    let _m = get_lock(&MTX);
    let ctx = MockClient::init_context();

    ctx.expect().returning(|_| {
        let mut client = MockClient::new();
        client.expect_predecessor().returning_error(ClientError::Unexpected("Test".to_string()));
        client
            .expect_notify()
            .with(predicate::function(|n: &Node| n.id == NodeId(8)))
            .returning(|_| Ok(()));
        client
    });

    let service: NodeService<MockClient> =
        NodeService::with_id(8, SocketAddr::from(([127, 0, 0, 1], 42001)), 3);
    service.store.db().set_successor(tests::node(16));

    assert_eq!(service.store.db().successor().id, NodeId(16));
    let _ = service.stabilize();

    assert_eq!(service.store.db().successor().id, NodeId(16));
}
