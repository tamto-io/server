use crate::client::MockClient;
use crate::service::tests;
use crate::service::tests::{get_lock, MTX};
use crate::{NodeService, NodeId};
use std::net::SocketAddr;

#[tokio::test]
async fn test_updating_successor_list_from_successor() {
    let _m = get_lock(&MTX);
    let ctx = MockClient::init_context();

    ctx.expect().returning(|addr: SocketAddr| {
        let mut client = MockClient::new();
        if addr.port() == 42016 {
            client
                .expect_predecessor()
                .returning(|| Ok(Some(tests::node(1))));

            client.expect_successor_list().returning(|| {
                Ok(vec![
                    tests::node(32),
                    tests::node(64),
                    tests::node(128),
                ])
            });
        }
        client.expect_notify().returning(|_| Ok(()));
        client
    });

    let service = NodeService::test_service(90);
    service.store.db().set_successor(tests::node(16));
    let successor_list = service.store.db().successor_list();
    assert_eq!(successor_list.len(), 1);

    service.reconcile_successors().await.unwrap();

    let successor_list = service.store.db().successor_list();
    assert_eq!(successor_list.len(), 3);
    assert_eq!(successor_list[0].id, NodeId(16));
    assert_eq!(successor_list[1].id, NodeId(32));
    assert_eq!(successor_list[2].id, NodeId(64));
}

#[tokio::test]
async fn test_updating_successor_list_from_successor_which_returns_only_one_node() {
    let _m = get_lock(&MTX);
    let ctx = MockClient::init_context();

    ctx.expect().returning(|addr: SocketAddr| {
        let mut client = MockClient::new();
        if addr.port() == 42016 {
            client
                .expect_predecessor()
                .returning(|| Ok(Some(tests::node(1))));

            client.expect_successor_list().returning(|| {
                Ok(vec![
                    tests::node(32),
                ])
            });
        }
        client.expect_notify().returning(|_| Ok(()));
        client
    });

    let service = NodeService::test_service(90);
    service.store.db().set_successor(tests::node(16));
    let successor_list = service.store.db().successor_list();
    assert_eq!(successor_list.len(), 1);

    service.reconcile_successors().await.unwrap();

    let successor_list = service.store.db().successor_list();
    assert_eq!(successor_list.len(), 2);
    assert_eq!(successor_list[0].id, NodeId(16));
    assert_eq!(successor_list[1].id, NodeId(32));
}

#[tokio::test]
async fn test_updating_successor_list_from_successor_which_returns_too_many_nodes() {
    let _m = get_lock(&MTX);
    let ctx = MockClient::init_context();

    ctx.expect().returning(|addr: SocketAddr| {
        let mut client = MockClient::new();
        if addr.port() == 42016 {
            client
                .expect_predecessor()
                .returning(|| Ok(Some(tests::node(1))));

            client.expect_successor_list().returning(|| {
                Ok(vec![
                    tests::node(32),
                    tests::node(64),
                    tests::node(128),
                    tests::node(256),
                ])
            });
        }
        client.expect_notify().returning(|_| Ok(()));
        client
    });

    let service = NodeService::test_service(90);
    service.store.db().set_successor(tests::node(16));
    let successor_list = service.store.db().successor_list();
    assert_eq!(successor_list.len(), 1);

    service.reconcile_successors().await.unwrap();

    let successor_list = service.store.db().successor_list();
    assert_eq!(successor_list.len(), 3);
    assert_eq!(successor_list[0].id, NodeId(16));
    assert_eq!(successor_list[1].id, NodeId(32));
    assert_eq!(successor_list[2].id, NodeId(64));
}
