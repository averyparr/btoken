python3 -m venv tmp_test_venv
source tmp_test_venv/bin/activate
pip install numpy tiktoken pytest maturin
maturin develop
pytest test
deactivate
rm -rf tmp_test_venv