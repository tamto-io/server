use crate::client::{ClientError, MockClient};
use crate::service::tests;
use crate::service::tests::{get_lock, MTX};
use crate::{NodeService, NodeId};
use mockall::predicate;
use std::net::SocketAddr;

#[tokio::test]
async fn join_test() {
    let _m = get_lock(&MTX);
    let ctx = MockClient::init_context();

    ctx.expect().returning(|addr: SocketAddr| {
        let mut client = MockClient::new();
        if addr.port() == 42115 {
            client
                .expect_find_successor()
                .with(predicate::eq(NodeId(1)))
                .times(1)
                .returning(|_| Ok(tests::node(115)));
        }

        client
    });
    let service: NodeService<MockClient> =
        NodeService::with_id(1, SocketAddr::from(([127, 0, 0, 1], 42001)), 3);

    service.join(tests::node(115)).await.unwrap();

    assert_eq!(service.store.db().successor().id, NodeId(115));
}

#[tokio::test]
async fn join_error_test() {
    let _m = get_lock(&MTX);
    let ctx = MockClient::init_context();

    ctx.expect().returning(|addr: SocketAddr| {
        let mut client = MockClient::new();
        if addr.port() == 42116 {
            client
                .expect_find_successor()
                .with(predicate::eq(NodeId(2)))
                .times(1)
                .returning(|_| Err(ClientError::Unexpected("Test".to_string())));
        }
        client
    });
    let service: NodeService<MockClient> =
        NodeService::with_id(2, SocketAddr::from(([127, 0, 0, 1], 42001)), 3);

    let result = service.join(tests::node(116)).await;

    assert!(result.is_err());
    let message = result.unwrap_err().to_string();
    assert_eq!(message, "Client error: Test");
}
