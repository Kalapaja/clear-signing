# Clear Signing Short Overview

Clear Signing is a transaction-authorization architecture that replaces blind signing with human-readable, verifiable
context. The contract is the law: execution is valid only if it matches the user-approved display (“display is law”).
Architecture splits the problem into two layers so wallets can validate both identity and intent without centralized
gatekeepers
or live dependencies.

## Core Idea

The goal is to let smart contract developers define the display format for their calldata so wallets can present
intent consistently and without trusted intermediaries.

- **Social layer (Address Verification):** Wallets verify contract identity via decentralized contract lists and
  community consensus to prevent phishing and impersonation.
- **Protocol layer (Display Format):** Developers ship a standardized, declarative display spec alongside the ABI so
  wallets can render and verify user intent locally. The contract enforces that the approved display exactly matches the
  executed transaction (“display is law”).

## Goals

- **Phishing protection:** Confirm who the counterparty contract is before signing.
- **Developer-driven meaning:** Bind human-readable intent to on-chain execution.
- **Cryptographic binding:** Use the EIP-712 `hashStruct` algorithm so the display spec is cryptographically tied to the
  contract and can be defined in contract code at compile/deploy time.
- **Local, offline verification:** Fully interpret transactions without RPC calls or external metadata services.
- **Stateless transactions:** Embed all required display metadata inside the transaction payload.

## Non-Goals

- **Contract list delivery:** The architecture defines the contract list format and proposes discovery, but how wallets
  fetch or embed lists is out of scope.
- **Legacy compatibility:** The trustless layer does not cover non-upgradeable contracts, but a fallback is possible via
  an on-chain DisplayRegistry plus constraint checks for smart contract wallets.
- **Execution path analysis:** Internal call tracing is not required; developers define display only for their
  contracts.
- **Business logic & tokenomics:** This does not validate economic soundness or smart contract code quality.

## High-Level Flow

1. Wallet resolves the contract address against contract lists to confirm identity.
2. Wallet parses the embedded display spec to render intent locally (e.g., `tokenAmount`, `nativeAmount`, `contract`,
   `token`, `match`, `array`).
3. Contract verifies the display spec against calldata to ensure the user approved exactly what executes.

## Why It Matters

Clear Signing avoids trusted intermediaries, works for air-gapped devices, and provides cryptographic guarantees that
the visual prompt matches on-chain behavior.
