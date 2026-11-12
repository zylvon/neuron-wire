//! Application-layer fragmentation for large messages.
//!
//! Splits messages larger than the MTU into smaller fragments and
//! reassembles them at the receiver. Each fragment carries a fragment
//! ID, total count, and sequence number for reassembly.

/// Maximum payload size per fragment (MTU-safe).
pub const MAX_FRAGMENT_SIZE: usize = 1200;

/// A single fragment of a larger message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    /// Unique message ID (for reassembly).
    pub message_id: u32,
    /// Fragment index (0-based).
    pub index: u16,
    /// Total number of fragments.
    pub total: u16,
    /// Fragment payload.
    pub payload: Vec<u8>,
}

impl Fragment {
    /// Serialize a fragment to bytes.
    /// Format: [4B msg_id][2B index][2B total][payload...]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8 + self.payload.len());
        buf.extend_from_slice(&self.message_id.to_le_bytes());
        buf.extend_from_slice(&self.index.to_le_bytes());
        buf.extend_from_slice(&self.total.to_le_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }

    /// Deserialize a fragment from bytes.
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }
        let message_id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let index = u16::from_le_bytes([data[4], data[5]]);
        let total = u16::from_le_bytes([data[6], data[7]]);
        if total == 0 || index >= total {
            return None;
        }
        let payload = data[8..].to_vec();
        Some(Fragment {
            message_id,
            index,
            total,
            payload,
        })
    }
}

/// Fragment a message into MTU-safe pieces.
pub fn fragment_message(message_id: u32, data: &[u8]) -> Vec<Fragment> {
    if data.is_empty() {
        return vec![Fragment {
            message_id,
            index: 0,
            total: 1,
            payload: Vec::new(),
        }];
    }

    let total = data.len().div_ceil(MAX_FRAGMENT_SIZE);
    let total = total as u16;

    (0..total)
        .map(|i| {
            let start = (i as usize) * MAX_FRAGMENT_SIZE;
            let end = (start + MAX_FRAGMENT_SIZE).min(data.len());
            Fragment {
                message_id,
                index: i,
                total,
                payload: data[start..end].to_vec(),
            }
        })
        .collect()
}

/// Reassemble fragments into a complete message.
/// Returns `Some(data)` when all fragments are present, `None` otherwise.
pub fn reassemble(fragments: &[Fragment]) -> Option<Vec<u8>> {
    if fragments.is_empty() {
        return None;
    }

    let total = fragments[0].total;
    let message_id = fragments[0].message_id;

    // Validate all fragments belong to the same message
    for f in fragments {
        if f.message_id != message_id || f.total != total {
            return None;
        }
    }

    // Check we have all fragments
    if fragments.len() != total as usize {
        return None;
    }

    // Check indices are 0..total
    let mut indices: Vec<u16> = fragments.iter().map(|f| f.index).collect();
    indices.sort();
    for (i, &idx) in indices.iter().enumerate() {
        if idx != i as u16 {
            return None;
        }
    }

    // Reassemble in order
    let mut result = Vec::new();
    for i in 0..total {
        let frag = fragments.iter().find(|f| f.index == i)?;
        result.extend_from_slice(&frag.payload);
    }

    Some(result)
}

/// Fragment reassembly buffer for tracking partial messages.
#[derive(Debug, Default)]
pub struct ReassemblyBuffer {
    /// Pending fragments keyed by message_id.
    pending: std::collections::HashMap<u32, Vec<Fragment>>,
}

impl ReassemblyBuffer {
    /// Create a new reassembly buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a fragment. Returns `Some(complete_data)` if the message is now complete.
    pub fn add(&mut self, fragment: Fragment) -> Option<Vec<u8>> {
        let msg_id = fragment.message_id;
        let entry = self.pending.entry(msg_id).or_default();
        entry.push(fragment);

        // Check if complete
        if let Some(first) = entry.first() {
            let total = first.total as usize;
            if entry.len() == total {
                let result = reassemble(entry);
                self.pending.remove(&msg_id);
                return result;
            }
        }
        None
    }

    /// Number of pending (incomplete) messages.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Clear all pending fragments.
    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_small_message() {
        let data = b"hello";
        let frags = fragment_message(1, data);
        assert_eq!(frags.len(), 1);
        assert_eq!(frags[0].index, 0);
        assert_eq!(frags[0].total, 1);
        assert_eq!(frags[0].payload, data);
    }

    #[test]
    fn test_fragment_large_message() {
        let data = vec![0xABu8; 3000];
        let frags = fragment_message(42, &data);
        assert_eq!(frags.len(), 3); // 1200 + 1200 + 600
        assert_eq!(frags[0].total, 3);
        assert_eq!(frags[1].total, 3);
        assert_eq!(frags[2].total, 3);
    }

    #[test]
    fn test_reassemble() {
        let data = vec![0xABu8; 3000];
        let frags = fragment_message(42, &data);
        let reassembled = reassemble(&frags).unwrap();
        assert_eq!(reassembled, data);
    }

    #[test]
    fn test_reassemble_incomplete() {
        let data = vec![0xABu8; 3000];
        let frags = fragment_message(42, &data);
        // Remove one fragment
        let incomplete: Vec<_> = frags.into_iter().take(2).collect();
        assert!(reassemble(&incomplete).is_none());
    }

    #[test]
    fn test_fragment_roundtrip() {
        let data = vec![0xCDu8; 5000];
        let frags = fragment_message(7, &data);
        for frag in &frags {
            let bytes = frag.to_bytes();
            let parsed = Fragment::from_bytes(&bytes).unwrap();
            assert_eq!(parsed, *frag);
        }
    }

    #[test]
    fn test_reassembly_buffer() {
        let data = vec![0xEFu8; 2500];
        let frags = fragment_message(99, &data);

        let mut buf = ReassemblyBuffer::new();
        // Add fragments one at a time
        for frag in &frags[..frags.len() - 1] {
            assert!(buf.add(frag.clone()).is_none());
        }
        // Last fragment completes the message
        let result = buf.add(frags[frags.len() - 1].clone());
        assert!(result.is_some());
        assert_eq!(result.unwrap(), data);
        assert_eq!(buf.pending_count(), 0);
    }

    #[test]
    fn test_empty_message() {
        let frags = fragment_message(1, &[]);
        assert_eq!(frags.len(), 1);
        let reassembled = reassemble(&frags).unwrap();
        assert!(reassembled.is_empty());
    }

    #[test]
    fn test_from_bytes_too_short() {
        assert!(Fragment::from_bytes(&[0, 1, 2]).is_none());
    }

    #[test]
    fn test_from_bytes_invalid_index() {
        // index >= total
        let bytes = [0u8, 0, 0, 0, 5, 0, 3, 0]; // index=5, total=3
        assert!(Fragment::from_bytes(&bytes).is_none());
    }
}
