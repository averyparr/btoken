# btoken
Bit-Set Based Byte Pair Encoding

This library is a fairly quickly thrown-together implementation of byte pair encoding. At its core, it uses a labled bit-set based labeled 256-tree to process tokens byte-by-byte, where each node consists of a `[u32; 8]` bit vector, a `u64` token ID, and a `usize` providing an offset specifying where the children of the node (should any exist) begin in memory. 

The API is fairly straightforward, and should hopefully be legible from the `lib.rs`. Right now, everything is centered around the `Tokenizer` object. There are a few ways to create it, but it ends up having exactly one method: `tokenize()` (as `detokenize()` was less interesting, and I am primarily doing this for myself). It works like this:

```py
import numpy as np, btoken
text = "The quick brown fox jumps over the lazy dog."
toks = ["The ", "quick ", "brown ", "fox ", "jumps ", "over ", "the ", "lazy ", "dog", "."]
tokenizer = btoken.Tokenizer.from_str_vec(toks)
np.testing.assert_equal(tokenizer.tokenize(text), np.arange(10))
```

The implementation ends up being mildly faster than `tiktoken`, at least in my own, local benchmarking. Tokenizing (my) `/usr/share/dict/words` on an M1 Pro MacBook, I get a runtime of 495 ms using `tiktoken`'s `o200k_base`, and 340 ms using `btoken`. The ratio is relatively stable for at least long strings (when tokenizing short prompts, we seem dominated by Python -> Rust overhead). 
