ps aux | grep "target/release/examples/"
pkill -f "target/release/examples/"
pkill -f "match-compute"

echo "Starting Program"
cargo build --release
cd ../src
cargo run --release --bin parallel-server &
sleep 1
cargo run --release --bin parallel-client &
