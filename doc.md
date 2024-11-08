# Project Structure

```sh
src/
├── main.rs                 # Entry point, CLI definition
├── cli.rs                  # CLI implementation and routing
├── error.rs               # Error types and handling
├── config.rs              # Configuration management
├── commands/              # Command implementations
│   ├── mod.rs            # Commands module definition
│   ├── common/           # Shared functionality
│   │   ├── mod.rs        # Common module definition
│   │   ├── types.rs      # Shared types and traits
│   │   ├── deploy.rs     # Deployment logic
│   │   └── utils.rs      # Shared utilities
│   ├── rust/             # Rust-specific implementations
│   │   ├── mod.rs        # Rust module definition
│   │   ├── init.rs       # Rust project initialization
│   │   ├── build.rs      # Rust build process
│   │   └── templates/    # Rust project templates
│   ├── typescript/       # TypeScript-specific implementations
│   │   ├── mod.rs        # TypeScript module definition
│   │   ├── init.rs       # TypeScript project initialization
│   │   ├── build.rs      # TypeScript build process
│   │   └── templates/    # TypeScript project templates
│   ├── solidity/        # Solidity-specific implementations
│   │   ├── mod.rs        # Solidity module definition
│   │   ├── init.rs       # Solidity project initialization
│   │   ├── build.rs      # Solidity build process
│   │   └── templates/    # Solidity project templates
│   └── go/              # Go-specific implementations
│       ├── mod.rs        # Go module definition
│       ├── init.rs       # Go project initialization
│       ├── build.rs      # Go build process
│       └── templates/    # Go project templates
└── utils/               # General utility functions
    ├── mod.rs           # Utils module definition
    ├── fs.rs            # File system operations
    └── wasm.rs          # WASM-related utilities

tests/                   # Integration tests
├── commands/           # Tests for each command
│   ├── rust/
│   ├── typescript/
│   ├── solidity/
│   └── go/
└── common/            # Common test utilities

templates/             # Project templates
├── rust/
│   ├── basic/        # Basic Rust smart contract
│   └── advanced/     # Advanced Rust smart contract
├── typescript/
│   ├── basic/
│   └── advanced/
├── solidity/
│   ├── basic/
│   └── advanced/
└── go/
    ├── basic/
    └── advanced/
```

## Directory Structure Explanation

### Core Directories

- `src/`: Contains all source code for the CLI tool
  - `commands/`: Each language implementation is isolated in its own module
  - `common/`: Shared functionality used across different language implementations
  - `utils/`: General utility functions and helpers

### Language-specific Directories

Each language (Rust, TypeScript, Solidity, Go) has its own directory with:

- `init.rs`: Handles project initialization
- `build.rs`: Manages build process
- `templates/`: Contains templates used during initialization

### Templates

The `templates/` directory contains the actual project templates that will be used when initializing new projects. Each language has:

- `basic/`: Simple project template
- `advanced/`: More complex project template with additional features

## Key Design Principles

1. **Modularity**: Each language implementation is isolated in its own module
2. **Code Reuse**: Common functionality is shared through the `common/` directory
3. **Extensibility**: New languages can be added by creating new directories under `commands/`
4. **Separation of Concerns**: Build, init, and deploy logic are separated into different files
5. **Template Management**: All project templates are stored in a dedicated directory

## Adding a New Language

To add support for a new language:

1. Create a new directory under `src/commands/`
2. Implement the required traits from `common/types.rs`
3. Add templates under `templates/`
4. Update the `ProjectMode` enum in `main.rs`

## Testing

Tests are organized to mirror the source structure:

- Integration tests for each language under `tests/commands/`
- Common test utilities in `tests/common/`
