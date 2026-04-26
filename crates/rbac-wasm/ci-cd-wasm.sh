echo "Running wasm-pack, wasm-opt, and node tests ..." && \
wasm-pack build --target nodejs && \
wasm-opt -Oz pkg/walrs_rbac_wasm_bg.wasm -o pkg/walrs_rbac_wasm_bg.wasm && \
cd tests-js && \
npm install && npm test
