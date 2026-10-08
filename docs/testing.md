# Testing

Each crate under `crates/` is a service in the modular monolith, and each is tested the same
way: most tests at the bottom (fast, cheap), a few at the top (slow, but they prove the pieces
fit). The layout follows [testing-in-microservices](https://github.com/buixuankhai1204/testing-in-microservices)
minus Kafka. `payment` is the fullest example, read it first.

## The layers

| Type | Where | Real | Faked |
|---|---|---|---|
| Unit | `src/` (`#[cfg(test)]`) | the code under test | its collaborators (mockall, or the in-memory repo) |
| Integration, database | `crates/*/tests/integration_*.rs` | our repository and a real Postgres | nothing else |
| Integration, other service over HTTP | `crates/payment/tests/integration_vnpay_gateway.rs` | our reqwest client and real HTTP | VNPay (wiremock) |
| Component, in-process | `crates/*/tests/component_in_process.rs` | routes, JSON, use cases, real sockets | the database (in-memory repos), other services |
| Contract | `crates/payment/tests/contract_ordering.rs` | the other service's actual use cases | its database |
| Component, out-of-process | `crates/api-gateway/tests/component_out_of_process.rs` | the compiled binary, Postgres, Redis | VNPay (wiremock) |
| End-to-end | `crates/api-gateway/tests/e2e.rs` | a deployed environment | nothing |

Edge cases go in unit and component tests. End-to-end only covers the main journeys.

## Two kinds of "another service"

* **A service in this monolith** (payment calls ordering). The caller owns a small port
  (`payment::domain::OrderLookup`), an adapter in `infrastructure/` translates to the other
  crate, and tests fake the port. The contract test runs the adapter against the real other
  service so a change on that side breaks a test here.
* **A real external service** (VNPay). The caller owns a port too (`PaymentGateway`), the
  adapter uses `reqwest`, and tests run it against a wiremock server. The fake lives in
  `crates/test-support/src/vnpay.rs`. It is written from VNPay's docs and shares no code with
  the service, and like VNPay it answers `97` to a badly signed request.

Anything that can fail on the wire gets a test: HTTP errors, timeouts, garbage bodies and
forged signatures each have one in `integration_vnpay_gateway.rs`.

## Running

```bash
cargo test --workspace --exclude frontend      # unit, in-process component, contract, VNPay/wiremock
cargo test --workspace --exclude frontend -- --ignored   # the ones that need Docker or a database
```

* Existing `integration_test.rs` / `vnpay_flow.rs` files use `#[sqlx::test]` and need
  `DATABASE_URL` pointing at a Postgres server (`docker compose up -d postgres`). They are not
  ignored.
* `api-gateway` needs a migrated database **at compile time** (`analytics` uses
  `sqlx::query!`). Keep `DATABASE_URL` set when building it, or run `cargo sqlx prepare
  --workspace` and commit `.sqlx` to build offline.
* The out-of-process tests start Postgres 16 and Redis 7 with Testcontainers (needs Docker).
  Set `TEST_DATABASE_URL` / `TEST_REDIS_URL` to use ones that are already running; use a
  throwaway database, the gateway migrates it.
* End-to-end, against a deployed environment:

  ```bash
  E2E_BASE_URL=https://staging.example.com cargo test -p api-gateway --test e2e -- --ignored
  ```

## Adding a test to a service

1. Rules and decisions: unit test next to the code.
2. A new route or error mapping: a case in `tests/component_in_process.rs` using the crate's
   `InMemory…Repository` and `shared::testing::NoopUnitOfWork`.
3. New SQL: a case in the Postgres integration file.
4. A new call to another service: a port, an adapter, a contract test (monolith) or a wiremock
   integration test (external), and one component test that shows what the caller does when
   it fails.
