---
eip: TBD
title: wallet_sendTransaction with Verifiable Metadata
description: A JSON-RPC method bundling a transaction with verifiable contextual metadata, ensuring wallets can interpret and validate the transaction without external dependencies.
author: TBD
discussions-to: TBD
status: Draft
type: Standards Track
category: Interface
created: 2026-03-13
requires: 1474, TBD (Onchain Display Specification), TBD (Onchain Display Verification)
---

## Table of Contents

- [Abstract](#abstract)
- [Motivation](#motivation)
- [Specification](#specification)
    - [Method](#method)
    - [Parameters](#parameters)
    - [Metadata Object](#metadata-object)
    - [Wallet Processing](#wallet-processing)
    - [Return Value](#return-value)
    - [Error Codes](#error-codes)
- [Rationale](#rationale)
    - [Push Model vs. Pull Model](#push-model-vs-pull-model)
    - [Extensibility](#extensibility)
- [Backwards Compatibility](#backwards-compatibility)
- [Security Considerations](#security-considerations)
- [Copyright](#copyright)

## Abstract

This standard defines `wallet_sendTransaction`, a JSON-RPC method that extends the conventional `eth_sendTransaction` with a `metadata` parameter. This parameter carries an array of verifiable context metadata for the transaction's smart contract calls. While the method is agnostic to the specific type of metadata it carries, any supported metadata MUST be cryptographically verifiable against the original transaction.

## Motivation

When a decentralized application (dApp) requests a transaction, the wallet receives calldata it must present to the user for approval. The raw calldata is semantically opaque—the wallet has no built-in knowledge of what the bytes represent. Current approaches rely on the wallet pulling metadata from external registries at signing time. If the registry is unavailable, unreachable, or lacks the necessary metadata, the wallet falls back to blind signing. Furthermore, unverifiable off-chain metadata introduces significant phishing vectors, as malicious actors can supply deceptive context.

Hardware signing devices cannot query external metadata at all; they operate in strictly air-gapped environments with no network access, requiring all information needed to verify a transaction to be present in the signing request itself. `wallet_sendTransaction` ensures that the transaction payload includes the cryptographically verifiable context needed to safely interpret it.

## Specification

The keywords "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119 and RFC 8174.

### Method

```
wallet_sendTransaction
```

### Parameters

```json
[
  transaction,
  metadata
]
```

| Position | Name          | Type                | Required | Description                                              |
|----------|---------------|---------------------|----------|----------------------------------------------------------|
| 0        | `transaction` | `TransactionObject` | yes      | Standard transaction object, identical to `eth_sendTransaction` |
| 1        | `metadata`    | `MetadataObject`    | yes      | Context metadata mapping for the transaction             |

The `transaction` object MUST follow the exact same structure as `eth_sendTransaction`. The wallet MUST reject the request if the `metadata` parameter is absent.

### Metadata Object

The `metadata` object is a key-value map where each key serves as a unique identifier for a specific metadata standard or well-known format, and the value contains the structured context data specific to that standard.

| Key | Type | Description |
|---|---|---|
| `[standard_identifier]` | `object`\|`array` | Identifier for the type of metadata being provided. Keys SHOULD be specific to an Ethereum proposal (e.g., `"eip-xyz"`) or a well-known identifier (e.g., `"display"`, `"abi"`). |

#### Example
```json
{
  "display": [...],
  "abi": [...]
}
```

The specific structure of the value (whether an `object` or `array`) is defined by the standard corresponding to the key.

### Wallet Processing

Upon receiving a `wallet_sendTransaction` request, a wallet processes the transaction depending on its role in the signing lifecycle. In all scenarios, the user maintains the ability to confirm or reject the transaction, identical to the behavior of `eth_sendTransaction`.

#### Terminal Wallet (Consuming and Signing)
If the wallet is the final entity responsible for signing the transaction (e.g., an externally owned account on a software wallet or a hardware signer), it processes the metadata for its intended purpose:

1. **Verification and Processing:** The wallet processes the metadata according to the rules of the supported standard. If the wallet does not recognize a key, or if no matching metadata is found for a specific call, it MUST treat the metadata as unavailable for that context and fall back to its default behavior.
2. **Approval:** Present the complete, rendered transaction context to the user.
3. **Execution:** If the user approves, sign the transaction and broadcast it to the network.
4. **Return:** Return the resulting transaction hash to the calling application.

A terminal wallet MUST NOT submit the transaction without explicit user approval and MUST NOT modify the transaction `data` provided in the request.

#### Intermediary Wallet (Forwarding and Wrapping)
If the wallet acts as an intermediary (e.g., a service managing a smart account or multisig that intercepts a request, wraps the inner execution in its own outer call, and forwards it to a hardware signer), it MUST preserve the context:

1. **Preserve Metadata:** The intermediary MUST retain the original `metadata` object provided in the incoming request.
2. **Append Context (Optional but Recommended):** When wrapping the original transaction inside a new call (e.g., an `execute` call on a smart account), the intermediary SHOULD append its own verifiable metadata to the `metadata` object corresponding to the new wrapper calls it introduces.
3. **Forward Request:** The intermediary forwards the newly wrapped transaction and the combined `metadata` object to the downstream wallet via `wallet_sendTransaction`.

This aggregation ensures that the final hardware wallet controlling the account receives the complete chain of metadata necessary to verify and display the entire sequence of execution (e.g., Smart Account Wallet -> Multisig -> DEX Swap).

#### Discovery of Supported Metadata

Wallets SHOULD expose the metadata standards they support via the `wallet_getCapabilities` RPC method defined in [EIP-5792](https://eips.ethereum.org/EIPS/eip-5792). This allows decentralized applications and intermediary wallets to determine whether it makes sense to include specific metadata payloads in the `wallet_sendTransaction` request.

For example, a wallet might indicate its supported metadata keys:
```json
{
  "0x2105": {
    "supportedMetadata": [
      "display",
      "abi"
    ]
  }
}
```
If a dApp or an intermediary wallet detects that a downstream wallet does not support a specific metadata standard, it MAY choose to omit that payload to optimize the request size. If it includes it anyway, the receiving wallet MUST safely ignore the unsupported key.


### Return Value

On success, the method returns the transaction hash as a 32-byte hex string, identical to the behavior of `eth_sendTransaction`.

```json
"result": "0x4e3a3754410177e8842851f9b47a2db3b2b07bc8..."
```

### Error Codes

Standard JSON-RPC error codes apply, including the following specific scenarios:

| Code    | Message                              | Description                                                      |
|---------|--------------------------------------|------------------------------------------------------------------|
| `-32602` | `Invalid params`                    | `metadata` is missing, empty, or incorrectly formatted           |
| `4001`  | `User rejected the request`         | User denied the transaction at the approval step                 |

Additional error codes MAY be returned depending on the specific validation rules of the processed metadata standard.

## Rationale

### Push Model vs. Pull Model

Bundling metadata with the transaction request (the "Push Model") rather than fetching them from an external registry (the "Pull Model") eliminates runtime dependencies. A wallet that relies on pulling metadata at signing time degrades to blind signing whenever the registry is unavailable, rate-limited, or returns an incorrect entry. The push model makes availability a property of the dApp, not the registry: if the dApp can propose a transaction, it must also provide the specifications needed to display it.

### Extensibility

By making the `metadata` parameter a generic key-value map, the standard decouples the transport mechanism from specific data formats. While the immediate motivation is to deliver Onchain Display Specifications (via the `"display"` key), this design allows future standards to use the same `wallet_sendTransaction` method to push other forms of context. For example, a dApp could provide an ABI for decoding (via `"abi"`), off-chain intent declarations, or alternative verification proofs using their respective EIP identifiers as keys, without requiring new RPC methods.

## Backwards Compatibility

`wallet_sendTransaction` uses the `wallet_` namespace, which is reserved for wallet-specific extensions and is not part of the standard Ethereum JSON-RPC API defined by `eth_`. Existing dApps using `eth_sendTransaction` continue to function without modification. Adoption of `wallet_sendTransaction` is opt-in; dApps that do not provide metadata continue to use `eth_sendTransaction`, and wallets continue to handle those calls with their existing display or blind-signing logic.

Wallets that do not implement `wallet_sendTransaction` SHOULD return a standard JSON-RPC Method Not Found error (`-32601`), allowing dApps to fall back to `eth_sendTransaction`.

ERC-4337 flows using `UserOperation` are fully supported by this architecture. When a terminal wallet receives an ERC-4337 transaction bundled with verifiable metadata via this method, it can utilize the metadata to display the transaction context to the user for approval. Upon approval, the wallet signs and submits the `UserOperation` to the bundler exactly as it would normally. No new RPC method is strictly required to accommodate account abstraction flows.

## Security Considerations

The security guarantees of `wallet_sendTransaction` depend entirely on the specific metadata standard being processed.

While metadata itself is inherently advisory context, the defining requirement of this standard is that it MUST be verifiable. Any metadata standard transmitted via this method MUST define a cryptographic binding mechanism to the transaction itself. This ensures that the wallet can trustlessly verify that the provided context accurately represents the underlying execution. If a standard's verification fails, the wallet MUST reject the metadata and fall back to its default behavior.

Wallets MUST safely handle unknown or unsupported metadata keys by falling back to blind signing (or rejecting the transaction, depending on policy) and MUST NOT attempt to parse or render arbitrary structures that could lead to injection attacks or misleading displays.

## Copyright

Copyright and related rights waived via [CC0](https://creativecommons.org/publicdomain/zero/1.0/).
