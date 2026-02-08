# ACL Benchmarks

Benchmarks demonstrating ACL performance in both standalone and web application contexts.

---

## 🚀 Benchmarks

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

#### Results

**Summary:**

```
Performance: 1.3M checks/sec @ ~750ns per check
Memory:      ~50 KB for 46 roles + 79 resources + 300+ rules
Loading:     ~7.5ms including cycle detection
```

**Detailed results:**

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

---

### 2. Web Server Benchmark

HTTP server example, using actix, with ACL middleware for simulating thread safety, and load testing.

#### Run

```bash
cargo run --release --example benchmark_actix_middleware
```

**Provides:**
- HTTP server on `http://127.0.0.1:8080`
- ACL middleware on every request
- Multiple test endpoints
- Header-based authorization
- Ready for external benchmarking tools

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

#### Results

| Metric | Expected Range | Notes |
|--------|----------------|-------|
| **HTTP Throughput** | 50,000+ req/sec | On 4 cores |
| **ACL Check Time** | 400-750ns | Per request |
| **Total Overhead** | <1µs | ACL middleware impact |
| **Latency Impact** | <0.01% | Of total request time |
| **Memory** | ~50 KB | Shared across workers |

**Comparison with Alternatives:**

| Approach | Latency | Speedup vs ACL |
|----------|---------|----------------|
| **In-memory ACL** | ~500ns | 1x (baseline) |
| Redis cache | ~100µs | **200x slower** |
| Database query | ~2ms | **4,000x slower** |
| External auth service | ~20ms | **40,000x slower** |

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

### Memory Layout

- **AclData (JSON parsed):** ~27 KB
  - 46 roles with inheritance chains
  - 79 resources with hierarchies
  - 300+ permission rules
- **Compiled ACL:** ~23 KB
  - Role graph (DAG): ~3 KB
  - Resource graph (DAG): ~5 KB
  - Rules (nested HashMaps): ~15 KB

**Total: ~50 KB** (static after load, no runtime growth)

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
