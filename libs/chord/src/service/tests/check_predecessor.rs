use crate::client::{ClientError, MockClient};
use crate::service::tests;
use crate::service::tests::{get_lock, MTX};
use crate::NodeService;
use std::net::SocketAddr;

#[test]
fn when_predecessor_is_up_it_should_not_be_removed() {
    let _m = get_lock(&MTX);
    let ctx = MockClient::init_context();

    ctx.expect().returning(|addr: SocketAddr| {
        let mut client = MockClient::new();
        if addr.port() == 42012 {
            client.expect_ping().times(1).returning(|| Ok(()));
        }
        client
    });

    let mut service: NodeService<MockClient> =
        NodeService::with_id(8, SocketAddr::from(([127, 0, 0, 1], 42001)));
    service.store.set_successor(tests::node(16));
    service.store.set_predecessor(tests::node(12));

    service.check_predecessor();

    assert!(service.store.predecessor().is_some());
    assert_eq!(service.store.predecessor().unwrap().id, 12);
}

#[test]
fn when_predecessor_is_down_it_should_be_removed() {
    let _m = get_lock(&MTX);
    let ctx = MockClient::init_context();

    ctx.expect().returning(|addr: SocketAddr| {
        let mut client = MockClient::new();
        if addr.port() == 42016 {
            client
                .expect_ping()
                .times(1)
                .returning(|| Err(ClientError::ConnectionFailed(tests::node(16))));
        }

        client
    });

    let mut service: NodeService<MockClient> =
        NodeService::with_id(8, SocketAddr::from(([127, 0, 0, 1], 42001)));
    service.store.set_successor(tests::node(16));
    service.store.set_predecessor(tests::node(16));

    service.check_predecessor();

    assert!(service.store.predecessor().is_none());
}

#[test]
fn when_ping_fails_with_unexpected_error_predecessor_should_not_be_removed() {
    let _m = get_lock(&MTX);
    let ctx = MockClient::init_context();

    ctx.expect().returning(|addr: SocketAddr| {
        let mut client = MockClient::new();
        if addr.port() == 42008 {
            client
                .expect_ping()
                .times(1)
                .returning(|| Err(ClientError::Unexpected("Error".to_string())));
        }
        client
    });

    let mut service: NodeService<MockClient> =
        NodeService::with_id(8, SocketAddr::from(([127, 0, 0, 1], 42001)));
    service.store.set_successor(tests::node(16));
    service.store.set_predecessor(tests::node(8));

    service.check_predecessor();

    assert!(service.store.predecessor().is_some());
    assert_eq!(service.store.predecessor().unwrap().id, 8);
}
