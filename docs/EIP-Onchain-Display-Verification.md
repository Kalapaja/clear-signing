---
eip: TBD
title: Onchain Display Verification
description: A contract-side enforcement mechanism that binds on-chain execution to the display specification presented to the signer.
author: TBD
discussions-to: TBD
status: Draft
type: Standards Track
category: ERC
created: 2026-03-11
requires: TBD
---

## Table of Contents

- [Abstract](#abstract)
- [Motivation](#motivation)
- [Specification](#specification)
    - [Display Identifier](#display-identifier)
    - [Packed Call Format](#packed-call-format)
    - [Display Identifier Storage](#display-identifier-storage)
    - [Nested clearCall Composition](#nested-clearcall-composition)
    - [clearCall() Entry Point: Reference Implementation](#clearcall-entry-point-reference-implementation)
- [Rationale](#rationale)
    - [Packed Format](#packed-format)
    - [Compile-Time Constants](#compile-time-constants)
    - [Delegatecall Semantics](#delegatecall-semantics)
- [Backwards Compatibility](#backwards-compatibility)
    - [Opt-in Adoption](#opt-in-adoption)
    - [Non-Upgradeable Contracts](#non-upgradeable-contracts)
    - [Tooling Compatibility](#tooling-compatibility)
- [Security Considerations](#security-considerations)
    - [Invalid clearCall Implementation](#invalid-clearcall-implementation)
    - [Display Determinism](#display-determinism)
    - [Registry Compromise](#registry-compromise)
    - [On-Chain Footprint](#on-chain-footprint)
- [Copyright](#copyright)

## Abstract

This standard defines `clearCall()` — a contract entry point that enforces a cryptographic binding between on-chain execution and the display specification presented to the signer. The standard extends the conventional Ethereum call format from `selector || calldata` to `clearCall_selector || display_identifier || selector || calldata`, embedding the display identifier as an explicit, verifiable component of every call. A contract embeds the expected display identifier for each supported function in its bytecode at deployment and verifies the provided value before delegating execution, reverting on mismatch. This transforms display specifications from advisory metadata into enforced preconditions of execution.

## Motivation

The Ethereum ABI encodes function calls as a 4-byte selector followed by typed parameters. The selector is derived from the function's canonical signature — name and parameter types only (e.g., `transfer(address,uint256)`) — as the binary interface requires only types for encoding and decoding; parameter names carry no meaning at the protocol level (a design that also enables function overloading). This leaves a semantic gap: the selector identifies what to execute but says nothing about what the arguments represent, and there is no on-chain link between the calldata and the description shown to the signer.

Binding the display specification to the contract itself achieves trustlessness and decentralization: the display identifier is embedded in the contract's bytecode at deployment and enforced by the contract's own logic on every call, without any external authority that could be bypassed or compromised.

This standard closes both gaps. `clearCall()` extends the call format to embed a display identifier derived from the specification the contract committed to at deployment, making the correspondence between display and calldata verifiable on-chain before execution. A wallet that renders any specification other than the committed one will produce a different identifier, and the transaction will revert.

## Specification

The keywords "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119 and RFC 8174.

### Display Identifier

The display identifier is an opaque 32-byte value that uniquely identifies a display specification, computed as defined in the companion On-chain Display Specification standard.

Wallets MUST compute the display identifier locally from the exact display specification presented to the user. A wallet MUST reject a transaction before submission if the locally computed identifier does not match the display identifier present in the `clearCall` payload.

Contracts MUST resolve the expected display identifier for each supported function selector. Upon receiving a `clearCall`, the contract MUST extract the display identifier from the payload and verify it against the expected identifier. If the identifiers do not match, the contract MUST revert.

### Packed Call Format

A `clearCall` payload MUST use the following packed byte sequence layout:

| Range (bytes) | Field              | Description                                                                 |
|---------------|--------------------|-----------------------------------------------------------------------------|
| 0–3           | Function Selector  | The `clearCall()` selector: `0x0ab793e2`                                    |
| 4–35          | Display Identifier | The 32-byte display identifier                                              |
| 36+           | Inner Calldata     | The 4-byte selector and ABI-encoded parameters of the target function       |

Any data appended after the inner calldata MUST be ignored by the `clearCall()` entry point but MAY be used by the target function.

### Display Identifier Storage

Contracts MUST implement a mechanism to resolve or verify the expected display identifier for a given function selector. Developers MAY choose from the following strategies based on their requirements for gas efficiency and upgradeability.

#### Compile-time Constants
The most gas-efficient approach, recommended for contracts with a single, static display specification per function.

```solidity
bytes32 constant TRANSFER_DISPLAY_ID = 0x1a2b3c...;
```

#### Deploy-time Immutables
Suitable for factory-deployed contracts where the display specification is fixed at deployment but may vary between instances (e.g., based on token parameters).

```solidity
bytes32 immutable _transferDisplayId;

constructor(string memory name, string memory symbol) {
    // Display identifier computed at deploy time from token-specific parameters
    _transferDisplayId = _computeTransferDisplayId(name, symbol);
}
```

#### Runtime Storage
Used when display specifications must often be updated.

**One-to-One Mapping:** Maps a function selector to its current authoritative display identifier.
```solidity
mapping(bytes4 => bytes32) private _displayIdentifiers;

function setDisplayIdentifier(bytes4 selector, bytes32 displayId) external onlyOwner {
    _displayIdentifiers[selector] = displayId;
}
```

**One-to-Many Mapping:** Maps multiple valid display identifiers to a function selector. This is useful for supporting multiple versions of a display specification simultaneously.
```solidity
mapping(bytes32 => bytes4) private _authorizedSelectors;

function authorizeDisplay(bytes32 displayId, bytes4 selector) external onlyOwner {
    _authorizedSelectors[displayId] = selector;
}

function _verifyDisplay(bytes4 selector, bytes32 displayId) internal view returns (bool) {
    return _authorizedSelectors[displayId] == selector;
}
```

### Nested clearCall Composition

Transaction flows may involve layered execution: a smart contract account wraps an inner call — such as a DEX swap — before forwarding it to the target contract. Each layer in this call tree carries its own display identifier.

Wallets MUST process nested `clearCall` payloads recursively. Rendering begins with the outermost call; when a nested `clearCall` is encountered during field iteration, the wallet MUST pause the current display and render the inner specification before resuming. This continues until a non-`clearCall` inner selector is reached.

For each layer, the wallet MUST verify the display identifier against the specification rendered at that layer. The complete call tree is considered verified only when every layer's identifier has been independently confirmed.

### clearCall() Entry Point: Reference Implementation

Contracts MUST implement a function with the selector `0x0ab793e2` (corresponding to `clearCall()`) declared as `external payable`. The implementation below uses `delegatecall` to retrofit clear signing onto an existing contract with minimal changes: `clearCall()` verifies the identifier and forwards the inner calldata to the contract itself, requiring no modifications to existing function implementations. Developers MAY instead decode the inner calldata and call the target function directly.

```solidity
function clearCall() external payable returns (bytes memory) {
    require(msg.data.length >= 40, "clearCall: payload too short");

    bytes32 displayId = bytes32(msg.data[4:36]);
    bytes4  selector  = bytes4(msg.data[36:40]);

    bytes32 expected = _expectedDisplayId(selector);
    require(expected != bytes32(0), "clearCall: unknown selector");
    require(displayId == expected,  "clearCall: display identifier mismatch");

    (bool success, bytes memory result) = address(this).delegatecall(msg.data[36:]);
    if (!success) {
        assembly { revert(add(32, result), mload(result)) }
    }
    return result;
}
```


## Rationale

### Packed Format

The packed byte format adds approximately 3,764–3,979 gas overhead per call measured against direct calls for token transfer and swap operations respectively, using the `delegatecall` reference implementation. This overhead is achieved by reading fixed-offset byte ranges from `msg.data` rather than ABI-decoding an explicit parameter. Developers using direct dispatch instead of `delegatecall` may reduce this overhead further. The fixed-offset layout also simplifies static analysis and tooling: the inner calldata is always recoverable by stripping the first 36 bytes, with no parsing required.

## Backwards Compatibility

### Opt-in Adoption

`clearCall()` is an additive entry point that does not conflict with existing function selectors or the Solidity `fallback` / `receive` dispatch mechanism. Contracts that implement `clearCall()` retain all existing ABI-defined functions, which remain callable directly via their original selectors — direct calls, internal calls, and contract-to-contract calls all bypass `clearCall()` entirely and continue to work as before. Adoption is fully opt-in.

### Non-Upgradeable Contracts

Contracts that cannot be modified cannot implement `clearCall()` and therefore cannot participate in the on-chain binding defined by this standard. For such contracts, display identifier binding must be established externally, keyed by chain ID, contract address, and function selector. Three approaches are recognised:

**Embedded display specifications.** Wallets SHOULD embed display specifications for well-known standard interfaces — such as ERC-20, WETH, and common staking contracts — directly in firmware. Trust is derived from the immutable behaviour of the standard interface rather than on-chain commitment.

**Off-chain display repositories.** A publicly accessible, community-maintained repository MAY map `(chainId, contractAddress, selector)` to a verified display specification. Each entry MUST be reviewed and approved by a human before publication. Wallets consuming such repositories MUST communicate to users that display specifications sourced this way carry social trust assumptions rather than cryptographic guarantees.

**On-chain DisplayRegistry.** A smart contract controlled by a DAO or multisig MAY maintain an on-chain mapping of `(chainId, contractAddress, selector)` to display identifiers, providing decentralised governance over the binding without requiring contract upgrades. Wallets MUST clearly differentiate registry-verified transactions from native `clearCall()` verification in the user interface.

### Tooling Compatibility

Tools that do not implement `clearCall` unwrapping — including block explorers, wallets, and off-chain indexers — will treat such transactions as opaque fallback invocations, obscuring the inner function call and its parameters. Such tools SHOULD implement unpacking of the packed format by stripping the first 36 bytes to recover the inner calldata, preserving auditability and correct display of transaction intent.

## Security Considerations

### Invalid clearCall Implementation

An incorrectly implemented `clearCall()` entry point — one that skips or weakens the display identifier verification — undermines the security guarantee of the entire standard. A contract that accepts any display identifier or performs a partial check provides no binding between display and execution. Implementations MUST perform a strict equality check between the extracted identifier and the stored expected value. Contracts SHOULD be audited with specific attention to the verification path and all revert conditions.

### On-Chain Footprint

The display identifier is permanently visible in transaction history. While the display specification itself is off-chain, the identifier reveals that a specific display version was used for a given call. Developers SHOULD NOT embed personally identifiable information in display specifications, as the identifier creates a linkable on-chain record.

## Copyright

Copyright and related rights waived via [CC0](https://creativecommons.org/publicdomain/zero/1.0/).
