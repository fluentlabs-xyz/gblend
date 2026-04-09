# GBLEND

A Foundry forge wrapper optimized for Fluent Network and WASM smart contract development.

## Installation

### Quick Install (Recommended)

1. Install `gblendup`:

```bash
curl -sSL https://raw.githubusercontent.com/fluentlabs-xyz/gblend/refs/tags/latest/gblendup/install | bash
```

1. Start a new terminal session or update source file

2. Install `gblend` using `gblendup`

```
gblendup
```

This will automatically download precompiled binaries for your platform or build from source if needed.

### Verify Installation

After installation, verify gblend is working:

```bash
gblend --version
```

### Updating

To update gblend to the latest version, simply run the installer again:

```bash
curl -sSL https://raw.githubusercontent.com/fluentlabs-xyz/gblend/refs/tags/latest/gblendup/install | bash
```

## Motivation

**gblend** is a wrapper around Foundry's forge that uses Fluent's version of REVM and is designed to work with Wasm
smart contracts. It enables you to:

- **Compile Rust smart contracts** for WASM execution
- **Deploy** contracts to Fluent Network
- **Verify** WASM contracts on-chain
- **Create new projects** using templates from [fluentlabs-xyz/examples](https://github.com/fluentlabs-xyz/examples)

The usage is very similar to regular Foundry forge with some additional features. For example, when verifying WASM
contracts, you need to pass the `--wasm` argument. Otherwise, it works almost identically to forge.

## Basic Commands

### Project Management

```bash
# Create a new project using Fluent examples
gblend init my-project

# Create project with specific template
gblend init my-project --template <template-name>

# Build your contracts
# For reproducibility, builds are run inside a Docker container.  
# The first build may take longer as the container image needs to be downloaded.
gblend build

# Clean build artifacts
gblend clean
```

### Testing

```bash
# Run tests
gblend test

# Run specific test
gblend test --match-test testMyFunction

# Run tests with gas reporting
gblend test --gas-report
```

### Deployment

```bash
# Deploy a contract

# Deploy a WASM contract with verification
# contract name - rust package name in pascal case with .wasm suffix
gblend create PowerCalculator.wasm \        
    --rpc-url <rpc-url> \       
    --private-key <key> \        
    --broadcast \        
    --verify \       
    --wasm \
    --verifier blockscout \      
    --verifier-url <verifier-url>

# Deploy a Solidity contract
# NOTE: constructor args should be the last argument if used
gblend create src/BlendedCounter.sol:BlendedCounter \
    --rpc-url <rpc-url> \
    --private-key <key> \
    --broadcast \
    --constructor-args <args>

# Deploy using a script
gblend script script/BlendedCounter.s.sol:Deploy \
    --rpc-url <rpc-url> \
    --private-key <key> \
    --broadcast
```

### Verification

```bash
# Verify a regular Solidity contract
gblend verify-contract <address> BlendedCounter \
    --verifier blockscout \
    --verifier-url <verifier-url>

gblend verify-contract <address> PowerCalculator.wasm \
    --wasm \
    --verifier blockscout \
    --verifier-url <verifier-url> \
    --constructor-args <args>

```

## Configuration

gblend uses the same configuration system as Foundry forge. Create a `foundry.toml` file in your project root:

You can find additional [setup and configurations guides](https://getfoundry.sh/config/overview) in the [Foundry Docs][foundry-docs] and in the [config crate](./crates/config/README.md):

- [Configuring with `foundry.toml`](https://getfoundry.sh/config/overview)
- [Setting up VSCode][vscode-setup]
- [Shell autocompletions][shell-setup]

**Profiles and Namespaces**

```toml
[profile.default]
src = "src"
out = "out"
libs = ["lib"]
optimizer = true
optimizer_runs = 200

[rpc_endpoints]
fluent = <rpc-url>

```

## Differences from Standard Forge

- **WASM Support**: Native support for WASM contract compilation and deployment
- **Enhanced Verification**: `--wasm` flag for verifying WASM contracts
- **Custom REVM**: Support for the fluentbase REVM implementation.

## Documentation

For complete documentation on forge commands, see the [Foundry Book](https://getfoundry.sh/forge/overview).

For Fluent-specific development guides, visit [Fluent Documentation](https://docs.fluent.xyz).

## Contributing

Contributions are welcome! Please see
our [contributing guidelines](https://github.com/fluentlabs-xyz/gblend/blob/main/CONTRIBUTING.md).

## Support

Having trouble? See if the answer to your question can be found in the [Foundry Docs][foundry-docs].

If the answer is not there:

- Join the [support Telegram][tg-support-url] to get help, or
- Open an issue with [the bug](https://github.com/foundry-rs/foundry/issues/new)

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in these crates by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
</sub>

## Acknowledgements

- Foundry is a clean-room rewrite of the testing framework [DappTools][dapptools]. None of this would have been possible without the DappHub team's work over the years.
- [Matthias Seitz](https://twitter.com/mattsse_): Created [ethers-solc] (now [foundry-compilers]) which is the backbone of our compilation pipeline, as well as countless contributions to ethers, in particular the `abigen` macros.
- [Rohit Narurkar](https://twitter.com/rohitnarurkar): Created the Rust Solidity version manager [svm-rs](https://github.com/roynalnaruto/svm-rs) which we use to auto-detect and manage multiple Solidity versions.
- [Brock Elmore](https://twitter.com/brockjelmore): For extending the VM's cheatcodes and implementing [structured call tracing](https://github.com/foundry-rs/foundry/pull/192), a critical feature for debugging smart contract calls.
- Thank you to [Depot](https://depot.dev) for sponsoring us with their fast GitHub runners and sccache, which we use in CI to reduce build and test times significantly.
- All the other [contributors](https://github.com/foundry-rs/foundry/graphs/contributors) to the [ethers-rs](https://github.com/gakonst/ethers-rs), [alloy][alloy] & [foundry](https://github.com/foundry-rs/foundry) repositories and chatrooms.

[foundry-docs]: https://getfoundry.sh
[foundry-compilers]: https://github.com/foundry-rs/compilers
[ethers-solc]: https://github.com/gakonst/ethers-rs/tree/master/ethers-solc/
[vscode-setup]: https://getfoundry.sh/config/vscode.html
[shell-setup]: https://getfoundry.sh/config/shell-autocompletion.html
[dapptools]: https://github.com/dapphub/dapptools
[alloy]: https://github.com/alloy-rs/alloy
Licensed under either of Apache License, Version 2.0 or MIT license at your option.

**Note**: gblend is built on top of Foundry forge and maintains full compatibility with existing forge projects while
adding Fluent Network and WASM-specific enhancements.
