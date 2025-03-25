# Trading Data API

## Overview

This is a Rust-based server application that provides a simple HTTP API for managing and analyzing financial trading data. The API allows bulk insertion of trading prices and provides statistical analyses on recent trading data.

## API Endpoints

### 1. Add Batch Data

**Endpoint:** `POST /add_batch/`

**Purpose:** Allows the bulk addition of consecutive trading data points for a specific symbol.

**Request Body:**

```json
{
  "symbol": "AAPL",
  "values": [150.1, 151.2, 149.8, 152.3]
}
```

**Response:**

```json
{
  "message": "Batch added successfully"
}
```

---

### 2. Retrieve Statistical Analysis

**Endpoint:** `GET /stats/`

**Purpose:** Provides rapid statistical analyses of recent trading data for a specified symbol.

**Query Parameters:**

- `symbol`: The financial instrument's identifier (e.g., "AAPL").
- `k`: An integer from 1 to 8, specifying the number of last `1e{k}` data points to analyze.

**Example Request:**

```
GET /stats/?symbol=AAPL&k=3
```

**Example Response:**

```json
{
  "min": 149.8,
  "max": 152.3,
  "last": 152.3,
  "avg": 150.85,
  "var": 1.02
}
```

---

## Installation Guide

### Install Rust

Ensure you have Rust installed. You can install Rust using [rustup](https://rustup.rs/).


### Build the Project

```sh
cargo build --release
```

### Run the Server

```sh
cargo run
```

The server will start and listen on a port `8080` - `http://localhost:8080`.

---

## Running Tests

To ensure everything is working correctly, run the test suite using Cargo:

```sh
cargo test
```

The integration test may be flaky. It runs `cargo run` and waits a while for the server to start. If the compilation is delayed for any reason, the test may fail when attempting to create the connection. In such cases, try again or build the binary beforehand by running `cargo build` before executing the tests.

---

## Dependencies

This project uses the following Rust libraries:

- `actix-web` for HTTP server functionality.
- `serde` and `serde_json` for JSON serialization.
- `tokio` for asynchronous runtime.

For detailed dependencies, check `Cargo.toml`.

---

## License

This project is licensed under the MIT License.

---

## Author

Tomasz Kulik - [[tomek.kulik2@gmail.com](mailto\:tomek.kulik2@gmail.com)]

