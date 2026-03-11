---
eip: TBD
title: Onchain Display Verification
description: A contract-side enforcement mechanism that binds on-chain execution to the display specification presented to the signer.
author: TBD
status: Draft
type: Standards Track
category: ERC
created: 2026-03-11
requires: 712, TBD
---

## Table of Contents

- [Abstract](#Abstract)
- [Motivation](#Motivation)
- [Specification](#Specification)
- [Rationale](#Rationale)
- [Backwards Compatibility](#Backwards-Compatibility)
- [Security Considerations](#Security-Considerations)
- [Copyright](#Copyright)

## Abstract

This standard defines `clearCall()` — a contract entry point that enforces a cryptographic binding between on-chain execution and the display specification presented to the signer. The standard extends the conventional transaction payload format from `selector || calldata` to `clearCall_selector || display_hash || selector || calldata`, making the display hash an explicit, verifiable part of every call. A contract stores a digest of its display specification as an immutable constant and verifies the provided hash against it before execution, reverting if they do not match. A wallet cannot produce an accepted transaction unless it computed the hash from the exact display specification shown to the user, establishing "display is law".

## Motivation

The standard Ethereum call format consists of a 4-byte function selector followed by ABI-encoded parameters. The selector identifies which function to execute; it carries no information about how the parameters should be presented to a human. The presentation layer is entirely detached from the execution layer, with no on-chain mechanism to verify that what a user saw before signing corresponds to what the contract will do.

This detachment means the display specification, however well-authored, remains advisory. Nothing in the execution layer verifies that the display a user approved corresponds to the calldata being submitted — the contract accepts any well-formed call regardless of what the user was shown.

`clearCall()` closes this gap by extending the call format with a display hash. Where a conventional call is `selector || calldata`, a clear call is `clearCall_selector || display_hash || selector || calldata`. The contract extracts the display hash from the payload and verifies it against its committed value before delegating to the inner call. A wallet that renders an incorrect display will derive a different hash, and the transaction will be rejected on-chain. The display specification is no longer advisory — it is a precondition of execution.
