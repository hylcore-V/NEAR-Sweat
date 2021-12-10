# 🚀 SWT Near playground

#### 📦 Dependencies

- Install near-cli: `npm install -g near-cli`
- Install Rust `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

#### 🧪 Run tests

```shell
cargo test -- --nocapture
```

Or use VS Code + Rust plugin

<img width="200" alt="image" src="https://user-images.githubusercontent.com/1473995/145069302-168e6aa9-e065-4ede-a643-2616faaac298.png">

#### 🏗 Build

```shell
cargo build --target wasm32-unknown-unknown --release
```

The wasm file will be at `target/wasm32-unknown-unknown/release/swt.wasm`

#### 🚀 Deploy

🎫 Specify your own accountId.

```shell
### deploy to your own account
near deploy --accountId=intmainreturn0.testnet --wasmFile=target/wasm32-unknown-unknown/release/swt.wasm --initArgs '{"oracle_id":"intmainreturn0.testnet"}' --initFunction new

### OR use dev-deploy
near dev-deploy --wasmFile=target/wasm32-unknown-unknown/release/swt.wasm

### OR create & deploy to sub-account
near create-account swt05.intmainreturn0.testnet --masterAccount intmainreturn0.testnet --initialBalance 100
near call intmainreturn0.testnet storage_deposit '' --accountId swt05.intmainreturn0.testnet --amount 99
near deploy --accountId=swt05.intmainreturn0.testnet --wasmFile=target/wasm32-unknown-unknown/release/swt.wasm --initArgs '{"oracle_id":"intmainreturn0.testnet"}' --initFunction new

https://wallet.testnet.near.org/profile/swt05.intmainreturn0.testnet
https://explorer.testnet.near.org/transactions/Bww2kj8C9tJFdwnB2urWCebBASQgt6Uq8FUuxKGGpRsj

```

- To register - https://wallet.testnet.near.org
- To check - run `near login`

- About dev-deploy (https://stackoverflow.com/a/69538060/1655153)
- https://docs.near.org/docs/tools/near-cli#near-dev-deploy

#### 🎇 Mint, Transfer, Balance...

```shell
⨐t
ntmainreturn0.testnet get_steps_from_tge '{}'
0
near view swt05.intmainreturn0.testnet formula '{"steps": 10000}'
10

#### mint to myself 🚶‍
near view swt05.intmainreturn0.testnet ft_balance_of '{"account_id":"intmainreturn0.testnet"}'
'0'

near call swt05.intmainreturn0.testnet record '{"account_id":"intmainreturn0.testnet", "steps": 10000}' --accountId intmainreturn0.testnet --gas=300000000000000
https://explorer.testnet.near.org/transactions/8uC2uwb7jgR3WD3iPmybntwc6f2qHXdH2qWzbENA3w9c

near view swt05.intmainreturn0.testnet ft_balance_of '{"account_id":"intmainreturn0.testnet"}'
'10'

owls-MacBook-Pro:swt-near owl$ near view swt05.intmainreturn0.testnet get_steps_from_tge '{}'
10000

#### mint to oleg 🚶‍
near view swt05.intmainreturn0.testnet ft_balance_of '{"account_id":"poddubny.testnet"}'
'0'

near call swt05.intmainreturn0.testnet record '{"account_id":"poddubny.testnet", "steps": 10000}' --accountId intmainreturn0.testnet --gas=300000000000000
https://explorer.testnet.near.org/transactions/6bSYEmncYe7Ln5TPoWADbg51X2Rqtr9TrhhsK6jnu3mP

near view swt05.intmainreturn0.testnet ft_balance_of '{"account_id":"poddubny.testnet"}'
'10'

#### mint to sub-account 🚶‍
near view swt05.intmainreturn0.testnet ft_balance_of '{"account_id":"swt05.intmainreturn0.testnet"}'
'0'

near call swt05.intmainreturn0.testnet record '{"account_id":"swt05.intmainreturn0.testnet", "steps": 10000}' --accountId intmainreturn0.testnet --gas=300000000000000
https://explorer.testnet.near.org/transactions/6bSYEmncYe7Ln5TPoWADbg51X2Rqtr9TrhhsK6jnu3mP
https://explorer.testnet.near.org/transactions/Ajpo5uHqvmXcEZL541u3rm66FNmqy1uRyEb22PHQa2eq

near view swt05.intmainreturn0.testnet ft_balance_of '{"account_id":"swt05.intmainreturn0.testnet"}'
'10'

### transfer swt05.intmainreturn0.testnet -> poddubny.testnet 5 swt 💸
near call swt05.intmainreturn0.testnet ft_transfer '{"receiver_id":"poddubny.testnet", "amount":"5"}' --accountId swt05.intmainreturn0.testnet --gas=2428088695050 --depositYocto 1
https://explorer.testnet.near.org/transactions/EiWcyUGcG4DnGgFLRrMwWwHpB7GBqLX72i7s8oKV5vk5

near view swt05.intmainreturn0.testnet ft_balance_of '{"account_id":"poddubny.testnet"}'
'15'


### transfer with memo text 💸💬
near call swt05.intmainreturn0.testnet ft_transfer '{"receiver_id":"poddubny.testnet", "amount":"1", "memo":"hello world!"}' --accountId swt05.intmainreturn0.testnet --gas=2428088695050 --depositYocto 1
https://explorer.testnet.near.org/transactions/5w1B5S5T3jsUB7ohLpgrWH2JGmH2LV9K8bT8smmsVVkv
near view swt05.intmainreturn0.testnet ft_balance_of '{"account_id":"poddubny.testnet"}'
'16'

### record_batch 🚶‍🚶‍
near call swt05.intmainreturn0.testnet record_batch '{"steps_batch": [["intmainreturn0.testnet", 1000],["poddubny.testnet", 1000] ]}' --accountId intmainreturn0.testnet --gas=300000000000000
https://explorer.testnet.near.org/transactions/BGm57cQre8t125pj3crASgfkgY4EtVdyKLKCYstVjk7X

near view swt05.intmainreturn0.testnet ft_balance_of '{"account_id":"poddubny.testnet"}'
'17'
near view swt05.intmainreturn0.testnet ft_balance_of '{"account_id":"intmainreturn0.testnet"}'
'11'
```

#### 🚶‍♀️Send steps

```shell
near call intmainreturn0.testnet record '{"account_id":"intmainreturn0.testnet", "steps": 1000}' --accountId intmainreturn0.testnet --gas=300000000000000
```

#### 🚶‍🚶‍♀️Send batch of stepss

```shell
near call swt05.intmainreturn0.testnet record_batch '{"steps_batch": [["intmainreturn0.testnet", 1000],["poddubny.testnet", 1000] ]}' --accountId intmainreturn0.testnet --gas=300000000000000
```

#### ⨐t Formula

```shell
near view intmainreturn0.testnet get_steps_from_tge '{ }'
```

#### #️⃣ Balances

```shell
near view intmainreturn0.testnet ft_balance_of '{"account_id":"intmainreturn0.testnet"}'
```
