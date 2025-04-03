set -e # Exit immediately if a command exits with a non-zero status

# すべての子プロセスを終了させる trap
trap 'kill -TERM $(jobs -p)' EXIT

# check if port 8000 is available
if lsof -i:8000 >/dev/null; then
    echo "Port 8000 is already in use. Please free it or use a different port."
    exit 1
fi

# initial build
make build

# parallel execution of watch => build
watchexec --postpone -q -w book/src -w book/assets -e md,toml,js -- make build_book &
PID1=$!
echo "📂 watchexec for build_book started with PID: $PID1"

watchexec --postpone -q -w models -w test_global_tape -e rs -- make build_models copy_assets build_book &
PID2=$!
echo "📂 watchexec for build_models and copy_assets started with PID: $PID2"

watchexec --postpone -q -w models -w test_global_tape -e js,html -- make copy_assets build_book &
PID3=$!
echo "📂 watchexec for copy_assets started with PID: $PID3"

echo "🌐 Starting server at http://localhost:8000"
python3 -m http.server

# 終了を待つ
wait $PID1 $PID2 $PID3
