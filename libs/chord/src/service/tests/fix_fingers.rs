use crate::client::MockClient;
use crate::service::tests::{get_lock, MTX};
use crate::NodeService;
use std::net::SocketAddr;

#[tokio::test]
// #[ignore = "The fix_fingers method needs to be reworked so it can be used in async"]
async fn fix_fingers_test() {
    let _m = get_lock(&MTX);
    let ctx = MockClient::init_context();

    ctx.expect().returning(|addr: SocketAddr| {
        let mut client = MockClient::new();
        if addr.port() == 42014 {
            client.mock_find_successor(16, 19);
        }
        if addr.port() == 42019 {
            client.mock_find_successor(24, 28);
        }
        if addr.port() == 42028 {
            client.mock_find_successor(40, 42);
        }

        client
    });
    let mut service: NodeService<MockClient> =
        NodeService::with_id(8, SocketAddr::from(([127, 0, 0, 1], 42008)));
    service.with_fingers_sized(6, vec![1, 14, 21, 32, 38, 42, 48, 51]);

    assert_eq!(service.store.finger_table.len(), 6);
    assert_eq!(
        service.collect_finger_node_ids(),
        vec![14, 14, 14, 21, 32, 42]
    );
    assert_eq!(service.collect_finger_ids(), vec![9, 10, 12, 16, 24, 40]);

    service.fix_fingers().await;

    assert_eq!(service.store.finger_table.len(), 6);
    assert_eq!(
        service.collect_finger_node_ids(),
        vec![14, 14, 14, 19, 28, 42]
    );
    assert_eq!(service.collect_finger_ids(), vec![9, 10, 12, 16, 24, 40]);
}
