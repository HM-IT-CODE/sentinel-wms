# SentinelWMS 🛡️📦

**Agentic Warehouse Intelligence** — an AI agent (Google Gemini + Google Cloud Agent Builder) that doesn't just answer questions, it **takes action** on a real warehouse-management workload. SentinelWMS gives the agent its "superpowers" through a **Fivetran** data pipeline exposed over the **Model Context Protocol (MCP)**.

> Built for the **Google Cloud Rapid Agent Hackathon** — Fivetran partner track.

---

## What it does

SentinelWMS is an MCP server written in **Rust** that lets a Gemini-powered agent:

- 🔎 **Auto-discover** the warehouse database schema (tables, columns, keys) and reason over it in English.
- 📊 **Query live inventory** — stock by warehouse, product locations, kardex movements, dispatches and invoices.
- 🔁 **Monitor & manage the Fivetran data pipeline** — check connector status and trigger syncs through the Fivetran REST API.
- 🤖 **Plan multi-step missions** (e.g. *"which products are below minimum stock in the Main DC and need reordering?"*) and execute them under human oversight.

## Architecture

```
            ┌──────────────────────┐
            │  Gemini / Agent       │
            │  Builder (Google Cloud)│
            └──────────┬────────────┘
                       │  MCP (JSON-RPC, STDIO / SSE)
                       ▼
            ┌──────────────────────┐
            │   SentinelWMS (Rust)  │
            │   - MCP server        │
            │   - WMS tools         │
            │   - Fivetran client   │
            └───────┬───────┬───────┘
                    │       │
        Tiberius    │       │  reqwest (REST)
        (SQL Server)│       │
                    ▼       ▼
        ┌───────────────┐  ┌──────────────────┐
        │ SQL Server    │  │ Fivetran (MCP /   │
        │ WMS data      │  │ REST API)         │
        └───────────────┘  └──────────────────┘
```

### Modules

| Path | Responsibility |
|------|----------------|
| `src/mcp/` | MCP protocol: JSON-RPC types, router, STDIO/SSE transports |
| `src/tools/` | Agent tools (inventory queries, Fivetran status/sync) |
| `src/fivetran/` | Async Fivetran REST client + models |
| `src/db/` | SQL Server pool (Tiberius/bb8) + schema introspection |
| `src/config/` | Environment loading |
| `schema/` | Bilingual data dictionary + synthetic demo database scripts |

## The demo dataset

To keep the demo **safe and reproducible** (no real customer data), this repo ships scripts that build a fully **synthetic** warehouse database, `WMS_SENTINEL_DEMO`, with generic English names and realistic volume:

| Table | Rows |
|-------|-----:|
| `products` | 5,000 |
| `customers` | 50,000 |
| `stock_by_warehouse` | 40,000 |
| `invoices` | 200,000 |
| `invoice_lines` | **2,000,000** |

See [`schema/create_wms_demo.sql`](schema/create_wms_demo.sql) and [`schema/seed_invoice_lines.sql`](schema/seed_invoice_lines.sql).
The ES→EN data dictionary the agent uses lives in [`schema/data_dictionary.json`](schema/data_dictionary.json).

## Getting started

### Prerequisites
- Rust (stable, edition 2021)
- SQL Server 2019+ (the demo DB)
- A Fivetran account + System Key

### 1. Configure
```bash
cp .env.example .env
# edit .env with your DATABASE_URL and Fivetran credentials
```

### 2. Build
```bash
cargo build --release
```

### 3. Run the MCP server
```bash
# STDIO transport (for Agent Builder / MCP clients)
cargo run -- --transport stdio

# or SSE (web) transport
cargo run -- --transport sse
```

### 4. Quick Fivetran integration check
```bash
cargo run -- --test-fivetran
```

## MCP tools exposed

| Tool | Description |
|------|-------------|
| `ObtenerEsquemaBaseDatos` | Auto-discover the full database schema |
| `EstadoCuentaCliente` | Look up a customer account |
| `EjecutarSQLDinamico` | Run a reasoned, dynamic SQL query |
| `FivetranConnectorStatus` | Get a Fivetran connector's status |
| `FivetranForceSync` | Force an immediate Fivetran sync |

## License

[MIT](LICENSE) © 2026 SentinelWMS
