# Blended Counter

This project demonstrates how to build, test, and deploy a **blended Solidity + Rust (WASM)** smart contract using **Gblend** — a Foundry-compatible CLI for the Fluent network.

---

## Project Structure

```
src/
├── BlendedCounter.sol        # Main Solidity contract
└── power-calculator/         # Rust WASM module for power calculations
test/                         # Solidity tests (Forge-compatible)
script/                       # Deployment scripts
foundry.toml / gblend.toml    # Project configuration
```

**How it works:**

* The Solidity contract `BlendedCounter.sol` calls a Rust module compiled to WASM.
* The Rust module (`power-calculator/`) contains compute logic built with `fluentbase-sdk`.
* Gblend compiles both Solidity and Rust components into a unified project ready for deployment.

---

## Usage

### Build

```bash
gblend build
```

Compiles both Solidity and Rust sources into deployable artifacts.

---

### Test

```bash
gblend test
```

Runs unit tests written in Solidity from the `test/` directory using Forge-compatible syntax.

---

### Deploy with a Script

```bash
gblend script script/BlendedCounter.sol:Deploy \
  --rpc-url <your_rpc_url> \
  --private-key <your_private_key>
```

Executes the deployment script using the specified RPC endpoint and private key.

---

### Direct Deployment with `gblend create`

You can also deploy a compiled WASM contract directly:

```bash
gblend create RustToken.wasm \
  --rpc-url <your_rpc_url> \
  --private-key <your_private_key> \
  --broadcast \
  --verify \
  --wasm \
  --verifier blockscout \
  --verifier-url https://testnet.fluentscan.xyz/api/
```

This command deploys the `RustToken` contract to Fluent Testnet, broadcasts the transaction, and verifies it on Blockscout.

---

## Documentation

* Network parameters: [Connect to Fluent](https://docs.fluent.xyz/connect-to-fluent/#fluent-testnet)
* Gblend usage: [Fluent Docs → Gblend](https://docs.fluent.xyz/gblend/usage)
* Foundry basics: [Foundry Book](https://getfoundry.sh/forge/overview)
* Fluent overview: [Fluent Knowledge Base](https://docs.fluent.xyz/knowledge-base/fluent-overview/)

---

Добавить секцию о “Common Issues” (типичные ошибки при установке и сборке) будет полезно для тех, кто впервые пробует Gblend. Хочешь, я подготовлю короткий блок с ними?
