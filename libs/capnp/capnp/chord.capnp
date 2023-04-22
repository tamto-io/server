@0x9fd191a2e74c5ef6;

struct Option(T) {
  union {
    none @0 :Void;
    some @1 :T;
  }
}

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
  getPredecessor @2 () -> (node :Option(Node));
  notify @3 (node :Node);

  # Inspection methods
  # TODO: It should be moved to a separate service 
  getFingerTable @4 () -> (fingerTable :List(Node));
}
