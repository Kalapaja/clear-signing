---
eip: TBD
title: Onchain Clear Signing Verification
description: A contract-side enforcement mechanism that binds on-chain execution to the display specification presented to the signer.
author: TBD (At least one author must include GitHub username)
discussions-to: TBD (Ethereum Magicians forum URL required)
status: Draft
type: Standards Track
category: ERC
created: 2026-03-11
requires: 712
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
- [Backwards Compatibility](#backwards-compatibility)
    - [Opt-in Adoption](#opt-in-adoption)
    - [Non-Upgradeable Contracts](#non-upgradeable-contracts)
    - [Tooling Compatibility](#tooling-compatibility)
- [Security Considerations](#security-considerations)
    - [Invalid clearCall Implementation](#invalid-clearcall-implementation)
- [Copyright](#copyright)

## Abstract

This standard defines `clearCall()` — a contract entry point that enforces a cryptographic binding between on-chain execution and the display specification presented to the signer. The standard extends the conventional Ethereum call format from `selector || calldata` to `clearCall_selector || display_identifier || selector || calldata`, embedding the display identifier as an explicit, verifiable component of every call. A contract embeds the expected display identifier for each supported function in its bytecode at deployment and verifies the provided value before delegating execution, reverting on mismatch. This transforms display specifications from advisory metadata into enforced preconditions of execution.

## Motivation

The Ethereum call format encodes function calls as a 4-byte selector followed by typed parameters. The selector identifies what to execute but says nothing about what the arguments represent, and there is no on-chain link between the calldata and the description shown to the signer.

`clearCall()` closes this gap by embedding a display identifier in every call — derived from the specification the contract committed to at deployment and enforced by the contract's own logic. A wallet that renders any specification other than the committed one produces a different identifier, and the transaction reverts. No external authority is required; the binding is a property of the deployed bytecode.

## Specification

The keywords "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119 and RFC 8174.

### Display Identifier

The display identifier is an opaque 32-byte value that uniquely identifies a display specification, computed as defined in the companion Onchain Clear Signing Specification standard (EIP-TBD).

Wallets MUST compute the display identifier locally from the exact display specification presented to the user. A wallet MUST reject a transaction before submission if the locally computed identifier does not match the display identifier present in the `clearCall` payload.

Contracts MUST resolve the expected display identifier for each supported function selector. Upon receiving a `clearCall`, the contract MUST extract the display identifier from the payload and verify it against the expected identifier. If the identifiers do not match, the contract MUST revert.

### Packed Call Format

A `clearCall` payload MUST use the following packed byte sequence layout:

| Range (bytes) | Field              | Description                                                                 |
|---------------|--------------------|-----------------------------------------------------------------------------|
| 0–3           | Function Selector  | The `clearCall()` selector: `0x0ab793e2`                                    |
| 4–35          | Display Identifier | The 32-byte display identifier                                              |
| 36+           | Inner Calldata     | The 4-byte selector and ABI-encoded parameters of the target function       |

```
┌─────────────┬───────────────────────┬─────────────────────────────┐
│   Bytes 0-3 │       Bytes 4-35      │         Bytes 36+           │
├─────────────┼───────────────────────┼─────────────────────────────┤
│  clearCall  │   Display Identifier  │      Inner Calldata         │
│  Selector   │      (32 bytes)       │  ┌──────────┬──────────────┐│
│ 0x0ab793e2  │                       │  │ Selector │  ABI Params  ││
│             │                       │  │ (4 bytes)│              ││
└─────────────┴───────────────────────┴──┴──────────┴──────────────┘
```

### Display Identifier Storage

Contracts MUST implement a mechanism to resolve or verify the expected display identifier for a given function selector. Developers MAY choose from the following strategies based on their requirements for gas efficiency and upgradeability.

#### Compile-time Constants
The most gas-efficient approach, recommended for contracts with a single, static display specification per function. This approach also works well for upgradeable proxy contracts: during a proxy upgrade, the display identifier is stored in the implementation contract's bytecode and changes automatically when the implementation is replaced.

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

When the inner calldata of a `clearCall` is itself a `clearCall`, the payloads are nested. Wallets MUST process nested payloads recursively: render and verify the outermost call first, pause when a nested `clearCall` is encountered, render the inner specification, and resume after completion. This continues until a non-`clearCall` inner selector is reached. Each layer's display identifier MUST be independently verified against the specification rendered at that layer.

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

The `clearCall()` function uses raw packed `msg.data` parsing instead of explicit ABI parameters like `clearCall(bytes32 displayId, bytes calldata innerCall)`. The packed format defines a fixed layout: bytes 0–3 contain the `clearCall()` selector (`0x0ab793e2`), bytes 4–35 contain the display identifier, and bytes 36 onward contain the inner function calldata. This layout allows the implementation to extract both the display identifier and inner calldata using fixed-offset reads from `msg.data`, avoiding ABI decoding overhead.

The packed byte format adds approximately 3,764–3,979 gas overhead per call measured against direct calls, using the `delegatecall` reference implementation. The fixed-offset structure simplifies tooling: the inner calldata is always recoverable by stripping the first 36 bytes, with no dynamic offset computation required. Block explorers and indexers must implement this unwrapping (see [Tooling Compatibility](#tooling-compatibility)).

## Backwards Compatibility

### Opt-in Adoption

`clearCall()` is an additive entry point that does not conflict with existing function selectors or the Solidity `fallback` / `receive` dispatch mechanism. Contracts that implement `clearCall()` retain all existing ABI-defined functions, which remain callable directly via their original selectors — direct calls, internal calls, and contract-to-contract calls all bypass `clearCall()` entirely.

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

## Copyright

Copyright and related rights waived via [CC0](../LICENSE.md).
