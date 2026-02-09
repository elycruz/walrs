#!/bin/bash

# Script to run the actix benchmark server and test with Apache Bench

echo "Starting ACL Actix-Web benchmark server..."

# Start the server in background
cargo run --release --example benchmark_actix_middleware > /tmp/actix_server.log 2>&1 &
SERVER_PID=$!

echo "Server PID: $SERVER_PID"
echo "Waiting for server to start..."
sleep 8

# Check if server is running
if ! ps -p $SERVER_PID > /dev/null; then
    echo "Server failed to start. Check /tmp/actix_server.log"
    cat /tmp/actix_server.log
    exit 1
fi

echo "Server is running. Running Apache Bench tests..."
echo ""

# Test 1: Basic load test
echo "=== Test 1: Basic Load (10,000 requests, 100 concurrent) ==="
ab -n 10000 -c 100 \
  -H 'X-User-Role: user' \
  -H 'X-Resource: blog' \
  -H 'X-Privilege: read' \
  http://127.0.0.1:8080/

echo ""

# Test 2: Admin access
echo "=== Test 2: Admin Access (5,000 requests, 50 concurrent) ==="
ab -n 5000 -c 50 \
  -H 'X-User-Role: admin' \
  -H 'X-Resource: admin_panel' \
  -H 'X-Privilege: read' \
  http://127.0.0.1:8080/admin

echo ""

# Test 3: High concurrency
echo "=== Test 3: High Concurrency (20,000 requests, 200 concurrent) ==="
ab -n 20000 -c 200 \
  -H 'X-User-Role: moderator' \
  -H 'X-Resource: forum' \
  -H 'X-Privilege: edit' \
  http://127.0.0.1:8080/protected

echo ""
echo "Benchmark complete. Stopping server..."
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null

echo "Done!"
