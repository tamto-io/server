@0x9fd191a2e74c5ef6;

interface ChordNode {
  struct Node {
    id @0 :UInt64;
    address @1 :IpAddress;

    struct IpAddress {
      port @0 :UInt16;

      union {
        ipv4 @1 :List(UInt8);
        ipv6 @2 :List(UInt16);
      }
    }
  }

  ping @0 ();
  findSuccessor @1 (id :UInt64) -> (node :Node);
}
