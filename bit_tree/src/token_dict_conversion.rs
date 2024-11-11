use std::collections::HashMap;

use crate::bittree::{ByteTree, PointerByteNode};

fn trace_hashset_into_pointer_nodes(
    prefix: &mut Vec<u8>,
    best_token_guess: u64,
    prefixed_token_dict: &HashMap<&[u8], u64>,
) -> PointerByteNode {
    let mut root_node = PointerByteNode::new(best_token_guess);
    for i in 0..=255 {
        prefix.push(i);
        if let Some(subtok_value) = prefixed_token_dict.get(prefix.as_slice()) {
            let subtree =
                trace_hashset_into_pointer_nodes(prefix, *subtok_value, prefixed_token_dict);
            root_node.insert_child(i as usize, subtree);
        }
        prefix.pop();
    }
    root_node
}

fn include_token_prefixes<'a, 'key>(
    token_dict: &'a HashMap<&'key [u8], u64>,
) -> HashMap<&'key [u8], u64> {
    let mut keys_by_length: HashMap<usize, Vec<&[u8]>> = HashMap::new();
    for k in token_dict.keys() {
        if let Some(keylist) = keys_by_length.get_mut(&k.len()) {
            keylist.push(&k);
        } else {
            keys_by_length.insert(k.len(), vec![&k]);
        }
    }
    let max_keylen = *keys_by_length.keys().max().expect("Empty keys by length");
    let mut token_dict_with_substrings = token_dict.clone();
    for token_length in (0..=max_keylen).rev() {
        if let Some(token_list) = keys_by_length.get(&token_length) {
            for full_token in token_list {
                let token_id = *token_dict.get(full_token).expect("Missing token");
                // Iterate backwards: If we find anything which is a prefix to our token,
                // then that will take over the token IDs for this.
                // Note that we never hit substr_len == token_length, but are
                // covered by cloning the initial keys
                for substr_len in (1..token_length).rev() {
                    if token_dict.contains_key(&full_token[..substr_len]) {
                        break;
                    }
                    token_dict_with_substrings.insert(&full_token[..substr_len], token_id);
                }
            }
        }
    }
    token_dict_with_substrings
}

fn convert_to_pointernode(token_dict: &HashMap<&[u8], u64>) -> PointerByteNode {
    let token_dict_with_prefixes = include_token_prefixes(&token_dict);
    let mut key_stor = vec![];
    trace_hashset_into_pointer_nodes(&mut key_stor, u64::MAX, &token_dict_with_prefixes)
}

pub(crate) fn convert_to_bytetree(token_dict: &HashMap<&[u8], u64>) -> ByteTree {
    convert_to_pointernode(token_dict).finalize()
}

#[test]
fn test_token_prefixing() {
    let mut token_dict: HashMap<&[u8], u64> = HashMap::new();

    token_dict.insert(&[0, 1, 5], 0);
    token_dict.insert(&[0, 1, 5, 6, 8], 1);
    token_dict.insert(&[128, 3], 2);
    token_dict.insert(&[128], 3);
    token_dict.insert(&[192, 34], 3);

    let prefixed_token_dict = include_token_prefixes(&token_dict);

    let mut ref_dict: HashMap<&[u8], u64> = HashMap::new();
    ref_dict.insert(&[0], 0);
    ref_dict.insert(&[0, 1], 0);
    ref_dict.insert(&[0, 1, 5], 0);
    ref_dict.insert(&[0, 1, 5, 6], 1);
    ref_dict.insert(&[0, 1, 5, 6, 8], 1);
    ref_dict.insert(&[128, 3], 2);
    ref_dict.insert(&[128], 3);
    ref_dict.insert(&[192, 34], 3);
    ref_dict.insert(&[192], 3);

    assert_eq!(ref_dict.len(), prefixed_token_dict.len());
    assert_eq!(ref_dict, prefixed_token_dict);
}

#[test]
fn test_pointer_bytenode() {
    let mut token_dict: HashMap<&[u8], u64> = HashMap::new();

    token_dict.insert(&[0, 1, 5], 0);
    token_dict.insert(&[0, 1, 5, 6, 8], 1);
    token_dict.insert(&[128, 3], 2);
    token_dict.insert(&[128], 3);
    token_dict.insert(&[192, 34], 7);

    let tree = convert_to_pointernode(&token_dict);
    assert_eq!(tree.child_unwrap(0).value(), 0);
    assert_eq!(tree.child_unwrap(0).child_unwrap(1).value(), 0);
    let first_tok_node = tree.child_unwrap(0).child_unwrap(1).child_unwrap(5);
    assert_eq!(first_tok_node.value(), 0);

    assert_eq!(first_tok_node.child_unwrap(6).value(), 1);
    assert_eq!(first_tok_node.child_unwrap(6).child_unwrap(8).value(), 1);

    assert_eq!(tree.child_unwrap(128).value(), 3);
    assert_eq!(tree.child_unwrap(128).child_unwrap(3).value(), 2);

    assert_eq!(tree.child_unwrap(192).child_unwrap(34).value(), 7);
}
