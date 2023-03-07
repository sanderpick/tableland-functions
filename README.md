# tableland-functions

[![Test](https://github.com/sanderpick/tableland-functions/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/sanderpick/tableland-functions/actions/workflows/test.yml)
[![License](https://img.shields.io/github/license/sanderpick/tableland-functions.svg)](./LICENSE)
[![standard-readme compliant](https://img.shields.io/badge/standard--readme-OK-green.svg)](https://github.com/RichardLitt/standard-readme)

> An experimental Rust SDK and runtime for Tableland Edge Functions

# Table of Contents

- [tableland-functions](#tableland-functions)
  - [Table of Contents](#table-of-contents)
  - [Background](#background)
    - [POC design](#poc-design)
    - [Why?](#why)
    - [Tableland Functions](#tableland-functions)
    - [Requirements](#requirements)
    - [Edge functions vs. serverless functions](#edge-functions-vs-serverless-functions)
    - [Comparison to smart contracts](#comparison-to-smart-contracts)
  - [Usage](#usage)
    - [Function development](#function-development)
      - [Example JSON API](#example-json-api)
    - [Run the Worker](#run-the-worker)
  - [Development](#development)
  - [Contributing](#contributing)
  - [License](#license)

# Background

`tableland-functions` is an experimental Rust SDK and [Wasmer](https://wasmer.io/) runtime for edge function handling on the Tableland network. The architecture is based on [cosmwasm](https://github.com/CosmWasm/cosmwasm) and the guest function interface is inspired by Cloudflares's [workers-rs](https://github.com/cloudflare/workers-rs). 

This repo contains the following crates:
- [`tableland_std`](/lib/std)`: Wasmer compiler, import and export definitions, and type bindings for guest function development. 
- [`tableland_derive`](/lib/std): Macros for guest function development in Rust.
- [`tableland_vm`](/lib/std): Wasmer host environment and imports API. The current import API provides a function request context with a `read` method for executing Tableland read-only queries.
- [`tableland_worker`](/lib/std): HTTP server and Wasmer instance cache. The Worker reponds to [`evm-tableland`](https://github.com/tablelandnetwork/evm-tableland) events forwarded by a validator, which trigger an instantiation of a WASM binary from IPFS. WASM binaries are compiled, cached, and made available over the Worker's `/v1/functions/{wasm_cid}` endpoint. See below for a diagram of how this works.
- [`tableland_client`](/lib/std): A (currently) read-only Tableland client.

## POC design

The POC uses IPFS to make the WASM binaries available to validators. However, it is possible to consider using a more resilient layer, such as Filecoin.

![tableland-functions](https://user-images.githubusercontent.com/361000/223291893-952296a5-7dff-4005-8c20-b0e0c4d7036a.png)

## Why?

The Tableland network aims to provide developers with a secure, deterministic, and cloud-like database experience. However, it currently lacks an important component for developers: where to deploy applications or APIs driven by Tableland data. Currently, the available options are limited:

- IPFS for client-side apps: Requires pinning your application with a pinning service, only works if your application can be built client-side, and does not allow application changes.
- Use a hosted service like Cloudflare, Supabase, Vercel, etc.: These are not decentralized or deterministic.

The goal of this experiment is to determine whether Tableland is a suitable location for providing a practical solution. This solution should:

- Enable developers to deploy backends that can query Tableland.
- Enable users to execute backends from any participating validator.
- Be scalable or have the potential to scale.
- Allow validators to measure the amount of work involved in executing queries (this is already necessary for simple read queries).

## Tableland Functions

This repository presents a potential solution to the problem described above by adopting the notion of an "edge function". In the context of Tableland, an edge function would provide the following benefits:

- Reduced latency: By executing functions, which may contain many queries, next to a validator's gateway, the time it takes to process requests can be reduced.
- Deploy your whole application without a server: Since Tableland is already a cloud-like database, serverless functions would allow developers to skip deploying their own backend, and instead build JSON APIs, render HTML, or even generate SVGs based on Tableland data.
- Respond with custom HTTP headers.
- Conditional authorization through signed requests (currently not implemented).

It comes as no surprise that many Database Software as a Service (DB SaaS) offerings have added edge functions that run next to the database.

### Requirements

- Functions should be deterministic. The output should always be the same for a given Tableland network state. This means no float types or external network access.
- Function execution should be quantifiable. The amount of work required to execute a function should be quantifiable in some unit for a given Tableland network state. Luckily, Wasmer provides a metering middleware that is used to calculate function “gas”. `tableland-functions` also has a notion of “external gas”, which is currently based on query statement and response size.
- Cold start for functions should be fast. Currently, it takes ~2 seconds, but there is plenty of room for optimization.
- Functions should execute quickly, and Wasmer is a very fast option. Currently, most of the latency is due to the validator. The example JSON API responds locally in approximately 5-10 milliseconds. This is actually the metric we care about because `tableland-functions` is intended to be localized with validators.
- WASM binaries should be relatively small. For example, the JSON API provided here builds to around 180KB. However, you can reduce this to less than 50KB by using custom HTTP types across the WASM bridge, and by using a more constrained JSON serialization library such as [serde-json-wasm](https://github.com/CosmWasm/serde-json-wasm). Additionally, there is ample opportunity for further optimization, such as compressing the binaries using a tool like [UPX](https://github.com/upx/upx).

### Edge functions vs. serverless functions

Edge functions are serverless functions that run on the edge, close to your users. Typically, they are part of a Content Delivery Network (CDN) such as Cloudflare, Netlify, Supabase, or Vercel.

In Tableland, database query requests are delivered by validators that may be distributed across the globe. Currently, we do not have latency-based routing for read queries. Once we implement this feature, the entire network will resemble a content delivery network (CDN), although without dynamic content distribution. However, this functionality could be added in the future. Therefore, something like `tableland-functions` can only be considered edge functions in this future context. For now, they are simply serverless functions.

### Comparison to smart contracts

In Tableland, developer-deployed smart contracts allow for on-chain actions that can write Tableland data and for Tableland data to drive on-chain actions via inclusion proofs.

However, smart contracts are not a solution to the problem at hand because they cannot readily respond to HTTP requests. Even if they could, they would be unable to query off-chain data, such as that in Tableland.

# Usage

## Function development
Currenlty, Rust is the only language you can use to write functions. Future experiments may include [AssemblyScript](https://www.assemblyscript.org/) support or JS/TS support using [QuickJS](https://bellard.org/quickjs/) (probably via [Javy](https://github.com/Shopify/javy)).

### Example JSON API

```rust
#[entry_point]
pub fn fetch(req: Request, ctx: CtxMut) -> Result<Response> {
    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::default();

    // Add as many routes as your Function needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to get route or query parameters.
    router
        .get("/", |_, _, _| Response::ok("Hello from Tableland!"))
        .get("/version", |_, _, _| Response::ok(VERSION))
        .get("/:type", |_, ctx, rctx| {
            if let Some(t) = rctx.param("type") {
                let data = ctx.tableland.read(
                    format!("select * from pets_31337_4 as pets join homes_31337_2 as homes on pets.owner_name = homes.owner_name where type = '{}';", t).as_str(),
                    ReadOptions::default(),
                )?;
                return Response::from_json(&data);
            }
            Response::error("Bad Request", 400)
        })
        .run(req, ctx)
}
```

This function returns an HTTP response with a JSON payload and headers describing the work performed by the Worker:

```
curl -v http://localhost:3030/v1/functions/bafkreia4c7orjt23vorxg65vm7b34xvenkgxigsgnmziuhcqp3hi2p5bbi/bird
*   Trying 127.0.0.1:3030...
* Connected to localhost (127.0.0.1) port 3030 (#0)
> GET /v1/functions/bafkreia4c7orjt23vorxg65vm7b34xvenkgxigsgnmziuhcqp3hi2p5bbi/bird HTTP/1.1
> Host: localhost:3030
> User-Agent: curl/7.85.0
> Accept: */*
>
* Mark bundle as not supporting multiuse
< HTTP/1.1 200 OK
< content-type: application/json
< x-gas-limit: 500000000000000
< x-gas-remaining: 499707906266700
< x-gas-external: 533300
< x-gas-internal: 292093200000
< content-length: 167
< date: Mon, 06 Mar 2023 23:35:49 GMT
<
* Connection #0 to host localhost left intact
[{"area":"country","name":"Harambe","owner_name":"Dani","type":"bird","value":67000},{"area":"urban","name":"Hodor","owner_name":"Eliza","type":"bird","value":210000}]
```

See [`examples`](/examples) for more, including HTML and SVG rendering.

The example tests use mock data. To run the examples in a real Worker, you will need to seed a local `go-validator` with data:

```bash
cd examples/data
./generate.sh
```

Target `wasm32-unknown-unknown` to build each example:

```bash
cd examples/json
cargo build --target wasm32-unknown-unknown --release

# or build a smaller binary (requires nightly toolchain)
cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target wasm32-unknown-unknown --release
```

See [here](https://github.com/johnthagen/min-sized-rust) for a guide on minimizing the size of binaries. Note, not all recommendations apply to `wasm32-unknown-unknown`.

While not currently very useful, functions can also respond to POST requests with payloads. This could be helpful in triggering Tableland writes in conjunction with conditional authentication and ERC-4337 account abstraction for gasless transactions.

## Run the Worker

```bash
cargo run -p tableland_worker
```

A config file will be written to the expected location for your system. See [here](https://crates.io/crates/directories) for details (specifically, `ProjectDirs::config_dir`). Below is the default Worker config:

```toml
[server]
host = '127.0.0.1'
port = '3030'

[chain]
id = 'Local'

[cache]
directory = '.'

[ipfs]
gateway = 'http://localhost:8081/ipfs'
```

# Development

You will need a local [`go-tableland`](https://github.com/tablelandnetwork/go-tableland) validator and a local EVM node running the `TablelandTables` contract from [`evm-tableland`](https://github.com/tablelandnetwork/evm-tableland). The easiest way to do this is with [local-tableland](https://github.com/tablelandnetwork/local-tableland). However, `tableland-functions` requires specific branches of `go-tableland` and `evm-tableland` (see [here](https://github.com/tablelandnetwork/go-tableland/compare/main...sander/functions) and [here](https://github.com/tablelandnetwork/evm-tableland/compare/main...sander/functions). You may find it easier to spin the components manually.

# Contributing

PRs accepted.

Small note: If editing the README, please conform to the
[standard-readme](https://github.com/RichardLitt/standard-readme) specification.

# License

MIT AND Apache-2.0, © 2021-2022 Tableland Network Contributors
