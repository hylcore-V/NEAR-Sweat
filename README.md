# Sweat Coin

## Testing

Install near-cli: `npm install -g near-cli`

Deploy to TestNet:
> near dev-deploy --wasmFile=res/sweat_coin.wasm

This will output the account name where it was deployed like `dev-1637234761005-51631980292450`.

Setup the contract:
> near call <contract name> new '{"oracle_id": "<your oracle>.near", "limit_per_day": "10000000000000000000000000"}' --accountId <contract name>

Record the batch of steps:
> near call <contract name> batch_record '{"steps": [["testmewell.testnet", 1000000]]}' --accountId <your oracle account>
