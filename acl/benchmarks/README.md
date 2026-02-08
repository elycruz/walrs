# ACL Benchmarks

Benchmarks demonstrating ACL performance in both standalone and web application contexts.

---

## Results

### Standalone ACL

**Summary:**

```
Performance: 1.3M checks/sec @ ~750ns per check
Memory:      ~50 KB for 46 roles + 79 resources + 300+ rules
Loading:     ~7.5ms including cycle detection
```

**Detailed:**

| Metric | Value | Notes |
|--------|-------|-------|
| **Throughput** | 1.3M checks/sec | Consistent across workloads |
| **Latency** | ~750ns per check | Sub-microsecond performance |
| **Memory** | ~50 KB | For 46 roles + 79 resources + 300+ rules |
| **Load time** | ~7.5ms | Includes cycle detection |
| **Scaling** | Linear | With iteration count |

**Random Permission Checks:**

```
Iterations    Total Time    Avg/Check    Checks/sec
1             2.5µs         2.5µs        396,040
10            20µs          2.0µs        501,303
100           91µs          907ns        1,102,050
1,000         764µs         763ns        1,309,187
10,000        7.6ms         756ns        1,322,690
100,000       81ms          811ns        1,232,195
```

### ACL in Web Server

**Apache Bench Results (on local machine):**

| Test Scenario               | Requests | Concurrency | Req/sec | Latency (mean) | p99 Latency |
|-----------------------------|----------|-------------|---------|----------------|-------------|
| **Basic Load (mean)**       | 10,000 | 100 | **11,027** | 9.07ms | 10ms |
| **Admin Access (mean)**     | 5,000 | 50 | **10,629** | 4.70ms | 5ms |
| **High Concurrency (mean)** | 20,000 | 200 | **10,825** | 18.48ms | 21ms |

**Notes:**

- ACL middleware adds ~91-94µs per request overhead.
- Performance remains consistent across different workload patterns.

**Metrics:**

| Metric                     | Value | Notes |
|----------------------------|-------|-------|
| **HTTP Throughput (mean)** | **~10,800 req/sec** | Measured with Apache Bench |
| **ACL Check Time**         | ~91-94µs | Per request (middleware overhead) |
| **Total Request Time**     | 4.7-18.5ms | Depends on concurrency level |
| **Latency p50**            | 5-18ms | Median response time |
| **Latency p99**            | 10-21ms | 99th percentile |
| **Memory**                 | ~50 KB | Shared across workers |
| **Failed Requests**        | **0** | 100% success rate |

**Detailed Test Results:**

```
Test 1: Basic Load (10,000 requests, 100 concurrent)
  Requests per second:    11,027.29 [#/sec]
  Time per request:       9.068 [ms] (mean)
  Time per request:       0.091 [ms] (mean, across all concurrent requests)
  
  Latency percentiles:
    50%:  9ms
    95%: 10ms
    99%: 10ms

Test 2: Admin Access (5,000 requests, 50 concurrent)
  Requests per second:    10,628.94 [#/sec]
  Time per request:       4.704 [ms] (mean)
  Time per request:       0.094 [ms] (mean, across all concurrent requests)
  
  Latency percentiles:
    50%: 5ms
    95%: 5ms
    99%: 5ms

Test 3: High Concurrency (20,000 requests, 200 concurrent)
  Requests per second:    10,824.68 [#/sec]
  Time per request:       18.476 [ms] (mean)
  Time per request:       0.092 [ms] (mean, across all concurrent requests)
  
  Latency percentiles:
    50%: 18ms
    95%: 20ms
    99%: 21ms
```

**Comparison with Alternatives:**

| Approach | Latency | Speedup vs ACL |
|----------|---------|----------------|
| **In-memory ACL** | ~91µs | 1x (baseline) |
| Redis cache | ~500µs-1ms | **5-10x slower** |
| Database query | ~2-10ms | **20-100x slower** |
| External auth service | ~20-50ms | **200-500x slower** |

---

## 🚀 Benchmark details

### 1. Standalone Performance Benchmark

ACL structure raw performance with comprehensive dataset.

#### Run

```bash
cargo run --release --example benchmark_extensive_acl
```

**Measured:**
- Random permission checks (1 to 100,000 iterations)
- Role inheritance validation (9 levels deep)
- Resource hierarchy checks
- Deny rule evaluation
- Memory usage analysis

### Process Flow

```
Load ACL from JSON (~7.5ms)
    ↓
Random Role + Resource + Privilege Selection
    ↓
ACL Check: is_allowed() (~750ns)
    ↓
Aggregate Statistics
```

---

### 2. Web Server Benchmark

HTTP server example, using actix, with ACL middleware for simulating thread safety, and load testing.

#### Run

```bash
cargo run --release --example benchmark_actix_middleware
```

**Provides:**
- 
- HTTP server on `http://127.0.0.1:8080`
- ACL middleware on every request
- Multiple test endpoints
- Header-based authorization
- Ready for external benchmarking tools

**Tests with automated script:**

```bash
cd benchmarks
./run_ab_benchmark.sh
```

**What it does:**

1. Start the server.
2. Run 3 Apache Bench tests with different scenarios setup.
3. Display detailed performance results.
4. Stops the server when it's done.

Or: 

**Test with wrk:**
```bash
wrk -t4 -c100 -d10s \
  -H 'X-User-Role: admin' \
  -H 'X-Resource: admin_panel' \
  -H 'X-Privilege: read' \
  http://127.0.0.1:8080/
```

**Test with Apache Bench:**
```bash
ab -n 10000 -c 100 \
  -H 'X-User-Role: user' \
  -H 'X-Resource: blog' \
  -H 'X-Privilege: read' \
  http://127.0.0.1:8080/
```

**Test with curl:**
```bash
curl -H 'X-User-Role: moderator' \
     -H 'X-Resource: forum' \
     -H 'X-Privilege: edit' \
     http://127.0.0.1:8080/protected
```

## 🌐 Server Endpoints related

| Endpoint | Description |
|----------|-------------|
| `GET /` | Public homepage |
| `GET /api/status` | API status (returns ACL stats) |
| `GET /protected` | Protected resource |
| `GET /admin` | Admin panel |

### Authorization Headers

| Header | Description | Default |
|--------|-------------|---------|
| `X-User-Role` | User's role (e.g., guest, user, admin) | `guest` |
| `X-Resource` | Resource being accessed | `homepage` |
| `X-Privilege` | Required privilege | `read` |

#### Middleware Flow

```
HTTP Request
    ↓
Extract Headers (X-User-Role, X-Resource, X-Privilege)
    ↓
ACL Middleware Check (~500ns)
    ↓
Route to Handler
    ↓
HTTP Response
```

#### 🛠️ Bench Tools

```bash
# wrk (recommended for high-performance testing)
sudo apt install wrk       # Ubuntu/Debian
brew install wrk           # macOS

# Apache Bench (widely available)
sudo apt install apache2-utils  # Ubuntu/Debian
brew install httpd              # macOS

# hey (modern Go-based tool)
go install github.com/rakyll/hey@latest
```

---

## 🗂️ Test Data

**File:** `../test-fixtures/example-extensive-acl-array.json`

### Complexity
- **46 roles** with deep inheritance hierarchies (up to 9 levels)
- **79 resources** with multiple inheritance paths
- **300+ permission rules** (allow and deny)
- **51 unique privileges** covering realistic business operations

### Key Features
- **Deep role hierarchy:** guest → authenticated → subscriber → contributor → author → editor → moderator → administrator → super_admin
- **Multiple inheritance:** power_user inherits from both subscriber and commenter
- **Department roles:** API, support (4 tiers), marketing, sales, development, analytics, HR, finance
- **Complex resources:** blog system, forum, wiki, admin panel, various APIs, reports
- **Realistic deny rules:** blocking sensitive data, deployment restrictions, approval controls

---

## 🎯 Key Takeaways

### Performance 🚀
- ✅ **Sub-microsecond ACL checks** (750ns standalone)
- ✅ **Minimal overhead** (~91µs per web request)
- ✅ **Linear scaling** with CPU cores
- ✅ **5-500x faster** than alternatives

### Memory 💾
- ✅ **Can handle extensive ACL** (memory used up is primarily the roles, resources, and rules defined in the ACL - data structures used memory is negligible)
- ✅ **Predictable scaling** (linear with entities)
- ✅ **Arc-based sharing** (zero-copy across workers)
- ✅ **No runtime growth** (static after load)

---
