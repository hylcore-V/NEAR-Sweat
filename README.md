# 🚀 SWT Near playground

#### Dependencies

- Install near-cli: `npm install -g near-cli`
- Install Rust `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

#### Run tests

```shell
cargo test -- --nocapture
```

Or use VS Code + Rust plugin

<img width="286" alt="image" src="https://user-images.githubusercontent.com/1473995/145069302-168e6aa9-e065-4ede-a643-2616faaac298.png">

#### Build

```shell
cargo build --target wasm32-unknown-unknown --release
```

The wasm file will be at `target/wasm32-unknown-unknown/release/swt.wasm`

#### Deploy

🎫 Specify your own accountId.

```shell
near deploy --accountId=intmainreturn0.testnet --wasmFile=target/wasm32-unknown-unknown/release/swt.wasm --initArgs '{"oracle_id":"intmainreturn0.testnet"}' --initFunction new

OR

near dev-deploy --wasmFile=target/wasm32-unknown-unknown/release/swt.wasm

```

- To register - https://wallet.testnet.near.org
- To check - run `near login`
- [About dev-deploy](https://stackoverflow.com/a/69538060/1655153)

#### Send steps

```shell
near call intmainreturn0.testnet record '{"account_id":"intmainreturn0.testnet", "steps": 1000}' --accountId intmainreturn0.testnet --gas=300000000000000
```

Record the batch of steps:

```shell
near call <contract name> batch_record '{"steps": [["testmewell.testnet", 1000000]]}' --accountId <your oracle account>
```

#### Formula ⨐t

```shell
near view intmainreturn0.testnet formula '{"account_id":"intmainreturn0.testnet", "steps": 10000}'
near view intmainreturn0.testnet get_steps_from_tge '{"account_id":"intmainreturn0.testnet"}'
```

#### Balances

```shell
near view intmainreturn0.testnet ft_balance_of '{"account_id":"intmainreturn0.testnet"}'
```
