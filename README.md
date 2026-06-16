# phat_squid_demo: SQD Network indexer in a Phala Phat Contract

A demo Phala Network [Phat Contract](https://docs.phala.network/) (ink!) that reads historical EVM data from [SQD Network](https://sqd.dev/network) and writes the results to S3-compatible storage. It shows how to run an indexer as offchain logic inside a Phala Worker instead of a standalone service.

This is example code maintained by SQD ([sqd.dev](https://sqd.dev)). It is meant as a starting point to copy and adapt, not a production indexer.

## What it does

The contract (`indexer_phat/contracts/http_client`) is an ink! 4.2 contract built on the [`pink-extension`](https://use.ink/), which lets a Phat Contract perform HTTP requests from inside a Phala Worker.

Its `start_indexer(start, end)` message loops over a block range and, for each block:

1. Requests a SQD Network worker URL for the `ethereum-mainnet` dataset from the SQD Network gateway (`https://v2.archive.subsquid.io/network/ethereum-mainnet`).
2. Sends an EVM query to that worker for ERC-20 `Transfer` logs of the USDC contract (`0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48`).
3. Stores the raw response for that block in an S3-compatible bucket using the [`pink-s3`](https://crates.io/crates/pink-s3) library, keyed as `block-<number>`.
4. Updates `last_indexed_block`.

`get_last_indexed_block()` returns the last block number the contract processed.

The query and the storage call are hardcoded in `contracts/http_client/src/lib.rs`. Change the query body to index different logs, transactions, or fields (see the [SQD Network EVM API reference](https://docs.sqd.dev/subsquid-network/reference/evm-api/) and the [list of supported networks](https://docs.sqd.dev/subsquid-network/reference/networks/)), and edit the `store_s3` function to point at your own storage endpoint and credentials. The endpoint, bucket, and keys in `store_s3` are placeholders and must be filled in before deployment.

## Repository layout

```
indexer_phat/
  contracts/http_client/   ink! contract source (Cargo project)
  tests/http_client/        TypeScript integration test (devphase + @phala/sdk)
  stacks/                   local Phala dev stack binaries
  devphase.config.json      devphase configuration (local, poc5, poc6 networks)
  package.json              Node dependencies (@devphase/service, typescript)
```

## Build

The contract is built with `cargo contract`. Set up the Rust and `cargo-contract` toolchain following the [cargo-contract installation guide](https://github.com/paritytech/cargo-contract#installation).

```bash
cd indexer_phat/contracts/http_client
cargo contract build
```

## Test

Rust unit test (mocks the Phala HTTP extension locally):

```bash
cd indexer_phat/contracts/http_client
cargo test -- --nocapture
```

The repository also includes a TypeScript integration test under `indexer_phat/tests`, run through [devphase](https://github.com/l00k/devphase) with the local stack defined in `devphase.config.json`. Install the Node dependencies first:

```bash
cd indexer_phat
yarn install
```

## Deploy

Deploy the compiled `.contract` artifact to Phala Network using the Phala tooling and UI. See the [Phala Network documentation](https://docs.phala.network/) for current deployment instructions. The `devphase.config.json` in this repo includes endpoints for a local stack and the `poc5` and `poc6` test networks.

After deployment, the contract exposes:

- `new()`: initializes the contract.
- `start_indexer(start: i32, end: i32)`: indexes blocks from `start` to `end`.
- `get_last_indexed_block()`: returns the last indexed block number.

## Notes and limitations

- If a request to SQD Network fails for a block, the contract returns an error for that call. Retry logic is not implemented.
- The S3 credentials and endpoint in `store_s3` are placeholders. They must be replaced with real values before the contract can store data.

## Documentation

- SQD Network: https://docs.sqd.dev/subsquid-network/overview/
- SQD Network EVM API reference: https://docs.sqd.dev/subsquid-network/reference/evm-api/
- SQD docs home: https://docs.sqd.dev
- Phala Network docs: https://docs.phala.network/

## License

MIT (see `indexer_phat/package.json`).
