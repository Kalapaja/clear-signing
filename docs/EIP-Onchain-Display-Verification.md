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
    - [clearCall() Entry Point](#clearcall-entry-point)
    - [Display Identifier Storage](#display-identifier-storage)
    - [Selector Dispatch](#selector-dispatch)
    - [DisplayRegistry](#displayregistry)
- [Rationale](#rationale)
    - [Packed Format](#packed-format)
    - [Compile-Time Constants](#compile-time-constants)
    - [Delegatecall Semantics](#delegatecall-semantics)
    - [DisplayRegistry Trade-offs](#displayregistry-trade-offs)
- [Backwards Compatibility](#backwards-compatibility)
- [Security Considerations](#security-considerations)
- [Copyright](#copyright)

## Abstract

This standard defines `clearCall()` — a contract entry point that enforces a cryptographic binding between on-chain execution and the display specification presented to the signer. The standard extends the conventional transaction payload format from `selector || calldata` to `clearCall_selector || display_identifier || selector || calldata`, making the display identifier an explicit, verifiable part of every call. A contract stores a digest of its display specification as an immutable constant and verifies the provided identifier against it before execution, reverting if they do not match. A wallet cannot produce an accepted transaction unless it computed the identifier from the exact display specification shown to the user, establishing "display is law".

## Motivation

The standard Ethereum call format consists of a 4-byte function selector followed by ABI-encoded parameters. The selector identifies which function to execute; it carries no information about how the parameters should be presented to a human. The presentation layer is entirely detached from the execution layer, with no on-chain mechanism to verify that what a user saw before signing corresponds to what the contract will do.

This detachment means the display specification, however well-authored, remains advisory. Nothing in the execution layer verifies that the display a user approved corresponds to the calldata being submitted — the contract accepts any well-formed call regardless of what the user was shown.

`clearCall()` closes this gap by extending the call format with a display identifier. Where a conventional call is `selector || calldata`, a clear call is `clearCall_selector || display_identifier || selector || calldata`. The contract extracts the display identifier from the payload and verifies it against its committed value before delegating to the inner call. A wallet that renders an incorrect display will derive a different identifier, and the transaction will be rejected on-chain. The display specification is no longer advisory — it is a precondition of execution.

## Specification

The keywords "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119 and RFC 8174.

### Display Identifier

The display identifier is an opaque 32-byte value that uniquely identifies a display specification. How the identifier is derived from a display specification is defined by the companion standard (TBD EIP); this standard treats it solely as a commitment. Wallets MUST compute the display identifier locally from the display specification rendered to the user and MUST reject the transaction if it does not match the identifier present in the packed format.

### Packed Call Format

A clear call encodes the display identifier and inner calldata in a single packed byte sequence with the following layout:

```
bytes  0–3:   clearCall selector  (0x0ab793e2)
bytes  4–35:  display identifier  (32 bytes, opaque commitment)
bytes  36+:   inner calldata      (4-byte function selector followed by ABI-encoded parameters)
```

The `clearCall` selector is `bytes4(keccak256("clearCall()"))`, equal to `0x0ab793e2`.

### clearCall() Entry Point

Contracts MUST implement a function with the selector `0x0ab793e2` (corresponding to `clearCall()`) declared as `external payable`. The function receives no explicit parameters; all input is read directly from `msg.data`.

Upon invocation, the contract MUST execute the following steps in order:

1. Extract the display identifier from `msg.data[4:36]`.
2. Extract the inner function selector from `bytes4(msg.data[36:40])`.
3. Resolve the expected display identifier for the inner selector (see [Selector Dispatch](#selector-dispatch)).
4. Revert if no expected identifier is registered for the inner selector.
5. Revert if the extracted display identifier does not equal the expected identifier.
6. Execute the inner call by forwarding `msg.data[36:]` via `delegatecall` to `address(this)`.
7. Propagate the return data on success; propagate the revert reason on failure.

The use of `delegatecall` preserves `msg.sender` and the storage context of the calling contract.

A reference implementation:

```solidity
function clearCall() external payable returns (bytes memory) {
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

### Display Identifier Storage

Contracts MUST store each function's display identifier as a compile-time `bytes32` constant derived per the companion standard. Storing display identifiers in mutable state variables is NOT RECOMMENDED, as post-deployment modification would undermine the trustless binding between display and execution.

Example:

```solidity
// Value computed offline per the companion Display Specification standard
bytes32 constant TRANSFER_DISPLAY_ID = 0x1a2b3c…;
bytes32 constant APPROVE_DISPLAY_ID  = 0x4d5e6f…;
```

### Selector Dispatch

Contracts MUST implement a mapping from inner function selectors to their corresponding display identifiers and MUST revert when `clearCall()` is invoked with an inner selector for which no display identifier is registered. Contracts SHOULD implement this mapping as an `if` / `else if` chain over known selectors, resolving to the corresponding compile-time constant, and returning `bytes32(0)` for unrecognised selectors.

### DisplayRegistry

Contracts that cannot be upgraded to implement `clearCall()` MAY use a `DisplayRegistry` as an alternative verification mechanism. A `DisplayRegistry` is a deployed contract that maps `(address contractAddress, bytes4 selector)` to a display identifier maintained by an external authority such as a DAO or multisig.

The standard `DisplayRegistry` interface is:

```solidity
interface IDisplayRegistry {
    /// @notice Reverts if the display identifier does not match the registered value.
    /// @param contractAddress The address of the target contract.
    /// @param selector        The 4-byte inner function selector.
    /// @param displayId       The display identifier to verify.
    function verifyClearCall(
        address contractAddress,
        bytes4  selector,
        bytes32 displayId
    ) external view;
}
```

Wallets using the `DisplayRegistry` path MUST bundle the `verifyClearCall` check atomically with the main call so that a registry mismatch reverts the entire transaction. Wallets MUST embed registry contract addresses in firmware; wallets MUST NOT resolve registry addresses dynamically. Wallets MUST clearly differentiate registry-verified transactions from native `clearCall` verification in the user interface.

## Rationale

### Packed Format

The packed byte format incurs approximately 2,700 gas overhead per call — compared with approximately 9,000 gas for manual ABI decoding of an explicit `bytes32` parameter and approximately 10,000 gas for a delegatecall wrapper approach. This efficiency is achieved by reading fixed-offset byte ranges from `msg.data` rather than decoding a dynamic parameter list, while remaining deterministically parseable by block explorers and off-chain tooling.

Encoding the display identifier within `msg.data` rather than as an EIP-712 envelope field keeps the verification logic entirely inside the receiving contract without requiring changes to the transaction signing flow.

### Compile-Time Constants

Storing display identifiers as immutable compile-time constants makes on-chain modification impossible after deployment. Any attempt to alter a deployed contract's identifier requires redeployment, changing the contract address and triggering community re-verification. This property is essential to the "display is law" guarantee: the identifier cannot diverge from the specification the developer committed to at deployment time.

### Delegatecall Semantics

`delegatecall` is used for the inner call so that `msg.sender` and storage context are preserved. A regular call would substitute the contract's own address for `msg.sender`, breaking access control checks in the inner function that depend on the original caller's identity.

### DisplayRegistry Trade-offs

The `DisplayRegistry` pattern extends clear signing to non-upgradeable contracts at the cost of the trustless property: the registry is controlled by a governance body, not immutable code. This introduces a social-trust dependency that native `clearCall` verification does not. The registry is provided as a pragmatic fallback; the primary recommendation is for contracts to implement `clearCall()` directly. Wallets must communicate this distinction clearly to users.

## Backwards Compatibility

This standard introduces a new entry point (`clearCall()`) that does not conflict with existing function selectors or the Solidity `fallback` / `receive` dispatch mechanism. Existing contracts that do not implement `clearCall()` continue to function without modification; wallets that support this standard fall back to conventional transaction display for such contracts.

Contracts implementing `clearCall()` retain their existing ABI-defined functions, which remain callable directly via their original selectors. The `clearCall()` entry point is an additive interface extension.

Block explorers that do not implement `clearCall` unwrapping will present such transactions as fallback invocations with opaque packed data rather than decoded inner calls. Block explorers SHOULD implement unpacking of the packed format to preserve post-execution auditability.

The `DisplayRegistry` interface described in the Specification provides a migration path for deployed contracts whose code cannot be modified.

## Security Considerations

### Display Determinism

Wallets MUST produce a deterministic display identifier from a given `Display` struct. Any ambiguity in identifier computation could allow a malicious dApp to supply a `Display` struct that renders differently on different wallets while producing the same identifier, undermining the "display is law" guarantee.

### Replay Across Chains

A display identifier does not incorporate a chain ID or domain separator. Contracts deployed to multiple chains with identical bytecode will have the same display identifier on each chain. This is intentional: the display specification describes the semantic meaning of a function, which is chain-independent. Chain-specific context (e.g., native token symbol) is the wallet's responsibility to surface independently.

### Registry Compromise

When using the `DisplayRegistry` pattern, compromise of the registry's controlling authority allows an attacker to register an incorrect display identifier for any listed contract, causing wallets to accept transactions whose rendered display does not correspond to the on-chain execution. Wallets MUST prominently communicate to users that registry-verified transactions carry governance trust assumptions absent from native `clearCall` verification.

### On-Chain Footprint

The display identifier is permanently visible in transaction history. While the display specification itself is off-chain, the identifier reveals that a specific display version was used for a given call. Developers SHOULD NOT embed personally identifiable information in display specifications, as the identifier creates a linkable on-chain record.

### Block Explorer Transparency

Block explorers that do not implement `clearCall` unwrapping will present clear calls as opaque fallback invocations. Explorer operators SHOULD implement support for unpacking the packed format to maintain transparency for post-execution audits.

## Copyright

Copyright and related rights waived via [CC0](https://creativecommons.org/publicdomain/zero/1.0/).
