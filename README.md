# June Pea E-commerce

A fullstack e-commerce project built entirely with Rust. This monorepo uses Cargo Workspaces to separate different domains, including an Axum-based backend API and a Leptos-based web frontend.

## Technology Stack

- **Backend:** Rust, Axum, SQLx (PostgreSQL), Redis
- **Frontend:** Rust, Leptos (WebAssembly), Trunk
- **Infrastructure:** Docker & Docker-Compose

## Project Structure

This project follows a modular, domain-driven design:

- `crates/api-gateway`: The main backend server exposing RESTful endpoints.
- `crates/frontend`: The WebAssembly frontend application built with Leptos.
- `crates/catalog`: Product catalog service.
- `crates/identify`: User authentication and identity service.
- `crates/inventory`: Inventory and stock management.
- `crates/ordering`: Order processing, cart, and checkout.
- `crates/marketing`: Marketing, promotions, and discounts.
- `crates/shared`: Common utilities, structs, and error handling for all crates.
- `migrations`: SQLx database migrations for the PostgreSQL database.

## Testing

See [docs/testing.md](docs/testing.md) for the test layers and how to run them.

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Docker](https://docs.docker.com/get-docker/) & Docker Compose
- [Trunk](https://trunkrs.dev/): `cargo install trunk`
- [SQLx CLI](https://crates.io/crates/sqlx-cli): `cargo install sqlx-cli --no-default-features --features rustls,postgres`

## Getting Started

### 1. Start Infrastructure (Database & Cache)

The project relies on PostgreSQL and Redis. Start them using Docker Compose:

```bash
docker-compose up -d postgres redis
```

### 2. Configure Environment and Database

Create a `.env` file in the root directory and ensure you have the `DATABASE_URL` configured. Then, run the migrations:

```bash
# Create the database and run all migrations in the /migrations folder
sqlx database create
sqlx migrate run
```

### 3. Run the Backend API

Start the Axum API Gateway from the root of the workspace. This will serve the backend on `localhost:8080`:

```bash
cargo run --bin api-gateway
```

### 4. Run the Web Frontend

In a new terminal window, use Trunk to serve the Leptos frontend. It will compile your Rust code to WebAssembly and auto-reload on changes:

```bash
cd crates/frontend
trunk serve --watch .
```

Then open your browser to view the application!