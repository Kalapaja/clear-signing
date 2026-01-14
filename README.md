# Clear Signing

**Clear Signing** is an architecture designed to eliminate "blind signing" in blockchain transactions. It establishes a trustless bridge between a user's wallet and smart contracts, ensuring that what the user sees on their screen is exactly what the contract will execute.

## Key Concepts

- **Social Layer**: Decentralized **Contract Lists** for verifying the identity of smart contracts, distinguishing legitimate protocols from phishing attempts.
- **Technical Layer**: A standardized, declarative **Display Format** that translates raw calldata into human-readable intent, cryptographically bound to the execution logic via the `clearCall` pattern.

## Project Structure

This repository is organized into several modules:

- [**docs/**](./docs): Comprehensive documentation of the Clear Signing architecture.
  - [Architecture Overview](./docs/architecture.md): The core design principles and specification.
- [**contracts/**](./contracts): Experimental Solidity implementation of the Clear Signing protocol.
  - `ClearCallRouter.sol`: A router for executing and verifying clear calls.
  - `Display.sol`: Library for handling and hashing display specifications.
- [**crates/**](./crates): Rust tools and libraries for working with Clear Signing.
  - `clear-signing`: Core logic for verification and parsing display fields.
  - `clear-signing-format`: Simple string formatter implementation.
  - `clear-signing-cli`: Command-line interface for developers for local testing.
- [**schemas/**](./schemas): JSON schemas for the various data formats used in the project.
  - `display.schema.json`: Schema for the declarative display specification.
  - `contractlist.schema.json`: Schema for decentralized contract identity lists.
  - `tokenlist.schema.json`: Schema for verified token lists.
- [**examples/**](./examples): Real-world examples of Clear Signing configurations for popular protocols.
  - `erc20`: Examples for standard token transfers and approvals.
  - `uniswap`: Examples for decentralized exchange interactions.
  - `multicall`: Examples for batching multiple interactions.

## Getting Started

To learn more about the project, start with the [Architecture Overview](./docs/architecture.md).

For developers looking to integrate Clear Signing:
- **Smart Contract Developers**: See the [contracts](./contracts) folder for experimental implementations and the [DApp Implementation guide](./docs/dapp_implementation.md).
