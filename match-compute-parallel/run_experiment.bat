echo "Starting Program"

start cmd /c cargo run --release --bin parallel-server ^& pause
timeout /t 1
start cmd /c cargo run --release --bin parallel-client ^& pause
