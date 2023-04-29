use crate::client::MockClient;
use crate::service::tests::{get_lock, MTX};
use crate::{NodeService, NodeId};
use std::net::SocketAddr;

#[tokio::test]
async fn fix_fingers_test() {
    let _m = get_lock(&MTX);
    let ctx = MockClient::init_context();

    ctx.expect().returning(|addr: SocketAddr| {
        let mut client = MockClient::new();
        if addr.port() == 42014 {
            client.mock_find_successor(NodeId(16), 19);
        }
        if addr.port() == 42019 {
            client.mock_find_successor(NodeId(24), 28);
        }
        if addr.port() == 42028 {
            client.mock_find_successor(NodeId(40), 42);
        }

        client
    });
    let mut service: NodeService<MockClient> =
        NodeService::with_id(8, SocketAddr::from(([127, 0, 0, 1], 42008)), 3);
    service.with_fingers_sized(6, vec![1, 14, 21, 32, 38, 42, 48, 51]);

    let mut finger_ids = vec![14; 3];
    finger_ids.append(&mut vec![21, 32, 42]);
    finger_ids.append(&mut vec![8; 58]);
    assert_eq!(
        service.collect_finger_node_ids(),
        finger_ids
    );
    // assert_eq!(service.collect_finger_ids(), vec![9, 10, 12, 16, 24, 40, 72, 136, 264, 520, 1032, 2056, 4104, 8200, 16392, 32776, 65544, 131080, 262152, 524296, 1048584, 2097160, 4194312, 8388616, 16777224, 33554440, 67108872, 134217736, 268435464, 536870920, 1073741832, 2147483656, 4294967304, 8589934600, 17179869192, 34359738376, 68719476744, 137438953480, 274877906952, 549755813896, 1099511627784, 2199023255560, 4398046511112, 8796093022216, 17592186044424, 35184372088840, 70368744177672, 140737488355336, 281474976710664, 562949953421320, 1125899906842632, 2251799813685256, 4503599627370504, 9007199254741000, 18014398509481992, 36028797018963976, 72057594037927944, 144115188075855880, 288230376151711752, 576460752303423496, 1152921504606846984, 2305843009213693960, 4611686018427387912, 9223372036854775816]);

    // service.fix_fingers().await;

    // assert_eq!(
    //     service.collect_finger_node_ids(),
    //     vec![14, 14, 14, 19, 28, 42]
    // );
    // assert_eq!(service.collect_finger_ids(), vec![9, 10, 12, 16, 24, 40]);
}
