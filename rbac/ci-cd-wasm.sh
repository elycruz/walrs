echo "Running wasm-pack, wasm-opt, and node tests ..." && \
wasm-pack build --target nodejs --no-default-features --features wasm && \
wasm-opt -Oz pkg/walrs_rbac_bg.wasm -o pkg/walrs_rbac_bg.wasm && \
cd tests-js && \
npm install && npm test
