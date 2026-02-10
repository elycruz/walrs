echo "Running wasm-pack, wasm-opt, and node tests ... sleeping for 3" && \
sleep 3 && \
wasm-pack build --target nodejs --no-default-features --features wasm && \
wasm-opt -Oz pkg/walrs_acl_bg.wasm -o pkg/walrs_acl_bg.wasm && \
cd tests-js && \
npm install && npm test
