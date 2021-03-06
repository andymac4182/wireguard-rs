// WireGuard semantics constants

pub const MAX_QUEUED_PACKETS: usize = 1024;

// performance constants

pub const PARALLEL_QUEUE_SIZE: usize = MAX_QUEUED_PACKETS;
pub const INORDER_QUEUE_SIZE: usize = MAX_QUEUED_PACKETS;
pub const MAX_INORDER_CONSUME: usize = INORDER_QUEUE_SIZE;
