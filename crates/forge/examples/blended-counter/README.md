# Blended Counter

This example demonstrates how to build, test, and deploy a **blended Solidity + Rust (WASM)** smart contract using **Gblend** — a Foundry-compatible CLI for the **Fluent** network.

---

## Project Structure

```
src/
├── BlendedCounter.sol        # Main Solidity contract
└── power-calculator/         # Rust WASM module for power calculations
test/                         # Solidity tests
script/                       # Deployment scripts
foundry.toml                  # Project configuration
```

**How it works:**

* The Solidity contract `BlendedCounter.sol` interacts with a Rust module compiled to **WASM**.
* The Rust module (`power-calculator/`) implements computation logic using **fluentbase-sdk**.
* **Gblend** builds, tests, and deploys both Solidity and Rust components in a single unified workflow.

---

## 1. Project Setup

Create a new blended project:

```bash
gblend init my-blended-app
cd my-blended-app
```

### VS Code Configuration (optional)

To enable Rust Analyzer and Solidity support, create `.vscode/settings.json`:

```json
{
  "rust-analyzer.linkedProjects": [
    "src/power-calculator/Cargo.toml"
  ],
  "rust-analyzer.cargo.target": "wasm32-unknown-unknown",
  "files.watcherExclude": {
    "**/target/**": true,
    "**/out/**": true
  },
  "solidity.packageDefaultDependenciesDirectory": "lib",
  "solidity.packageDefaultDependenciesContractsDirectory": ""
}
```

Make sure you have the **wasm32** target installed:

```bash
rustup target add wasm32-unknown-unknown
```

---

## 2. Build

```bash
gblend build
```

Compiles both Solidity and Rust (WASM) contracts and generates artifacts in the `out/` directory.
The first build may take longer as the Fluent build image is downloaded for reproducible builds.

---

## 3. Test

```bash
gblend test -v
```

Runs all tests in a shared Fluent-compatible runtime.
Both Solidity and Rust contracts execute in the same environment — identical to the runtime used by Fluent nodes on-chain.

---

## 4. Deploy

Use the provided Forge-style script to deploy both contracts:

```bash
gblend script script/Deploy.s.sol \
  --private-key $PK \
  --rpc-url https://rpc.devnet.fluent.xyz \
  --broadcast
```

This script:

* Deploys the **Rust PowerCalculator** (compiled to WASM)
* Deploys the **Solidity BlendedCounter**
* Links them together on-chain

---

## 5. Verify

Contracts can be verified separately on Blockscout.

### Rust (WASM) Contract

```bash
gblend verify-contract <POWER_CALCULATOR_ADDRESS> power-calculator.wasm \
  --rpc-url https://rpc.devnet.fluent.xyz \
  --wasm \
  --verifier blockscout \
  --verifier-url https://devnet.fluentscan.xyz/api/
```

### Solidity Contract

```bash
gblend verify-contract <BLENDED_COUNTER_ADDRESS> BlendedCounter \
  --rpc-url https://rpc.devnet.fluent.xyz \
  --verifier blockscout \
  --verifier-url https://devnet.fluentscan.xyz/api/
```

Verification for WASM contracts may take several minutes.

---

## 6. Alternative: Direct Deployment

You can also deploy a single compiled WASM contract directly:

```bash
gblend create power-calculator.wasm \
  --rpc-url https://rpc.devnet.fluent.xyz \
  --private-key $PK \
  --broadcast \
  --verify \
  --wasm \
  --verifier blockscout \
  --verifier-url https://devnet.fluentscan.xyz/api/
```

---

## References

* [Gblend Installation Guide](https://docs.fluent.xyz/gblend/installation)
* [Fluent Overview](https://docs.fluent.xyz/knowledge-base/fluent-overview)
* [Foundry Book](https://getfoundry.sh/forge/overview)
