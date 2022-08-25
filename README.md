# SWEAT the TOKEN

#### 📦 Dependencies

- Install near-cli: `npm install -g near-cli`
- Install Rust `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- `rustup target add wasm32-unknown-unknown`

#### Build & Run tests

```rust
./sweat/build.sh
//The wasm file will be at `target/wasm32-unknown-unknown/release/sweat.wasm`

cargo test -- --nocapture
cargo run --example mint
cargo run --example transfer
cargo run --example formula
```

## Api

Let's say my account is `sweat_testing_11.testnet`:

```js
near deploy --accountId=sweat_testing_11.testnet --wasmFile=target/wasm32-unknown-unknown/release/sweat.wasm --initArgs '{ "postfix": ".u.sweat.testnet"}' --initFunction new

near call sweat_testing_11.testnet add_oracle '{"account_id":"intmainreturn0.testnet"}' --accountId sweat_testing_11.testnet --gas=2428088695050

near view sweat_testing_11.testnet get_oracles ''
[ 'intmainreturn0.testnet' ]

near view sweat_testing_11.testnet get_steps_from_tge '{ }'
'0'

near view sweat_testing_11.testnet formula '{"steps_from_tge":"1", "steps":1000}'
'999999999912699776'

near view sweat_testing_11.testnet ft_balance_of '{"account_id":"intmainreturn0.testnet"}'
'0'

near call sweat_testing_11.testnet record_batch '{"steps_batch": [["intmainreturn0.testnet", 10000],["poddubny.testnet", 1000] ]}' --accountId intmainreturn0.testnet --gas=300000000000000

near call sweat_testing_11.testnet ft_transfer '{"receiver_id":"poddubny.testnet", "amount":"2", "memo":"hello world!"}' --accountId intmainreturn0.testnet --gas=2428088695050

near call sweat_testing_11.testnet storage_deposit '{"account_id":"sweat_lookup_testing_03.testnet"}' --accountId intmainreturn0.testnet --depositYocto 2350000000000000000000

near call sweat_testing_11.testnet storage_balance_of '{"account_id":"sweat_lookup_testing_03.testnet"}' --accountId intmainreturn0.testnet


near call <token_account> mint_tge '{"amount":"<amount>", "account_for":"<account_for>"}' --accountId <oracle> --gas=300000000000000

near call <token_account> ft_transfer_call '{"receiver_id" : "<lockups_account>", "amount":"<amount>", "msg": "{\"account_id\":\"<account_for>\",\"schedule\":[{\"timestamp\":<tge>,\"balance\":\"0\"},{\"timestamp\":<tge + 6 month in s>,\"balance\":\"<amount>\"}],\"claimed_balance\":\"0\",\"termination_config\": null } " }' --accountId <oracle> --gas=300000000000000 --depositYocto 1


```
