use crate::primitives::block::Block;

fn string_of_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(ToOwned::to_owned)
        .map(char::from)
        .collect()
}

pub fn block_as_json(block: &Block) -> serde_json::Value {
    let (hash, previous_hash, data) = (
        string_of_bytes(&block.hash),
        string_of_bytes(&block.previous_hash),
        string_of_bytes(block.data.buffer()),
    );

    serde_json::json!({ "index": block.index, "data": data, "timestamp": block.timestamp, "hash": hash, "previous_hash": previous_hash })
}
