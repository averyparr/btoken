use std::{collections::HashMap, iter::Peekable, u64};

use bittree::ByteTree;

mod bitset;
mod bittree;
mod token_dict_conversion;

pub struct Tokenizer {
    tree: ByteTree,
}

impl Tokenizer {
    pub fn from_token_dict(token_dict: HashMap<&str, u64>) -> Self {
        let dict_with_byte_keys = token_dict.iter().map(|(k, &v)| (k.as_bytes(), v)).collect();
        let tree = token_dict_conversion::convert_to_bytetree(&dict_with_byte_keys);
        Self { tree }
    }

    pub fn from_byte_token_dict(token_dict: HashMap<&[u8], u64>) -> Self {
        let tree = token_dict_conversion::convert_to_bytetree(&token_dict);
        Self { tree }
    }

    pub fn nibble_token(&self, bytes: &mut Peekable<impl Iterator<Item = u8>>) -> u64 {
        let mut walker = self.tree.walker();
        if let Some(_) = bytes.peek() {
            while let Some(next_walker) =
                walker.step(*bytes.peek().expect("Casework failure!") as usize)
            {
                walker = next_walker;
                _ = bytes.next().unwrap();
                if bytes.peek().is_none() {
                    break;
                }
            }
            walker.value()
        } else {
            u64::MAX
        }
    }
}

#[test]
fn test_tokenizer() {
    let token_dict = [("hello ", 1), ("world", 2), (".", 3), ("!", 4)]
        .into_iter()
        .collect();

    let tokenizer = Tokenizer::from_token_dict(token_dict);

    let input = "hello world!";

    let mut bytes = input.as_bytes().into_iter().map(|&b| b).peekable();

    assert_eq!(tokenizer.nibble_token(&mut bytes), 1);
    assert_eq!(tokenizer.nibble_token(&mut bytes), 2);
    assert_eq!(tokenizer.nibble_token(&mut bytes), 4);
    assert_eq!(tokenizer.nibble_token(&mut bytes), u64::MAX);
}
