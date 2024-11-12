# gblend

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

CLI tool for bootstrapping, building, and deploying Fluent Network projects.

## Installation

```bash
cargo install gblend
```

## Usage

```bash
# Initialize a new Rust project
gblend init rust --help

# Build your project
gblend build rust --help

# Deploy to network
gblend deploy --help


```

> 📌 **Note:** We also support legacy version of the CLI. That allows you to bootstrap a project with a single command:

```bash
gblend init
```

You can find more information about legacy mode in the [legacy section](#legacy).

## Commands

```bash
gblend <COMMAND>

Commands:
  init    Initialize a new project
         Subcommands:
         - rust    Initialize Rust smart contract project
  
  build   Build the project
         Subcommands:
         - rust    Build Rust smart contract project
  
  deploy  Deploy the compiled WASM file to a specified network

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Project Structure

```bash
your-project/
├── src/
│   └── lib.rs
├── Cargo.toml
└── .gitignore
```

## Legacy

<p align="center">
  <img src="https://i.ibb.co/CwXLfxb/Screenshot-2024-10-14-at-7-15-34-PM.png" alt="gblend" width="500" height="300">
</p>

### Choose Your Setup

You can start your project with any of the following setups:

- **Hardhat JavaScript (Solidity & Vyper)**: Ideal for developers comfortable with JavaScript.
- **Hardhat TypeScript (Solidity & Vyper)**: Perfect for those who prefer TypeScript for type-safety.
- **Rust**: Best for developers looking to leverage the power of Rust in WASM.
- **Blended app**: Blended app ( Wasm & Solidity template)
- **Exit**: Leave the setup.

## Contributing

**GBLEND** is an open-source project, and community contributions are vital to its growth and improvement. Whether it's fixing bugs, adding features, or improving documentation, all contributions are welcome. If you're interested in helping out, please take a look at our issues tracker and read our contribution guidelines before submitting a pull request.

Feel free to fork the repository, make your changes, and submit a pull request for review. Let's build something great together!

We use [Conventional Commits](https://www.conventionalcommits.org/) for commit messages:

```bash
# Format
<type>[optional scope]: <description>

# Types
feat:     New features
fix:      Bug fixes
docs:     Documentation only changes
style:    Code style changes (formatting, etc)
refactor: Code changes that neither fix bugs nor add features
test:     Adding or modifying tests
chore:    Changes to build process or auxiliary tools
```

### Workflow

1. Fork and clone the repository
2. Create a new branch: `git checkout -b feat/your-feature`
3. Make changes following conventional commits
4. Push changes: `git push origin feat/your-feature`
5. Open a Pull Request

## License

MIT License - see the [LICENSE](LICENSE) file for details.
