# Run tests
echo "Running tests..."
if command -v nvidia-smi >/dev/null 2>&1 && \
   nvidia-smi -L 2>/dev/null | grep -qE '^GPU [0-9]+'; then
    cargo llvm-cov nextest --html --open --branch --features neco-felis-compile/has-ptx-device
else
    cargo llvm-cov nextest --html --open --branch
fi
