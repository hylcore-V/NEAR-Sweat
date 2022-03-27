# SWEAT the TOKEN

#### 📦 Dependencies

- Install near-cli: `npm install -g near-cli`
- Install Rust `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- `rustup target add wasm32-unknown-unknown`

#### 🏗 Test, Build, Deploy 🚀

```shell
### run unit tests
cargo test -- --nocapture
### add target
rustup target add wasm32-unknown-unknown
## compile to wasm
./sweat/build.sh
```

The wasm file will be at `target/wasm32-unknown-unknown/release/sweat.wasm`

Let's say my account is `sweat_01_testing.testnet`:

```rust
### login to near wallet -> choose sweat_01_testing.testnet
near login

### deploy
near deploy --accountId=sweat_01_testing.testnet --wasmFile=target/wasm32-unknown-unknown/release/sweat.wasm --initArgs '{"oracles_vec": ["intmainreturn0.testnet", "swt07.intmainreturn00.testnet", "f86edd60952fee3ed6413a2a95782a49823e24645dce27c08c7db92", "b18c66baba05d37dceaeaba743ca20b7ca077ff51861d69484939f80", "poddubny.testnet", "f86edd60952fee3ed6413a2a95782a49823e24645dce27c08c7db9206", "0952227bc4149978a3e57bbae26e9ae8c0d7d9e0fb267a2a179cafb2", "2b16781c2e614e0e48fba532d591e3d293cb3cf6bbb601a8a1a2e9bd", "16b5d44ce3e24652291bedefc484b2ae9f414c41963b54556b013aa5", "dev-1638368516362-28263900316041", "e09644746eebdd4752a17bed1d632c395720f65a46e31d1d260e77a5", "1fbbd38dca41fdcfe06dc41bec0a86fa7b67f4f8695f292b07664062", "e7c81b495a2e703507823169346f55c76681c887937b9d5ff2b70577", "9e1a12ca8cd0d161a5f097d3e8b279f4e9389a1d7a48491f3475c97e", "f86edd60952fee3ed6413a2a95782a49823e24645dce27c08c7db", "7decdbd0b274350167ac652f0d71d8ce9bfaefb696b37aa0d3b1c1db", "9cf22fcf8c9046c336a3403671e77847754146aad0d9912ffb473bdf", "f86edd60952fee3ed6413a2a95782a49823e24645dce27c08c7db920", "3a84d3213a3d0926b65b85af424fa4142e03da79059af6037fec449b", "3b0018b793b4f5c3e9d6d900d5a9c7ba6038713969e003b617f8a9fc", "f86edd60952fee3ed6413a2a95782a49823e24645dce27c08c7db92066e2", "0020bb98eb586c398de2ec9f3bc4a09e2b31df164ab539b2065c8701", "128aef0eaab32bd9431599ce63f60ff76e4aaecd646c41fe6520a477", "e449e2c812e69a7344131f0674828220af3d10569a9d5b6abdfb3916", "beaa2b7330beeb2d18d405acbabfbb408a011e773c45a3022be5e018", "f86edd60952fee3ed6413a2a95782a49823e24645dce27c08c7db92066", "f86edd60952fee3ed6413a2a95782a49823e24645dce27c08c7", "65fb16895c3c0ececdffddc548789011b84d6dd478c39d8b5ba44151", "ff38c7d3fef553aa9cd50b35d1550e91264577d104619f9ff9f15607", "a24e1910e6e1fc9ebffb78694c8c1c159bc5979dce5a27b9e781d1ad", "f86edd60952fee3ed6413a2a95782a49823e24645dce27c08c7d", "4c7144edfe88c29fa298af4117026e0378e4deb81e748ade553e3ff5", "a5710943f4cda2dbb215935fa803972c629347f0997645c56a6af404", "852946492b191edde85a0348bbc665689da474b4d6419eff11842d18", "341c0902001dcfd3e1ee47acd738fe5192b5f13e5bb8f1dc9ce8e9cd", "c5d06d38e8b1f89e57c2263d870c1fa968111d62e692eb510b0f636d", "f892bc646afb7f4b6bee1466b897a12c83fa32c2b3777d343c23c95a", "e03889a45805090915a40ce26c3e0148aa5072d13c13cadabc4007f4", "ec3733e268dc0508f04cc0ed2d0379ef2e54f960ad3197bea7dbf68b", "8ec0225375bed1f41f9116ea7e304ff7c08974d6705097a7cc52c3a9", "f0f096baf3f4729f579ac4f9a48ab3102b82ba3fa71d5c3b37d1f36b", "eb86b72885c9fd20844c3498063733a447bfc02ec7c7572b5075433b", "074b17cdd47b780198245c5da9a91d87e8e45c4c47be54d8623a5afc", "a9e0ed8d3154c0d7ebc686b847f442b0fb03bff0d67da7d5c6506be9", "7511e77a778a2dcfb906b3a358037eb317db86e21171849f49fa0ab2", "f86edd60952fee3ed6413a2a95782a49823e24645dce27c08c7db92066e", "c6d751c80ce523ab23e2bff7c679d31e927cd9f98a5b2c6103673c39", "b1911f3458a0f73a25ff2f511a4522f8fd57c1b2e06e17de57c9bd59", "30a73e6dc1cb90d9d7a073a89ae80c4e9dc4e830ec65700a245bbd37", "369638ca693f8c17e33e5c5d379b08c01e52b5f99b4c14cb34404eee", "33fd8265a5861195c870b74d71a81d3df92389852bbf5a6bab37fa21", "f86edd60952fee3ed6413a2a95782a49823e24645dce27c08c7db9", "801961e195e5159a0aae400f764469adb2f570609e84d915c4b34fd7", "6535a3d2f90d1bfb96c9e74783ce03a3bc1e6767c624ba44e3f7308d", "afcacb56c6d356ad4e3a04de364f5f85199118bff8373747a8b66f48", "6e5ca052ced8c49da1c084ef72afc1db7c778d57c0dfe5a5bf755278", "f86edd60952fee3ed6413a2a95782a49823e24645dce27c08c7db92066e2a1", "f86edd60952fee3ed6413a2a95782a49823e24645dce27c08c7db92066e2a", "3c2cf2a96f7d142533f2fc3d92b01d01c7e9ede24520eb45b289c4f7", "4e7186dbe9ac6d138a437a770c54ff709d3dab3e679296be02aef718", "320f7e178eedc903bdddc68727b8c2447df509d06204b4951f3ffb47", "9c0877a138f5308a6909b5101d9e6bd2fabd36bf0f69d221ffd546fa", "568a60e9f7b6783c2955ecc46264be6381d9e5619f12575bd0a772d5", "76470137d8d81e4455bfa0fc5cc23f14953b380d2c5fd3ec3efddfe8", "bbab974b6bffc7532696e9f2c67da10d6626461533181fb731d7bd1a"]}' --initFunction new

```

#### 🛠 Usage

```rust
near view sweat_01_testing.testnet get_steps_from_tge '{ }'
'0'

near view sweat_01_testing.testnet formula '{"steps_from_tge":"1", "steps":1000}'
'999999999912699776'

near view sweat_01_testing.testnet ft_balance_of '{"account_id":"intmainreturn0.testnet"}'
'0'

near call sweat_01_testing.testnet record_batch '{"steps_batch": [["intmainreturn0.testnet", 10000],["poddubny.testnet", 1000] ]}' --accountId intmainreturn0.testnet --gas=300000000000000

near view sweat_01_testing.testnet ft_balance_of '{"account_id":"intmainreturn0.testnet"}'
'10049999991195916064'

near view sweat_01_testing.testnet get_steps_from_tge '{ }'
'11000'

near call sweat_01_testing.testnet ft_transfer '{"receiver_id":"poddubny.testnet", "amount":"2", "memo":"hello world!"}' --accountId intmainreturn0.testnet --gas=2428088695050 --depositYocto 1

near view sweat_01_testing.testnet ft_balance_of '{"account_id":"intmainreturn0.testnet"}'
'10049999991195916062'

```
