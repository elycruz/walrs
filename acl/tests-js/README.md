# WASM Module Tests

Unit tests for the `walrs_acl` WebAssembly module using Node.js built-in test runner.

## Prerequisites

- Node.js 18+ (for built-in test runner)
- WASM module must be built first (see below)

## Building the WASM Module

Before running tests, you need to build the WASM module:

```bash
# From the acl directory
cd ..
wasm-pack build --target nodejs --no-default-features --features wasm

# Or use web target if you prefer
wasm-pack build --target web --no-default-features --features wasm
```

## Running Tests

```bash
# Run all tests
npm test

# Run with verbose output
npm run test:verbose

# Run in watch mode (re-runs on file changes)
npm run test:watch
```

## Test Structure

- `acl.test.js` - Main test suite covering:
  - `JsAclBuilder` - Constructor, role/resource management, permissions
  - `JsAcl` - Permission checking, inheritance, queries
  - JSON operations - Loading from/to JSON
  - Convenience functions - Quick ACL creation and permission checks
  - Error handling - Invalid input, cycles, missing roles/resources
  - Complex scenarios - Multi-level hierarchies

## Why Tests Are Outside the `pkg` Folder

The `pkg` folder contains a `.gitignore` file with `*`, meaning all files are ignored by git.
When you run `wasm-pack build`, it **completely regenerates** the `pkg` directory, deleting any
custom files you've added.

To avoid losing test files on rebuild:
1. Tests are kept in this separate `tests-js` directory
2. Tests import from `../pkg/walrs_acl.js`
3. The test directory is tracked in git
4. You can run tests after any rebuild without losing them

## Adding New Tests

Add new test cases to `acl.test.js` using the Node.js test API:

```javascript
import { describe, it } from 'node:test';
import assert from 'node:assert/strict';

describe('My Feature', () => {
    it('should do something', () => {
        // Your test code
        assert.equal(actual, expected);
    });
});
```

## Test Fixtures

The tests use JSON fixtures from `../test-fixtures/`:
- `example-acl.json` - Valid ACL configuration
- `invalid-acl.json` - Invalid ACL for error testing
- `example-acl-allow-and-deny-rules.json` - ACL with both allow and deny rules
- `example-extensive-acl-array.json` - Large ACL for performance testing

## Continuous Integration

To run tests in CI/CD:

```bash
# Build WASM module
wasm-pack build --target nodejs --no-default-features --features wasm

# Run tests
cd tests-js
npm test
```

## Debugging Tests

To debug a specific test:

```javascript
// Add only() to run just one test
it.only('should test specific feature', () => {
    // test code
});
```

Or run tests with Node.js inspector:

```bash
node --test --inspect-brk acl.test.js
```

## Node.js Test Runner Features

The built-in test runner supports:
- ✅ Nested test suites with `describe()`
- ✅ Test lifecycle hooks (`before`, `after`, `beforeEach`, `afterEach`)
- ✅ Watch mode for development
- ✅ Test filtering with `it.only()` and `it.skip()`
- ✅ Parallel test execution
- ✅ Multiple reporters (TAP, spec, dot)

No need for external test frameworks like Jest or Mocha!

