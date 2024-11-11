#!pytest

import numpy as np
import btoken
import tiktoken


def test_tokenizer():
    text = "The quick brown fox jumps over the lazy dog."

    token_mapping = {
        "The ": 0,
        "quick ": 5,
        "brown ": 3,
        "fox ": 10,
        "jumps ": 1023,
        "over ": 334,
        "the ": 95,
        "lazy ": 9,
        "dog": 100,
        ".": 6,
        "\t": 91,
        "\n": 13,
        " ": 14,
    }
    tokenizer = btoken.Tokenizer.from_str_dict(token_mapping)
    btoken_toks = tokenizer.tokenize(text)
    np.testing.assert_equal(btoken_toks, [0, 5, 3, 10, 1023, 334, 95, 9, 100, 6])

    # Commented out because they make testing very slow

    # enc = tiktoken.get_encoding("gpt2")
    # token_ids = enc.decode_batch(np.arange(enc.n_vocab)[:, None])
    # tokenizer = btoken.Tokenizer.from_str_vec(token_ids)
    # own = tokenizer.tokenize(text)
    # ref = np.array(enc.encode(text))
    # np.testing.assert_equal(own, ref)
