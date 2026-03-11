---
eip: TBD
title: Onchain Display Specification
description: A standardized on-chain display format and verification mechanism for smart contract intents.
author: TBD
status: Draft
type: Standards Track
category: ERC
created: 2026-03-10
requires: 712
---

## Table of Contents

- [Abstract](#Abstract)
- [Motivation](#Motivation)
- [Specification](#Specification)
    - [1. Type Definitions](#1-Type-Definitions)
        - [1.1 Example: ERC-20 Transfer](#11-Example-ERC-20-Transfer)
    - [2. Function Signature Format](#2-Function-Signature-Format)
    - [3. Variable References](#3-Variable-References)
        - [3.1 Literals](#31-Literals)
        - [3.2 Reference Containers](#32-Reference-Containers)
        - [3.3 Resolution Failure](#33-Resolution-Failure)
    - [4. Field Formats](#4-Field-Formats)
        - [4.1 Raw Solidity Types](#41-Raw-Solidity-Types)
        - [4.2 Rich Formats](#42-Rich-Formats)
        - [4.3 Address Formats](#43-Address-Formats)
        - [4.4 Value Formats](#44-Value-Formats)
        - [4.5 Structural Formats](#45-Structural-Formats)
    - [5. Localization](#5-Localization)
        - [5.1 Label Resolution](#51-Label-Resolution)
        - [5.2 Labels Structure](#52-Labels-Structure)
- [Rationale](#Rationale)
- [Security Considerations](#Security-Considerations)

## Abstract

Smart contract ABIs define parameter encoding but carry no semantic meaning: a UNIX timestamp is indistinguishable from
an arbitrary integer, and a nested call payload is an opaque byte array. This standard defines an **on-chain Display
specification** — a structured, developer-authored schema that maps raw calldata fields to rich display types (amounts,
durations, addresses, nested calls, and more). Each specification is uniquely identified by a 32-byte **display
identifier**
computed as an EIP-712 `hashStruct` digest. The identifier can be computed statically at compile or deploy time,
enabling
resource-constrained devices to cryptographically identify specifications without network access. A companion EIP
addresses
how this identifier is used for on-chain verification.

## Motivation

The Ethereum ABI was designed as a calling convention, not a presentation layer. It encodes and decodes function
arguments reliably, but the type system it exposes — `uint256`, `address`, `bytes`, and their array forms — carries no
domain meaning. A deadline and a price are both `uint256`; a token transfer and an arbitrary delegated execution are
both `bytes`. No information in the ABI tells a wallet, a hardware device, or an end user what a value represents or how
it should be displayed.

The absence of a standardized, machine-parseable display format is the direct cause of blind signing. Users are asked to
authorize transactions whose parameters they cannot interpret, relying entirely on the correctness and honesty of the
dApp interface. This trust assumption is irreconcilable with the security model of self-custodial wallets, particularly
hardware devices whose purpose is to provide an independent verification layer.

A display specification standard must satisfy two properties to be useful in practice. First, it must be rich enough to
cover the semantic types that appear in real contracts: denominated token amounts, timestamps, durations, percentage
rates, address roles, and nested execution payloads. Second, it must provide a compact **display identifier** that
resource-constrained devices can compute and verify without maintaining live network connections. This standard defines
the
type system and EIP-712-based display identifier that satisfies these requirements. A companion EIP addresses how this
identifier is used for on-chain verification.

## Specification

The keywords "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT
RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119 and RFC 8174.

**Scope and Responsibilities:**

The display specification defines a **semantic presentation layer** that describes *what* data should be displayed and
*what it means*, not *how* it should be visually rendered. The specification is responsible for:

- Identifying data types and their semantic meaning (e.g., "this is a token amount", "this is a timestamp")
- Structuring the data hierarchy and relationships
- Providing localized labels and descriptions
- Specifying which data should be emphasized or conditionally shown

Visual presentation is the responsibility of the wallet implementation. For example:

- A `tokenAmount` field semantically identifies an amount denominated in a specific token
- The wallet decides how to render it: as plain text, as a list item with token icon, with fiat conversion, with balance
  context, etc.
- Different wallet implementations may render the same specification differently based on their UI capabilities and user
  preferences

This separation ensures the specification remains device-agnostic while allowing wallets to optimize presentation for
their specific form factors (mobile, desktop, hardware wallets).

### 1. Type Definitions

A display specification is built from four composable types. The `displayHash` is computed as `hashStruct(Display)`
using EIP-712. Implementations MUST use the type strings shown below verbatim.

**`Display`** — the root type representing a complete display specification for one function.

- `abi` — a Solidity function signature string used to match the specification against an incoming function selector (
  see Section 2).
- `title` — a label reference or literal string used as the transaction title.
- `description` — human-readable description of the operation.
- `fields` — ordered array of `Field` definitions.
- `labels` — array of `Labels` bundles.

**`Field`** — defines a single display item.

- `title` — a label reference or literal string shown as the field name.
- `description` — optional human-readable description.
- `format` — the display format identifier (see Section 4).
- `case` — controls conditional visibility when nested inside a `switch` field. An empty array means always shown.
- `params` — an array of `Entry` pairs supplying arguments to the formatter. Keys are format-specific parameter names (
  see Section 4). Values are variable references or literals (see Section 3).
- `fields` — nested `Field` definitions used by structural formats (`map`, `array`, `switch`, `call`).

**`Labels`** — a locale-specific string bundle for user-facing text.

- `locale` — locale string (e.g. `en`, `fr`).
- `items` — an array of `Entry` pairs mapping label keys to translated strings.

**`Entry`** — a key-value pair used throughout as a generic parameter carrier.

The corresponding Solidity typehash constants are:

```solidity
// EIP-712 type hash for Entry
bytes32 constant ENTRY_TH = keccak256(
    "Entry(string key,string value)"
);

// EIP-712 type hash for Labels
bytes32 constant LABELS_TH = keccak256(
    "Labels(string locale,Entry[] items)Entry(string key,string value)"
);

// EIP-712 type hash for Field
bytes32 constant FIELD_TH = keccak256(
    "Field(string title,string description,string format,string[] case,Entry[] params,Field[] fields)Entry(string key,string value)"
);

// EIP-712 type hash for Display
bytes32 constant DISPLAY_TH = keccak256(
    "Display(string abi,string title,string description,Field[] fields,Labels[] labels)Entry(string key,string value)Field(string title,string description,string format,string[] case,Entry[] params,Field[] fields)Labels(string locale,Entry[] items)"
);
```

#### 1.1 Example: ERC-20 Transfer

The following shows a complete `displayHash` computation for the ERC-20 `transfer` function using the `Display` library:

```solidity
bytes32 constant TRANSFER_DISPLAY_HASH = Display.display(
    "function transfer(address to, uint256 amount)",  // abi
    "$labels.title",                                  // title
    "$labels.description",                            // description
    abi.encodePacked(                                 // fields
        Display.addressField(
            "$labels.sender",      // title
            "$labels.senderDesc",  // description
            "",                    // case (empty)
            "$msg.sender"          // value
        ),
        Display.tokenAmountField(
            "$labels.amount",      // title
            "$labels.amountDesc",  // description
            "",                    // case (empty)
            "$msg.to",             // token
            "$data.amount"         // amount
        ),
        Display.addressField(
            "$labels.recipient",      // title
            "$labels.recipientDesc",  // description
            "",                       // case (empty)
            "$data.to"                // value
        )
    ),
    abi.encodePacked(                                 // labels
        Display.labels(
            "en",                  // locale
            abi.encodePacked(      // items
                Display.entry("title", "Transfer"),
                Display.entry("description", "Transfer ERC-20 tokens to another address"),
                Display.entry("sender", "From"),
                Display.entry("senderDesc", "Address sending the tokens"),
                Display.entry("amount", "Amount"),
                Display.entry("amountDesc", "Amount of tokens to send"),
                Display.entry("recipient", "Recipient"),
                Display.entry("recipientDesc", "Address that will receive the tokens")
            )
        )
    )
);
```

### 2. Function Signature Format

The `Display.abi` field MUST be a full Solidity function signature of the form:

```
function <name>(<type> <name>, ...) [<modifier>]
```

The `function` keyword is required. Each parameter MUST include both a type and a name — parameter names are used to
populate `$data` so that field params can reference them by name (e.g. `$data.amount`). The optional state mutability
modifier MUST be one of `pure`, `view`, `payable`, or `nonpayable`; if omitted, `nonpayable` is assumed. Wallets MUST
reject any call where `msg.value > 0` and the modifier is not `payable`.

```
function transfer(address to, uint256 amount)
function approve(address spender, uint256 amount) nonpayable
function deposit() payable
```

The function selector used for matching is derived from the canonical ABI signature — type names only, parameter names
and modifier stripped — via `keccak256`, consistent with standard Ethereum ABI selector derivation.

### 3. Variable References

`Entry` values in `Field.params` are either a **variable reference** or a **literal**. A variable reference starts with
`$` and resolves at render time against the current rendering context. Any string that does not start with `$` is
treated as a constant literal value (e.g. a decimals count `"6"`, a basis `"10000"`, a bit label `"Read"`).

#### 3.1 Literals

Literals are untyped strings. Coercion to the type required by the formatter happens at render time and MUST follow
these rules:

| Target type | Coercion rule                                                        |
|-------------|----------------------------------------------------------------------|
| `bool`      | `"true"` or `"false"` (case-insensitive). Any other value MUST halt. |
| `uint`      | Decimal integer string parseable as `uint256`.                       |
| `int`       | Decimal integer string parseable as `int256`.                        |
| `address`   | Hex string parseable as a 20-byte address.                           |
| `bytes`     | Hex string with optional `0x` prefix.                                |
| `string`    | Used as-is.                                                          |

If the literal cannot be coerced to the required type, resolution MUST halt.

#### 3.2 Reference Containers

Two reference containers are defined:

**`$msg`** — the transaction context. Constant throughout the entire rendering of a display specification, including
inside nested structural fields. Supports exactly one level of field access, no nesting:

- `$msg.sender` — the caller address.
- `$msg.to` — the contract address receiving the call.
- `$msg.value` — the native value attached to the call (`uint256`).
- `$msg.data` — the raw calldata bytes.

**`$data`** — the decoded function arguments for the current rendering scope. At the top level, `$data` is populated by
decoding `$msg.data` against `Display.abi`. Structural formats (`map`, `array`, `switch`) create a new `$data` scope
from their own `params`; nested `fields` within those formats resolve `$data` against the new scope and do NOT inherit
the parent's `$data`. The following access patterns are supported:

- Named access: `$data.amount`, `$data.to`
- Positional access: `$data.0`, `$data.1`
- Nested path: `$data.order.token`
- Array index: `$data.items[0]`, `$data.items[-1]` (negative indices count from the end)
- Slice: `$data.items[1:3]`, `$data.data[:]`

#### 3.3 Resolution Failure

Resolution MUST halt if:

- The container is unknown.
- The referenced path does not exist in the current `$data` scope.
- An array index is out of bounds.
- The resolved value type is incompatible with what the formatter requires.
- A literal cannot be coerced to the required type.

### 4. Field Formats

#### 4.1 Raw Solidity Types

These formats display values as-is with minimal transformation, corresponding directly to their Solidity types. All
accept a single `value` param.

| Format    | Solidity type     | Display                                                                                                             |
|-----------|-------------------|---------------------------------------------------------------------------------------------------------------------|
| `boolean` | `bool`            | Localised yes/no string                                                                                             |
| `string`  | `string`          | UTF-8 string                                                                                                        |
| `bytes`   | `bytes`, `bytesN` | Hex-encoded bytes. Fixed-length `bytesN` (`bytes1`…`bytes32`) are accepted and displayed as hex.                    |
| `int`     | `intN`            | Signed integer. Accepts any width (`int8`, `int16`, `int32`, `int64`, `int128`, `int192`, `int256`, etc.).          |
| `uint`    | `uintN`           | Unsigned integer. Accepts any width (`uint8`, `uint16`, `uint32`, `uint64`, `uint128`, `uint192`, `uint256`, etc.). |

```solidity
// Using Display library
Display.booleanField(
    "$labels.approved",      // title
    "$labels.approvedDesc",  // description
    "",                      // case (empty)
    "$data.approved"         // value
)

// Equivalent raw hash computation
bytes32 approvedField = keccak256(abi.encode(
    FIELD_TH,
    keccak256(bytes("$labels.approved")),     // title
    keccak256(bytes("$labels.approvedDesc")), // description
    keccak256(bytes("boolean")),              // format
    keccak256(bytes("")),                     // case (empty)
    keccak256(abi.encodePacked(
        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("value")), keccak256(bytes("$data.approved"))))
    )),                                       // params
    keccak256(bytes(""))                      // fields (empty)
));
```

#### 4.2 Rich Formats

These formats interpret a raw Solidity value into a human-readable semantic representation.

**`datetime`** — displays a Unix timestamp as an absolute, locale-formatted date and time. Accepts any `uintN` type. An
optional `units` param specifies the unit of the input value; if omitted, seconds is assumed.

| Param   | Required | Description                                                                    |
|---------|----------|--------------------------------------------------------------------------------|
| `value` | yes      | Reference resolving to a `uintN` timestamp                                     |
| `units` | no       | Input unit: `"seconds"` (default), `"minutes"`, `"hours"`, `"days"`, `"weeks"` |

```solidity
Display.datetimeField(
    "$labels.deadline",      // title
    "$labels.deadlineDesc",  // description
    "",                      // case (empty)
    "$data.deadline"         // value
)
```

---

**`duration`** — displays a relative time span as a human-readable duration (e.g. "2 weeks", "3 days"). Accepts any
`uintN` type. An optional `units` param specifies the unit of the input value; if omitted, seconds is assumed.

| Param   | Required | Description                                                                    |
|---------|----------|--------------------------------------------------------------------------------|
| `value` | yes      | Reference resolving to a `uintN` duration value                                |
| `units` | no       | Input unit: `"seconds"` (default), `"minutes"`, `"hours"`, `"days"`, `"weeks"` |

```solidity
Display.durationField(
    "$labels.lockPeriod",      // title
    "$labels.lockPeriodDesc",  // description
    "",                        // case (empty)
    "$data.lockPeriod"         // value
)
```

---

**`percentage`** — displays a rational percentage computed as `value / basis`. Both params are interpreted as unsigned
integers; rendering MUST halt if `basis` is zero.

| Param   | Required | Description                                                                               |
|---------|----------|-------------------------------------------------------------------------------------------|
| `value` | yes      | Reference resolving to a `uintN` numerator                                                |
| `basis` | yes      | Literal or reference resolving to a `uintN` denominator (e.g. `"10000"` for basis points) |

```solidity
Display.percentageField(
    "$labels.fee",      // title
    "$labels.feeDesc",  // description
    "",                 // case (empty)
    "$data.feeBps",     // value
    "10000"             // basis
)
// Renders: "1.5%" for feeBps=150
```

---

**`bitmask`** — displays an integer as the list of labels for each set bit. Bit labels are supplied as additional params
using `#N` keys where `N` is the zero-based bit index. Only labels for set bits are shown.

| Param         | Required | Description                                                                   |
|---------------|----------|-------------------------------------------------------------------------------|
| `value`       | yes      | Reference resolving to a `uintN` bitmask. Accepts any unsigned integer width. |
| `#0`, `#1`, … | no       | Label string for each bit position                                            |

```solidity
Display.bitmaskField(
    "$labels.permissions",      // title
    "$labels.permissionsDesc",  // description
    "",                         // case (empty)
    "$data.permissions",        // value
    abi.encodePacked(           // bit labels
        Display.entry("#0", "Read"),
        Display.entry("#1", "Write"),
        Display.entry("#2", "Execute")
    )
)
```

---

**`units`** — displays a raw integer scaled by a decimal exponent (e.g. USDC with 6 decimals).

| Param      | Required | Description                                                                       |
|------------|----------|-----------------------------------------------------------------------------------|
| `value`    | yes      | Reference resolving to a `uintN` raw integer. Accepts any unsigned integer width. |
| `decimals` | yes      | Literal or reference resolving to a `uintN` number of decimal places.             |

```solidity
Display.unitsField(
    "$labels.amount",      // title
    "$labels.amountDesc",  // description
    "",                    // case (empty)
    "$data.amount",        // value
    "6"                    // decimals
)
// Renders: "1.234567" for amount=1234567
```

#### 4.3 Address Formats

The three address format types have different verification requirements. `address` is informational — any 20-byte value
is accepted and displayed with best-effort name resolution. `token` and `contract` require verification — rendering
halts if the address cannot be matched against a Token List or Contract List respectively. Both list types can be
curated by anyone: protocol authors, security firms, DAOs, or individual users through personal contact lists or
wallet "add token" features. The trust level a wallet assigns to a given list is a wallet policy decision outside the
scope of this standard. Developers MUST choose the format appropriate for the semantic role of the address.

**`address`** — displays a 20-byte address. Resolved to a human-readable name via local contacts, ENS, or other
directories if available; otherwise shown as a hex string. Use for addresses whose identity is informational but not a
security precondition (e.g. a recipient).

| Param   | Required | Description                       |
|---------|----------|-----------------------------------|
| `value` | yes      | Reference resolving to an address |

```solidity
Display.addressField(
    "$labels.recipient",      // title
    "$labels.recipientDesc",  // description
    "",                       // case (empty)
    "$data.to"                // value
)
```

---

**`token`** — displays a token address verified against a Token List. If the token is not found, rendering MUST stop.
Use when the token identity is a precondition for the user to assess the transaction (e.g. what asset is being
transferred). Supports NFTs via an optional `tokenId`.

| Param     | Required | Description                                     |
|-----------|----------|-------------------------------------------------|
| `value`   | yes      | Reference resolving to a token address          |
| `tokenId` | no       | Reference resolving to a `uint256` NFT token ID |

```solidity
Display.tokenField(
    "$labels.token",      // title
    "$labels.tokenDesc",  // description
    "",                   // case (empty)
    "$msg.to"             // value
)
```

---

**`contract`** — displays a contract address verified against a Contract List. If the address is not found, rendering
MUST stop. Use when a parameter identifies a contract receiving delegated authority or executing on the user's behalf —
where an unverified counterparty is a direct security risk. A canonical example is ERC-20 `approve`: without
verification, a phishing interface could substitute any contract as the `spender` and the wallet would display it
without warning.

| Param   | Required | Description                               |
|---------|----------|-------------------------------------------|
| `value` | yes      | Reference resolving to a contract address |

```solidity
// Full display spec for ERC-20 approve
bytes32 constant APPROVE_DISPLAY_HASH = Display.display(
    "function approve(address spender, uint256 amount) nonpayable",  // abi
    "$labels.title",                                                  // title
    "$labels.description",                                            // description
    abi.encodePacked(                                                 // fields
        Display.addressField(
            "$labels.owner",      // title
            "$labels.ownerDesc",  // description
            "",                   // case (empty)
            "$msg.sender"         // value
        ),
        Display.contractField(
            "$labels.spender",      // title
            "$labels.spenderDesc",  // description
            "",                     // case (empty)
            "$data.spender"         // value
        ),
        Display.tokenAmountField(
            "$labels.amount",      // title
            "$labels.amountDesc",  // description
            "",                    // case (empty)
            "$msg.to",             // token
            "$data.amount"         // amount
        )
    ),
    abi.encodePacked(                                                 // labels
        Display.labels(
            "en",                  // locale
            abi.encodePacked(      // items
                Display.entry("title", "Approve"),
                Display.entry("description", "Grant a contract permission to spend your tokens"),
                Display.entry("owner", "Owner"),
                Display.entry("ownerDesc", "Account granting spending permission"),
                Display.entry("spender", "Spender"),
                Display.entry("spenderDesc", "Contract receiving permission to spend tokens on your behalf"),
                Display.entry("amount", "Allowance"),
                Display.entry("amountDesc", "Maximum amount the spender is allowed to transfer")
            )
        )
    )
);
```

#### 4.4 Value Formats

**`nativeAmount`** — displays a native currency amount (e.g. ETH). An optional `direction` indicates whether the amount
flows `in` or `out` relative to the user. When `direction` is omitted, the amount is displayed without directional
indication.

| Param       | Required | Description                                                       |
|-------------|----------|-------------------------------------------------------------------|
| `amount`    | yes      | Reference resolving to a `uintN` in the smallest unit             |
| `direction` | no       | `"in"` or `"out"`. If omitted, no directional indicator is shown. |

```solidity
Display.nativeAmountField(
    "$labels.value",          // title
    "$labels.valueDesc",      // description
    "",                       // case (empty)
    "$msg.value",             // amount
    Display.Direction.Out     // direction
)
```

---

**`tokenAmount`** — displays a token amount denominated in a specific token, resolved against a Token List (see Section
4.3 for token verification requirements). Supports NFTs via `tokenId` and an optional transfer direction. When
`direction` is omitted, the amount is displayed without directional indication.

| Param       | Required | Description                                                       |
|-------------|----------|-------------------------------------------------------------------|
| `token`     | yes      | Reference resolving to a token address                            |
| `amount`    | yes      | Reference resolving to a `uintN` raw token amount                 |
| `tokenId`   | no       | Reference resolving to a `uintN` NFT token ID                     |
| `direction` | no       | `"in"` or `"out"`. If omitted, no directional indicator is shown. |

```solidity
// Fungible token
Display.tokenAmountField(
    "$labels.amount",      // title
    "$labels.amountDesc",  // description
    "",                    // case (empty)
    "$msg.to",             // token
    "$data.amount"         // amount
)

// NFT
Display.tokenAmountField(
    "$labels.nft",            // title
    "$labels.nftDesc",        // description
    "",                       // case (empty)
    "$msg.to",                // token
    "1",                      // amount
    "$data.tokenId",          // tokenId
    Display.Direction.Out     // direction
)
```

#### 4.5 Structural Formats

Structural formats carry a nested `fields` array and modify the rendering context for their children:

- **`map`**, **`array`** — create a new isolated `$data` scope. Nested fields resolve variable references against the
  new scope and do NOT inherit the parent's `$data`. `$msg` remains constant.
- **`call`** — creates a new `$msg` context derived from its params. The nested display is resolved as a completely
  independent rendering with its own `$msg` and `$data`.
- **`switch`** — does NOT create a new scope. Child fields inherit the parent's `$data` context as-is.

This scope isolation is a security feature: it prevents accidental variable leakage between parent and child contexts,
ensuring that nested fields can only access data explicitly passed to them through their params. Wallets MUST enforce
strict scope boundaries when rendering structural formats.

**`map`** — creates a new isolated `$data` scope and renders its nested `fields` within that scope. The new scope is
populated through two mechanisms, which may be used independently or combined:

- `$`-prefixed params: each `$<name>` parameter binds the resolved value into the child `$data` scope as `$data.<name>`.
- `abi` + `value` params: the bytes value referenced by `value` is ABI-decoded according to the Solidity type signature
  in `abi`, and all decoded fields are merged into the child `$data` scope as individual variables.

| Param     | Required | Description                                                                                                                                                                                         |
|-----------|----------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `$<name>` | no       | `$`-prefixed entries bind values into child `$data` scope (e.g. `$token` → `$data.token`)                                                                                                           |
| `abi`     | no       | Solidity type signature string (e.g. `"(address token,uint256 amount)"`) that ABI-decodes the bytes in `value` param and populates the child `$data` scope with the decoded fields as new variables |
| `value`   | no       | Reference resolving to bytes to be ABI-decoded using `abi` signature into child `$data` scope                                                                                                       |

```solidity
// Example 1: Using $-prefixed params to bind values
Display.mapField(
    "$labels.transfer",      // title
    "$labels.transferDesc",  // description
    "",                      // case (empty)
    abi.encodePacked(        // params
        Display.entry("$token", "$msg.to"),
        Display.entry("$amount", "$data.value")
    ),
    abi.encodePacked(        // fields
        Display.tokenAmountField(
            "$labels.amount",      // title
            "$labels.amountDesc",  // description
            "",                    // case (empty)
            "$data.token",         // token
            "$data.amount"         // amount
        )
    )
)

// Example 2: Using ABI decoding to populate child scope
Display.mapField(
    "$labels.order",          // title
    "$labels.orderDesc",      // description
    "",                       // case (empty)
    abi.encodePacked(         // params
        Display.entry("abi", "(address token,uint256 amount,uint256 deadline)"),
        Display.entry("value", "$data.orderData")
    ),
    abi.encodePacked(         // fields
        Display.tokenAmountField(
            "$labels.amount",      // title
            "",                    // description (empty)
            "",                    // case (empty)
            "$data.token",         // token (from ABI-decoded orderData)
            "$data.amount"         // amount (from ABI-decoded orderData)
        ),
        Display.datetimeField(
            "$labels.deadline",    // title
            "",                    // description (empty)
            "",                    // case (empty)
            "$data.deadline"       // deadline (from ABI-decoded orderData)
        )
    )
)
```

---

**`array`** — iterates over one or more parallel arrays and renders the nested `fields` once per element, creating a
fresh isolated `$data` scope for each iteration. All `$`-prefixed params MUST resolve to arrays of equal length; wallets
MUST halt rendering if array lengths differ. For each iteration index `i`, the wallet binds `array[i]` for each param
into the new `$data` scope. The nested `fields` are then rendered against this scope.

| Param     | Required           | Description                                                                            |
|-----------|--------------------|----------------------------------------------------------------------------------------|
| `$<name>` | yes (at least one) | Reference resolving to an array; each element is bound as `$data.<name>` per iteration |

```solidity
Display.arrayField(
    "$labels.transfers",      // title
    "$labels.transfersDesc",  // description
    "",                       // case (empty)
    abi.encodePacked(         // params
        Display.entry("$to", "$data.recipients"),
        Display.entry("$amount", "$data.amounts")
    ),
    abi.encodePacked(         // fields
        Display.addressField(
            "$labels.recipient",  // title
            "",                   // description (empty)
            "",                   // case (empty)
            "$data.to"            // value
        ),
        Display.tokenAmountField(
            "$labels.amount",  // title
            "",                // description (empty)
            "",                // case (empty)
            "$msg.to",         // token
            "$data.amount"     // amount
        )
    )
)
```

---

**`call`** — recursively resolves and renders the display specification for a nested contract call.

**Behavior:**

- Execution of the current display is paused
- A new `$msg` context is derived from the `to`, `value`, and `data` params:
    - `$msg.to` ← target contract address from `to` param
    - `$msg.value` ← native value from `value` param
    - `$msg.data` ← calldata from `data` param
    - `$msg.sender` ← the current contract address (propagated from parent context)
- The wallet matches `$msg.to` and the function selector against available display specifications
- The matched display specification is rendered
- Once complete, rendering of the parent display resumes

| Param   | Required | Description                                                       |
|---------|----------|-------------------------------------------------------------------|
| `to`    | yes      | Reference resolving to the target contract address                |
| `value` | yes      | Reference resolving to a `uint256` native value of the inner call |
| `data`  | yes      | Reference resolving to the calldata bytes of the inner call       |

```solidity
Display.callField(
    "$labels.innerCall",      // title
    "$labels.innerCallDesc",  // description
    "",                       // case (empty)
    "$data.to",               // to
    "$data.value",            // value
    "$data.data"              // data
)
```

---

**`switch`** — conditionally renders a subset of its nested `fields` based on a discriminant `value`. The `switch`
format does NOT create a new `$data` scope; child fields inherit the parent's `$data` context. For each child field, the
wallet evaluates its `case` array: if `case` is empty, the field is always rendered; if `case` is non-empty, the field
is rendered only if `value` matches at least one entry in the `case` array. Matching is performed via equality
comparison of the resolved values.

| Param   | Required | Description                                   |
|---------|----------|-----------------------------------------------|
| `value` | yes      | Reference or literal used as the discriminant |

```solidity
Display.switchField(
    "$labels.asset",      // title
    "$labels.assetDesc",  // description
    "",                   // case (empty)
    "$data.assetType",    // value
    abi.encodePacked(     // fields
        Display.tokenAmountField(
            "$labels.erc20Amount",                        // title
            "",                                           // description (empty)
            abi.encodePacked(keccak256(bytes("erc20"))), // case
            "$msg.to",                                    // token
            "$data.amount"                                // amount
        ),
        Display.tokenAmountField(
            "$labels.nft",                                 // title
            "",                                            // description (empty)
            abi.encodePacked(keccak256(bytes("erc721"))), // case
            "$msg.to",                                     // token
            "1",                                           // amount
            "$data.tokenId",                               // tokenId
            Display.Direction.None                         // direction
        )
    )
)
```

### 5. Localization

User-facing strings in `Display.title`, `Field.title`, and `Field.description` MAY use label references of the form
`$labels.key` to support internationalization.

#### 5.1 Label Resolution

When the wallet encounters a label reference:

1. **Locale matching** — the wallet selects the `Labels` bundle whose `locale` field best matches the wallet's active
   locale:
    - Exact match (e.g. `en-US`) is preferred
    - Language-only fallback (e.g. `en` for `en-US`) is attempted if no exact match
    - If no match is found, the wallet SHOULD fall back to the `en` locale
2. **Key lookup** — the wallet searches the selected bundle's `items` array for an `Entry` with `key` matching the
   reference
3. **Error handling** — if the key is not found in any bundle, rendering MUST halt with an error

#### 5.2 Labels Structure

A `Labels` bundle consists of:

- `locale` — a locale identifier string (e.g. `en`, `fr`, `en-US`, `zh-CN`)
- `items` — an array of `Entry` pairs where each `key` maps to a localized string `value`

The `Display` specification includes a `labels` array containing one or more `Labels` bundles, typically including at
least an `en` bundle as the default.

**Example:**

```solidity
Display.labels(
    "en",                  // locale
    abi.encodePacked(      // items
        Display.entry("title", "Transfer"),
        Display.entry("description", "Transfer ERC-20 tokens to another address"),
        Display.entry("recipient", "Recipient"),
        Display.entry("amount", "Amount")
    )
)
```

See Section 1.1 for a complete example showing multiple label references resolved from a single Labels bundle.

## Rationale

### Design Philosophy

This standard addresses the semantic gap between raw calldata and human-readable intent through three trustless
evaluation criteria:

1. **Local Interpretation** — wallets decode calldata locally without network dependencies
2. **Verifiability** — display specifications are uniquely identified via EIP-712 hash
3. **Censorship Resistance** — no gatekeepers control metadata access

The specification solves **calldata interpretation** ("What are you doing?") at the smart contract level, complementing
**address verification** ("Who are you?") handled by decentralized Contract Lists at the social layer.

Core design principles:

- **Semantic clarity over visual prescription** — defines what data means, not how it renders, ensuring device-agnostic
  validity
- **Security by default** — explicit declarations and halt-on-error behavior prevent incomplete or malicious
  specifications
- **Compact cryptographic identifier** — 32-byte EIP-712 hash enables verification on resource-constrained devices

The standard establishes **"display is law"**: what users see during transaction approval constitutes the authoritative
representation of contract behavior. Developers commit to display semantics cryptographically, creating a binding
contract between implementation and user-facing representation. This principle makes display specifications
security-critical artifacts requiring the same rigor as smart contract code.

### EIP-712 Display Identifier

The display identifier is computed using EIP-712 `hashStruct` rather than simpler hash schemes:

1. **Compact representation** — a 32-byte identifier is efficient for smart contract storage (bytecode or state) and
   enables resource-constrained devices to perform cryptographic verification. Display specifications can be embedded
   separately in contract metadata.
2. **Proven security and adoption** — EIP-712 is battle-tested across wallets, hardware devices, and libraries, reducing
   implementation risk and ensuring consistent verification across the ecosystem.
3. **Structured hashing** — recursive computation from typed components enables both static precomputation (at compile
   or deploy time) and dynamic on-chain computation (for factory contracts generating specifications at runtime). Both
   off-chain tools and on-chain contracts compute identical identifiers deterministically, enabling verification without
   external dependencies.

### Semantic vs Visual Separation

The specification defines *what* data means, not *how* it renders. This separation is fundamental:

**Specification:** Identifies data types and semantic meaning (e.g., "token amount", "deadline"), defines data
hierarchy, provides localized labels, specifies conditional visibility.

**Wallet:** Handles visual layout, icons, fiat conversions, balance context, and device-specific optimizations.

This ensures specifications remain valid across UI paradigms and devices. For example, `tokenAmount` semantically
identifies "an amount in a specific token"—a hardware wallet renders it as plain text with symbol, while a mobile wallet
may show an icon, USD value, and balance percentage. Both are correct interpretations of the same semantic
specification.

### Structural Format Types

Four structural format types (`map`, `array`, `switch`, `call`) enable representation of complex transaction
architectures essential for command-based protocols (batch executors, universal routers) and nested execution patterns:

- **`array`** — iterates over parallel arrays, creating an isolated scope per element
- **`switch`** — dispatches on discriminant values to render variant-specific fields
- **`map`** — decodes bytes via inline ABI decoding, extracting structured data into a new scope
- **`call`** — represents nested contract calls for account abstraction, multisigs, and delegation

The `map` format addresses a common pattern where complex types are encoded as bytes in function parameters. Without
inline ABI decoding, developers would flatten structures into top-level parameters, creating unwieldy signatures.
Universal router specifications compose these formats: `array` iterates commands → `switch` selects command type → `map`
decodes parameters → nested `switch` handles conditional fields. This composition enables clear display of complex
transaction flows while maintaining scope isolation.

### Scope Isolation and Security

Structural formats create isolated `$data` scopes for their nested fields to improve readability and security:

- **`map`, `array`** — create new isolated scopes. Child fields access only data explicitly passed through `params`.
- **`switch`** — does NOT create new scope; used for conditional rendering within the same context.
- **`call`** — creates entirely new rendering context with its own `$msg` and `$data`, mirroring contract call
  semantics.

Scope isolation serves three purposes: (1) prevents variable shadowing attacks from malicious specifications, (2) makes
data flow auditable by reviewers examining `params`, and (3) improves specification readability by making data
dependencies explicit.

Wallets MUST enforce strict scope boundaries. Cross-scope variable access violates the security model and may enable
display specification attacks.

### Format Type System Design

The format type system is designed for forward compatibility. Field parameters use generic `Entry` key-value pairs
rather than specialized structs, allowing new format types to be added without modifying the core EIP-712 type
definitions:

```solidity
Field(string title, string description, string format, string[] case, Entry[] params, Field[] fields)
```

The `format` field is a string, enabling new semantic types (e.g., `ipfsHash`, `ens`, `coordinate`) to be defined in
future proposals without breaking existing implementations. Wallets MUST reject unknown format types to prevent
downgrade attacks where unsupported semantics are silently ignored.

Formats are organized into three categories: **Raw types** (minimal transformation, direct Solidity correspondence), *
*Rich formats** (semantic interpretation like `datetime`, `percentage`), and **Structural formats** (context
modification with nested fields).

### Localization Architecture

Internationalization is essential for human-readable transaction interpretation — users must understand what a
transaction does in their native language to make informed decisions. Labels are included in the display specification (
and thus the display identifier hash) because localized strings carry the same security guarantees as field definitions:
they must be authenticated and cannot be tampered with.

**Security and authenticity:**

- All labels are part of the EIP-712 display identifier, cryptographically binding translations to the specification
- Adding or updating translations requires recomputing the display identifier (equivalent to a specification update; for
  immutable contracts, this may require contract redeployment)
- Prevents malicious label substitution attacks

**Bundle-based design:**

- Separates translatable strings from field definitions, preventing format specifications from being polluted with
  locale data
- Enables sharing common labels across multiple display specifications
- Fallback mechanism (exact locale → language-only → `en` default) ensures accessibility
- Missing label keys halt rendering, preventing corrupted displays from incomplete translations

### Error Handling Strategy

The specification adopts a **halt-on-error** philosophy: any resolution failure, type mismatch, verification failure, or
missing key halts rendering immediately. This prevents misleading displays — partial information is more dangerous than
no information when users make irreversible transaction decisions.

**Critical failure scenarios:**

- Variable reference to non-existent path or out-of-bounds array index
- Type coercion failure or unverified token/contract address
- Missing label key or array length mismatch in structural formats

Lenient error handling (showing empty strings or defaults) creates security risks: a bypassed token verification could
silently display an unverified address, leading users to approve malicious transactions. The trade-off is that
specifications must be complete and correct, appropriate for a security-critical standard where user funds are at stake.

## Backwards Compatibility

This standard introduces a new display metadata format and does not modify existing Ethereum ABI encoding or transaction
structures. Contracts without display specifications continue to function normally; wallets fall back to existing
transaction display methods (raw calldata, function selector matching, or proprietary metadata systems).

The EIP-712 type system used for display identifier computation is stable and widely deployed. Future extensions to the
format type system use string-based format identifiers, ensuring forward compatibility: wallets MUST reject unknown
format types rather than attempting to render them.

## Security Considerations

### Binding Display Specifications to Contracts

The display identifier uniquely identifies a display specification via its EIP-712 hash, but this specification does NOT
define how that identifier is bound to a smart contract. The binding mechanism — how wallets cryptographically verify
that a given display specification is the authoritative one for a specific contract and function — is addressed in a
separate companion EIP.

Without on-chain verification, users face significant security risks:

- **Specification substitution attacks** — malicious actors could present fraudulent display specifications that pass
  wallet validation but misrepresent transaction behavior
- **Phishing via trusted contracts** — attackers could create malicious contracts with stolen display specifications
  from legitimate protocols, inheriting user trust without authorization
- **Downgrade attacks** — outdated or vulnerable specifications could be substituted for current ones

The companion EIP defines mechanisms for on-chain display identifier verification, ensuring that wallets can trustlessly
determine the correct display specification for any given contract call. Wallet implementations MUST NOT display
transactions based on unverified specifications.

### Native Value Transfer Omission

A display specification may define a payable function that accepts non-zero `msg.value` but omit any field displaying
the native token transfer to the user. This allows malicious or incomplete specifications to hide value transfers.

When `msg.value > 0` but the display specification omits this information, wallet implementations MUST display a
prominent warning about the undisclosed native transfer. Users are responsible for rejecting such transactions if they
do not expect or don't see the native transfer. Alternatively, the wallet should include a synthetic native transfer
field,
which may lead to duplication.

### Developer Responsibilities

Display specifications are security-critical artifacts. Malicious developers can create display specifications that
misrepresent contract behavior, mislead users through incorrect labels or field mappings, or omit critical parameters.

This threat is outside the scope of this standard. The standard assumes users interact with contracts from developers
and protocols they trust. One mitigation approach is for wallets to restrict transactions to contracts verified via
Contract Lists, ensuring only reputable contracts are called.

Developers adopting this standard MUST ensure display specifications accurately represent contract behavior and SHOULD
subject them to the same security review process as smart contract code.

### Denial of Service (DoS)

Malicious display specifications can exhaust wallet resources through excessive recursion depth (`call` format), large
array iterations (`array` format), deeply nested structural formats, or oversized label bundles.

Wallet implementations MUST enforce rational limits on recursion depth, array sizes, and computational complexity
appropriate to their platform constraints. Rendering MUST halt when limits are exceeded.
