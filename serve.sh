# すべての子プロセスを終了させる trap
trap 'kill $(jobs -p)' EXIT

# 初回ビルド
make build

# 並列実行：watch & server
watchexec -q -w book/src -w book/assets -e md,toml -- make build_book &

watchexec -q -w models -e rs,js,html -- make build_models copy_assets &

echo "🌐 Starting server at http://localhost:8000"
python3 -m http.server

# 終了を待つ
wait
