ps aux | grep "target/release/examples/"
pkill -f "target/release/examples/"

echo "Starting Program"

cargo run --release --bin parallel-server &
sleep 1
cargo run --release --bin parallel-client &
