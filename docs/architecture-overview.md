# Clear Signing Short Overview

Clear Signing is a transaction-authorization architecture that replaces blind signing with human-readable, verifiable
context. The core principle is "display is law": execution is valid only if it matches the user-approved display.
The architecture splits the problem into two layers, enabling wallets to validate both identity and intent without
centralized gatekeepers or live dependencies.

## Core Idea

The goal is to enable smart contract developers to define the display format for their calldata, allowing wallets to
present user intent consistently without trusted intermediaries.

- **Social layer (Address Verification):** Wallets verify contract identity using contract lists inspired by Token Lists
  to prevent phishing and impersonation. The `contract` and `token` display types reference these well-known
  registries.
- **Protocol layer (Display Format):** Developers define a standardized, declarative display specification alongside
  smart contract code, enabling wallets to render and verify user intent locally. This extends "code is law" with
  "display is law": the contract enforces that the approved display exactly matches the executed transaction.

## Goals

- **Phishing protection:** Confirm the identity of the counterparty contract before signing.
- **Developer-driven meaning:** Bind the display specification used to render human-readable context to on-chain
  execution.
- **Cryptographic binding:** Use the EIP-712 `hashStruct` algorithm to cryptographically tie the display specification
  to the contract, allowing it to be defined in contract code at compile/deploy time.
- **Comprehensive display specification:** Support rich display types with composable specifications that can display
  call graphs (e.g., smart account call -> multisig -> swap, or multicall -> [approve, swap, transfer]).
- **Local, offline verification:** Fully interpret transactions without RPC calls or external metadata services, making
  it suitable for air-gapped hardware wallets.
- **Backward compatibility:** Support well-known interfaces such as ERC-20, ERC-721, ERC-1155, and WETH by embedding
  their display specifications.
- **Stateless transactions:** Embed all required display metadata within the transaction request.

## Non-Goals

- **Contract list delivery:** The architecture defines the contract list format and proposes discovery mechanisms, but
  how wallets fetch or embed lists is out of scope.
- **Execution path analysis:** Internal call tracing is not required; developers define displays only for their own
  contracts.
- **Business logic and tokenomics:** This architecture does not validate economic soundness or smart contract code
  quality.
- **Legacy compatibility:** The trustless layer does not cover non-upgradeable contracts, though a fallback is possible
  via an on-chain DisplayRegistry with constraint checks for smart contract wallets.

## High-Level Flow

1. The wallet resolves the contract address against contract lists to confirm identity.
2. The wallet reads the embedded display specification and renders fields locally (e.g., `tokenAmount`, `nativeAmount`,
   `contract`, `token`, `match`, `array`).
3. The wallet verifies the embedded display specification against the corresponding display hash.
4. The contract verifies the display hash to ensure the user approved exactly what will be executed.

## Why It Matters

Clear Signing eliminates the need for trusted intermediaries, operates on air-gapped devices, and provides cryptographic
guarantees that the visual prompt accurately matches the on-chain execution.
