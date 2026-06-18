# Contributing to stellar-rec

First off, thank you for considering contributing to **stellar-rec**! It's people like you that make this project a great tool for the green energy ecosystem.

When contributing to this repository, please first discuss the change you wish to make via issue, email, or any other method with the owners of this repository before making a change.

Please note we have a [Code of Conduct](./CODE_OF_CONDUCT.md), please follow it in all your interactions with the project.

## 🚀 Getting Started

### Prerequisites

To build and test the smart contracts, you'll need:

- **Rust**: v1.81+
- **WASM Target**: `wasm32-unknown-unknown`
- **Soroban CLI**: For deploying and invoking contracts

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Install Soroban CLI
cargo install soroban-cli --locked
```

### Development Workflow

1.  **Fork the repository** on GitHub.
2.  **Clone your fork** locally:
    ```bash
    git clone https://github.com/your-username/stellar-rec.git
    cd stellar-rec
    ```
3.  **Create a branch** for your changes:
    ```bash
    git checkout -b feature/your-feature-name
    ```
4.  **Implement your changes** and add tests.
5.  **Run the tests** to ensure everything is working correctly:
    ```bash
    cargo test
    ```
6.  **Commit your changes** using descriptive commit messages.
7.  **Push to your fork** and **submit a Pull Request**.

## 🛠 Coding Standards

- **Rust Style**: Follow standard Rust idioms. Use `cargo fmt` to format your code.
- **Testing**: Every new feature or bug fix should include corresponding tests.
- **Documentation**: Update the `README.md` or relevant documentation if you change any public APIs or add new features.
- **Commits**: Use clear, concise, and descriptive commit messages.

## 🧪 Testing

We use Cargo's built-in test runner. Since this is a Soroban project, we have both unit tests within each contract and integration tests in the `tests/integration` directory.

```bash
# Run all tests
cargo test

# Run tests for a specific contract
cargo test -p rec-token
```

## 🐞 Reporting Bugs

If you find a bug, please open an issue and include:

- A clear, descriptive title.
- Steps to reproduce the bug.
- Expected behavior vs. actual behavior.
- Any relevant logs or screenshots.

## 💡 Suggesting Features

We love hearing new ideas! If you have a feature request:

- Check if the feature has already been suggested.
- Open a new issue with the "Feature Request" template.
- Explain why this feature would be useful and how it should work.

## 🛡 Security

If you discover a security vulnerability, please refer to our [Security Policy](./SECURITY.md). Do **not** open a public issue for security-related bugs.

---

Thank you for your contributions!
