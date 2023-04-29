use crate::client::MockClient;
use crate::service::tests;
use crate::service::tests::{get_lock, MTX};
use crate::{NodeId, NodeService};
use std::net::SocketAddr;

#[tokio::test]
async fn test_find_successor() {
    let _m = get_lock(&MTX);
    let service: NodeService<MockClient> =
        NodeService::with_id(8, SocketAddr::from(([127, 0, 0, 1], 42001)), 3);
    let result = service.find_successor(NodeId(10)).await;
    assert!(result.is_ok());
    let successor = result.unwrap();

    assert_eq!(successor.id, NodeId(8));
}

#[tokio::test]
async fn find_successor_with_2_nodes() {
    let _m = get_lock(&MTX);
    let ctx = MockClient::init_context();

    ctx.expect().returning(|_| {
        let mut client = MockClient::new();
        client
            .expect_find_successor()
            .times(1)
            .returning(|_| Ok(tests::node(6)));
        client
    });

    let service: NodeService<MockClient> =
        NodeService::with_id(8, SocketAddr::from(([127, 0, 0, 1], 42001)), 3);
    service.store.db().set_successor(tests::node(16));

    assert_eq!(
        service.find_successor(NodeId(10)).await.unwrap().id,
        NodeId(16)
    );
    assert_eq!(
        service.find_successor(NodeId(2)).await.unwrap().id,
        NodeId(6)
    );
}

#[tokio::test]
async fn find_successor_with_2_nodes_but_the_same_id() {
    let _m = get_lock(&MTX);
    let ctx = MockClient::init_context();

    ctx.expect().returning(|addr: SocketAddr| {
        let mut client = MockClient::new();
        if addr.port() == 42006 {
            client
                .expect_find_successor()
                .times(1)
                .returning(|_| Ok(tests::node(6)));
        }
        client
    });

    let service: NodeService<MockClient> =
        NodeService::with_id(6, SocketAddr::from(([127, 0, 0, 1], 42001)), 3);
    service.store.db().set_successor(tests::node(6));

    assert_eq!(
        service.find_successor(NodeId(6)).await.unwrap().id,
        NodeId(6)
    );
    assert_eq!(
        service.find_successor(NodeId(6)).await.unwrap().id,
        NodeId(6)
    );
}

#[tokio::test]
#[ignore]
async fn find_successor_using_finger_table_nodes() {
    let _m = get_lock(&MTX);
    let ctx = MockClient::init_context();

    ctx.expect().returning(|addr: SocketAddr| {
        let mut client = MockClient::new();
        if addr.port() == 42035 {
            client
                .expect_find_successor()
                .times(1)
                .returning(|_| Ok(tests::node(111)));
        }

        if addr.port() == 42001 {
            client
                .expect_find_successor()
                .times(1)
                .returning(|_| Ok(tests::node(5)));
        }
        client
    });

    let mut service: NodeService<MockClient> = NodeService::default();
    service.with_fingers(vec![1, 10, 35, 129]);

    assert_eq!(
        service.find_successor(NodeId(40)).await.unwrap().id,
        NodeId(111)
    );
    assert_eq!(
        service.find_successor(NodeId(2)).await.unwrap().id,
        NodeId(5)
    );
}

#[tokio::test]
async fn check_closest_preceding_node() {
    let mut service: NodeService<MockClient> = NodeService::default();
    service.with_fingers(vec![1, 10, 35, 129]);

    assert_eq!(service.closest_preceding_node(NodeId(2)).id, NodeId(1));
    assert_eq!(service.closest_preceding_node(NodeId(11)).id, NodeId(10));
    assert_eq!(service.closest_preceding_node(NodeId(35)).id, NodeId(10));
    assert_eq!(service.closest_preceding_node(NodeId(100)).id, NodeId(35));
    assert_eq!(service.closest_preceding_node(NodeId(150)).id, NodeId(129));
}
