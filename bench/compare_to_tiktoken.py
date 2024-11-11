#!python

import timeit
from typing import Optional
import btoken, tiktoken


def bench_tokenize(path: Optional[str] = None):
    if path is not None:
        with open(path) as intext:
            text = intext.read()
    else:
        # Do this here to minimize required dependencies
        import lorem

        text = lorem.text()

    enc = tiktoken.get_encoding("o200k_base")
    token_ids = enc.decode_batch([[i] for i in range(enc.n_vocab - 21)])
    tokenizer = btoken.Tokenizer.from_str_vec(token_ids)
    enc.encode(text)
    tokenizer.tokenize(text)

    tiktoken_timer = timeit.Timer(stmt="enc.encode(text)", globals=locals())
    tiktoken_runtime = tiktoken_timer.timeit(10000) / 10000
    print(f"tiktoken runtime: {tiktoken_runtime*1e3:6.3f}ms")
    btoken_timer = timeit.Timer(stmt="tokenizer.tokenize(text)", globals=locals())
    btoken_runtime = btoken_timer.timeit(10000) / 10000
    print(f"btoken runtime: {btoken_runtime*1e3:6.3f}ms")


if __name__ == "__main__":
    bench_tokenize()
