# Cross chain link contracts

Cross chain link contracts is a set of contracts to create chain link via Wormhole from Wormhole bridged chains.

## Info

| Chain           | Chain ID | Contract Address                                                 |
| --------------- | -------- | ---------------------------------------------------------------- |  
| Polygon Testnet | Mumbai   | 0xff7aD632d39D169E2595Cc87591191a5F54E48A5                       |
| Terra Testnet   | pisco-1  | terra18sy2gpk9w308e9k7dz8l64v4x78344kv5apz5fylr7cdnscvtuss6q8m9e |

Channel to Desmos: channel-513

## Quick start

1. Setup `.env` using `.env.sample` inside `scripts/`
2. Get faucet on three chains: EVM testnet, Terra2 testnet, Desmos testnet
3. Create Desmos profile using DESMOS_MNEMONIC
4. Run `yarn demo` to pass IBC packet
5. Wait for script finished then the new chain link will be created