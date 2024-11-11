# Rust library testing
# Note: the `btoken` library is really just Python bindings for 
# the `bit_tree` library, so we don't test `btoken` directly.
cargo test --workspace 

# Python library testing
python3 -m venv tmp_test_venv
source tmp_test_venv/bin/activate
pip install numpy tiktoken pytest maturin
maturin develop
pytest test
deactivate
rm -rf tmp_test_venv