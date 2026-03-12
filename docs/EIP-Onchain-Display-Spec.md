---
eip: TBD
title: Onchain Display Specification
description: A standardized on-chain display format for rendering human-readable transaction context from smart contract calldata.
author: TBD
discussions-to: TBD
status: Draft
type: Standards Track
category: ERC
created: 2026-03-10
requires: 712, TBD (Onchain Display Verification)
---

## Table of Contents

- [Abstract](#abstract)
- [Motivation](#motivation)
- [Specification](#specification)
    - [Type Definitions](#type-definitions)
    - [Function Signature Format](#function-signature-format)
    - [Variable References](#variable-references)
    - [Rendering](#rendering)
    - [Field Formats](#field-formats)
    - [Localization](#localization)
- [Rationale](#rationale)
- [Backwards Compatibility](#backwards-compatibility)
- [Security Considerations](#security-considerations)
- [Copyright](#copyright)

## Abstract

This standard defines a structured display specification for smart contract functions, associating ABI-decoded calldata parameters with semantic display fields covering types such as token amounts, date and time values, percentages, and addresses. Each display specification is uniquely identified by a 32-byte digest computed as an EIP-712 structured data hash. This compact identifier enables resource-constrained devices to deterministically compute and verify the integrity of a specification without network access. The standard specifies the type system, format rules, and identifier computation process; a companion standard defines the on-chain verification mechanisms that bind these identifiers to smart contracts.

## Motivation

The Ethereum ABI encodes function call parameters as typed byte sequences but carries no semantic meaning: a Unix timestamp and a token amount are indistinguishable representations of `uint256`, and a `bytes` parameter encoding a token transfer is structurally identical to one encoding delegated execution. This absence of machine-parseable semantics produces blind signing — users authorize transactions whose effects they cannot independently verify, relying entirely on the originating application interface to describe what they are approving. This trust model is incompatible with the security properties expected of self-custodial wallets and hardware signing devices, where the integrity of displayed information must be verifiable independent of any connected software.

Hardware signing devices are the most constrained signing environment: limited memory and no network connectivity preclude fetching or validating external metadata at signing time. A standard that works within these constraints works everywhere — software wallets on web and mobile inherit the same guarantees while being free to present richer context on top.

A viable solution requires two complementary properties: an expressive type system covering the semantic patterns common in deployed contracts—token amounts, timestamps, durations, percentages, and addresses—with support for structural composition of nested contract calls; and a compact identifier derivable from a complete display specification by any device without network access. This standard defines the type system and specifies identifier computation using EIP-712 structured data hashing. A companion standard defines the on-chain mechanisms by which these identifiers are bound to deployed contracts.

## Specification

The keywords "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119 and RFC 8174.

This specification defines semantic presentation: what data represents and how it relates to other data. Visual rendering is explicitly out of scope; wallets MAY adapt the presentation to device constraints and capabilities while preserving the specified semantics.

### Type Definitions

A display specification is composed of four EIP-712 compatible structs: `Display`, `Field`, `Labels`, and `Entry`. The **display identifier** is the 32-byte value produced by `hashStruct(Display)` as defined in EIP-712; it uniquely identifies a complete display specification and is the value registered on-chain by the companion verification standard. Implementations MUST use the type strings below verbatim, as any deviation produces a different identifier.

**`Display`** — root type for one function's display specification.

- `abi` — Solidity function signature for selector matching
- `title` — label reference or literal string for transaction title
- `description` — label reference or literal string for human-readable operation description
- `fields` — ordered array of `Field` definitions
- `labels` — array of `Labels` bundles

**`Field`** — single display item definition.

- `title` — label reference or literal for field name
- `description` — label reference or literal string for field description; an empty string indicates no description
- `format` — display format identifier
- `case` — array of discriminant values for conditional rendering; an empty array indicates unconditional rendering
- `params` — `Entry` array supplying formatter arguments (format-specific keys, variable references, or literals)
- `fields` — nested `Field` definitions for structural formats

**`Labels`** — locale-specific string bundle.

- `locale` — locale identifier (e.g., `en`, `fr`)
- `items` — `Entry` array mapping label keys to translated strings

**`Entry`** — generic key-value pair.

- `key` — string identifier
- `value` — string value; interpreted as a variable reference or literal depending on context

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

#### Example: ERC-20 Transfer

The following shows a complete display identifier computation for the ERC-20 `transfer` function using the `Display` library:

```solidity
bytes32 constant TRANSFER_DISPLAY_HASH = Display.display(
    "transfer(address to, uint256 amount)",  // abi
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

### Function Signature Format

The `Display.abi` field MUST be a function signature of the form `<name>(<type> <name>, ...) [<modifier>]`.

Each parameter MUST include a type and SHOULD include a name. Named parameters are accessible via `$data.<name>` (e.g., `$data.amount`); unnamed parameters are accessible only by zero-based positional index via `$data.<index>` (e.g., `$data.0`). The optional state mutability modifier MUST be `pure`, `view`, `payable`, or `nonpayable` (default if omitted). Wallets MUST reject calls where `msg.value > 0` and the modifier is not `payable`.

The following are valid function signatures:

```
transfer(address to, uint256 amount)
approve(address spender, uint256 amount) nonpayable
deposit() payable
```

The function selector for matching derives from the canonical ABI signature (types only, names and modifier stripped) via `keccak256`.

### Variable References

String fields throughout a display specification — including `Display.title`, `Display.description`, `Field.title`, `Field.description`, `Field.params` values, and `Field.case` entries — are resolved at render time as either a **literal** (a static string constant) or a **variable reference** (a `$`-prefixed path that resolves to a runtime value). In both cases the resolved value is then **type-cast** to the type expected by the consuming formatter or comparator. Resolution MUST halt if the cast fails.

#### Literals

A literal is any `Entry` value that does not start with `$`. Literals are untyped strings coerced to the target type at render time per the following rules:

| Target type | Coercion rule                                                        |
|-------------|----------------------------------------------------------------------|
| `bool`      | `"true"` or `"false"` (case-insensitive). Any other value MUST halt. |
| `uint`      | Decimal integer string parseable as `uint256`.                       |
| `int`       | Decimal integer string parseable as `int256`.                        |
| `address`   | Hex string parseable as a 20-byte address.                           |
| `bytes`     | Hex string with optional `0x` prefix.                                |
| `string`    | Used as-is.                                                          |

If the literal cannot be coerced to the required type, resolution MUST halt.

#### Variable References

A variable reference is any `Entry` value that starts with `$`. References are resolved at render time by looking up the path in the appropriate container. The resolved value retains its ABI type from the decoded calldata or transaction context; type casting is applied subsequently as described in [Type Casting](#type-casting).

#### Reference Containers

**`$labels`** — localized string bundle selected for the current locale. This container is read-only and constant for the duration of rendering. Only one level of property access is permitted. `$labels.<key>` resolves to the string value associated with `key` in the active `Labels` bundle. Rendering MUST halt if the key is not found. Full locale selection and fallback rules are defined in [Localization](#localization). `$labels` references are valid in `title` and `description` fields of `Display` and `Field`, as well as in `Field.params` values.

**`$msg`** — transaction context. This container is read-only and constant for the duration of rendering. Only one level of property access is permitted; nested access is not supported.
- `$msg.sender` — caller address
- `$msg.to` — contract receiving the call
- `$msg.value` — native value (`uint256`)
- `$msg.data` — raw calldata bytes

**`$data`** — decoded function arguments for the current rendering scope. At the top level, arguments are decoded from `$msg.data` per `Display.abi`. The `map` and `array` structural formats create a new isolated `$data` scope; nested fields within those formats do not inherit the parent `$data`. The `switch` format does not create a new scope. Access patterns:
- Named: `$data.amount`, `$data.to`
- Positional: `$data.0`, `$data.1`
- Nested: `$data.order.token`
- Array index: `$data.items[0]`, `$data.items[-1]` (negative from end)
- Slice: `$data.items[1:3]`, `$data.data[:]`

#### Type Casting

After a value is resolved — whether from a literal or a variable reference — it is cast to the type expected by the consuming formatter parameter. For variable references, the resolved ABI type MUST be compatible with the expected type; if it is not, resolution MUST halt. For literals, the coercion rules in the [Literals](#literals) table apply.

For `switch` case matching, each entry in `Field.case` is cast to the type of the resolved `switch` `value` parameter before comparison. For example, if `value` resolves to a `bytes32`, each `case` entry is coerced from its hex string representation to `bytes32`; if `value` resolves to a `uint256`, each `case` entry is parsed as a decimal integer. Comparison is performed as equality after casting. Resolution MUST halt if any `case` entry cannot be cast to the type of `value`.

#### Resolution Failure

Resolution MUST halt if:

- The container is unknown.
- The referenced path does not exist in the current `$data` scope.
- An array index is out of bounds.
- The resolved value type is incompatible with what the formatter requires.
- A literal cannot be coerced to the required type.
- A `case` entry cannot be cast to the type of the `switch` `value` parameter.

### Rendering

A wallet renders a display specification by executing the following steps in order. Any failure at any step MUST halt rendering.

**Step 1 — Context Initialization**

Initialize the two rendering contexts:

- `$msg` is populated from the transaction envelope: `sender`, `to`, `value`, and `data`. This context is read-only and constant for the entire top-level rendering scope.
- `$data` is initialized by ABI-decoding `$msg.data` per `Display.abi`. Named parameters are accessible by name (e.g., `$data.amount`); unnamed parameters are accessible by zero-based positional index (e.g., `$data.0`).

**Step 2 — Native Value Check**

If `$msg.value > 0`, the wallet MUST display a warning to the user indicating that native value is being transferred.

**Step 3 — Field Iteration**

Iterate `Display.fields` in declaration order. For each `Field`:

1. **Conditional visibility**: If `Field.case` is non-empty, the field is rendered only if the enclosing `switch` value matches at least one `case` entry after type casting; otherwise the field is skipped. Fields with an empty `case` are always rendered.
2. **Reference resolution**: Resolve all `title`, `description`, and `params` values per [Variable References](#variable-references).
3. **Formatting**: Cast and format the resolved parameter values per the rules of `Field.format` defined in [Field Formats](#field-formats).
4. **Structural recursion**: For structural formats (`map`, `array`, `switch`, `call`), process nested `fields` with the scope rules specified for each format in [Field Formats](#field-formats).

**Step 4 — Nested Call Processing**

When a `call` field is encountered during step 3, the wallet enters a recursive rendering context:

- A new `$msg` is constructed from the `call` field's `to`, `value`, and `data` parameters.
- `$msg.sender` of the inner context is set to the parent `$msg.to` (the contract making the inner call).
- A display specification matching the inner call's function selector is located via the mechanism defined in the companion verification standard and rendered recursively from step 1.
- Wallets MUST enforce a maximum recursion depth. Rendering MUST halt if the limit is exceeded.

### Field Formats

#### Raw Solidity Types

These formats display values as-is with minimal transformation, corresponding directly to their Solidity types. All accept a single `value` parameter.

| Format    | Solidity type     | Display                                                                                                             |
|-----------|-------------------|---------------------------------------------------------------------------------------------------------------------|
| `boolean` | `bool`            | Wallet-localized yes/no string                                                                                      |
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

#### Rich Formats

These formats interpret a raw Solidity value into a human-readable semantic representation.

**`datetime`** — displays a Unix timestamp as an absolute, locale-formatted date and time. Accepts any `uintN` type. An optional `units` parameter specifies the unit of the input value; if omitted, seconds are assumed.

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

**`duration`** — displays a relative time span as a human-readable duration (e.g. "2 weeks", "3 days"). Accepts any `uintN` type. An optional `units` parameter specifies the unit of the input value; if omitted, seconds are assumed.

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

**`percentage`** — displays a rational percentage computed as `value / basis`. Both parameters are interpreted as unsigned integers; rendering MUST halt if `basis` is zero.

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

**`bitmask`** — displays an unsigned integer as a list of labels corresponding to each set bit. Bit labels are supplied as additional parameters using `#N` keys where `N` is the zero-based bit index. Only labels for set bits are shown.

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

**`units`** — displays an unsigned integer scaled by a decimal exponent (e.g. USDC with 6 decimals).

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

#### Address Formats

Three address formats are defined, each with different verification requirements. `address` performs best-effort name resolution and is informational. `token` and `contract` require verification against a Token List and Contract List respectively; rendering MUST halt if the address is not found in the applicable list. These lists are defined in the companion verification standard. Developers MUST choose the semantically appropriate format.

**`address`** — 20-byte address with best-effort name resolution (local contacts, ENS). Use when identity is informational, not a security precondition.

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

**`token`** — token address verified against the Token List defined in the companion verification standard; rendering MUST halt if the address is not found. Use when token identity is critical to transaction assessment. An optional `tokenId` parameter enables display of non-fungible token identities.

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

**`contract`** — contract address verified against the Contract List defined in the companion verification standard; rendering MUST halt if the address is not found. Use when the contract receives delegated authority or executes on the user's behalf (e.g., the spender in an ERC-20 `approve` call).

| Param   | Required | Description                               |
|---------|----------|-------------------------------------------|
| `value` | yes      | Reference resolving to a contract address |

```solidity
// Full display spec for ERC-20 approve
bytes32 constant APPROVE_DISPLAY_HASH = Display.display(
    "approve(address spender, uint256 amount) nonpayable",  // abi
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

#### Value Formats

**`nativeAmount`** — displays a native currency amount (e.g. ETH). An optional `direction` indicates whether the amount
flows `in` or `out` relative to the user. When `direction` is omitted, the amount is displayed without directional
indication.

| Param       | Required | Description                                                       |
|-------------|----------|-------------------------------------------------------------------|
| `amount`    | yes      | Reference resolving to a `uintN` in the smallest denomination (e.g. wei for ETH) |
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

**`tokenAmount`** — displays a token amount denominated in a specific token, resolved against a Token List (token verification requirements are defined in the companion verification standard). An optional `tokenId` parameter enables display of non-fungible token amounts. When `direction` is omitted, the amount is displayed without directional indication.

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

#### Structural Formats

Structural formats carry nested `fields` and modify rendering context:
- **`map`, `array`** — create isolated `$data` scope (nested fields access only explicitly passed data; `$msg` constant)
- **`call`** — creates new `$msg` context (independent rendering with own `$msg` and `$data`)
- **`switch`** — no new scope (child fields inherit parent's `$data`)

Wallets MUST enforce strict scope boundaries.

**`map`** — creates isolated `$data` scope, renders nested `fields`. Scope populated via:
- `$<name>` parameters: bind values to child scope (`$token` → `$data.token`)
- `abi` + `value`: ABI-decode bytes, merge fields into child scope

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

**`array`** — iterates parallel arrays, renders nested `fields` per element with fresh isolated `$data` scope. All `$`-prefixed parameters MUST be equal-length arrays; rendering MUST halt if lengths differ. Binds `array[i]` for each parameter per iteration.

Each `$<name>` parameter MUST resolve to one of the following iterable types:

| Input type  | Element type per iteration |
|-------------|----------------------------|
| `T[]`       | `T` (element of dynamic array) |
| `T[N]`      | `T` (element of fixed-size array) |
| `bytes`     | `bytes1` (single byte) |

Resolution MUST halt if a parameter resolves to a non-iterable type.

| Param     | Required           | Description                                                                            |
|-----------|--------------------|----------------------------------------------------------------------------------------|
| `$<name>` | yes (at least one) | Reference resolving to an iterable type; each element is bound as `$data.<name>` per iteration |

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

**`call`** — renders nested contract call display specification. Creates new `$msg` from `to`, `value`, `data` parameters; `$msg.sender` of the inner context is set to the parent `$msg.to`. Wallet matches specification, renders, then resumes parent rendering.

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

**`switch`** — conditionally renders nested `fields` based on discriminant `value`. No new scope; child fields inherit parent's `$data`. Empty `case` array: always render. Non-empty `case`: render only if `value` matches at least one entry (equality comparison).

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

Wallets MUST reject any `Field` whose `format` value is not defined in this standard or a recognized extension. This prevents downgrade attacks via unrecognised format identifiers.

### Localization

User-facing strings in `title` and `description` fields MAY use `$labels.<key>` references for internationalization. Specifications that omit localization MAY use literal strings directly.

#### Label Resolution

The wallet selects the active `Labels` bundle from `Display.labels` using the following priority order:

1. Exact locale match (e.g., `en-US` matches `en-US`).
2. Language-only fallback (e.g., `en-US` falls back to `en`).
3. Default to the `en` bundle if present.

If no matching bundle is found, rendering MUST halt. Once a bundle is selected, the wallet searches its `items` array for an `Entry` whose `key` matches the reference key. Rendering MUST halt if the key is not found in the selected bundle.

#### Example

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


## Rationale

### Design Goals

The specification addresses calldata interpretation through local decoding without network dependencies, verifiable display identifiers via EIP-712, and censorship-resistant metadata access. The core principle is "display is law": display specifications are security-critical artifacts that cryptographically commit developers to the semantics shown to users.

### EIP-712 Display Identifier

The EIP-712 `hashStruct` provides a compact, 32-byte identifier compatible with established ecosystem infrastructure and resource-constrained devices. This mechanism enables deterministic verification through both static precomputation and dynamic, on-chain generation.

The sequential accumulation of `keccak256` operations within a `hashStruct` facilitates a memory-efficient processing model. A wallet can theoretically stream field definitions sequentially: interpreting a field's value for the user, updating a running hash accumulator, and then discarding the associated metadata. This approach avoids the requirement to buffer the entire display specification, making the design viable for memory-constrained hardware signing devices. While this streaming model assumes that the underlying calldata is accessible for random access during field resolution, it significantly reduces the peak memory overhead for the display logic itself.

Adopting EIP-712 for identifier computation means that improvements to the EIP-712 algorithm and its Solidity tooling accrue to this standard without requiring specification changes. Currently, display identifiers must be expressed as nested `keccak256(abi.encode(...))` chains — correct but verbose. Proposed Solidity compiler enhancements, including native `type(S).typehash` and `type(S).hashStruct(s)` support, will allow these to be replaced with direct type-level expressions evaluated at compile time. Solidity does not yet support compile-time constant evaluation (`constexpr`); as this capability is added to the compiler, display identifier constants will be expressible as simple compiler-verified declarations rather than manually assembled hash computations, further reducing boilerplate and eliminating a class of encoding errors.

### Semantic vs Visual Separation

The specification defines semantic meaning and data hierarchy, not visual presentation. This separation ensures specifications remain valid across different wallet implementations and device form factors while allowing wallets to optimize rendering for their specific constraints.

### Structural Formats

Structural formats (`map`, `array`, `switch`, `call`) enable display specifications to cover transaction patterns that cannot be expressed as flat field listings. `map` enables typed ABI decoding of `bytes`-encoded sub-parameters, allowing nested structured data to be accessed by field name rather than extracted via unsafe raw byte offset arithmetic. `array` handles homogeneous repetition across batched transfers and multicall sequences without requiring per-element format duplication. `switch` supports command-indexed dispatch, covering protocols that multiplex multiple operations through a single entry point — such as universal routers — without requiring a separate display specification per command variant. `call` handles dynamically constructed calls where the target address and calldata are themselves ABI-encoded parameters, the canonical pattern in smart contract accounts, multisigs, and DAOs; wallets that implement `call` can render any account abstraction contract without per-contract special-casing in firmware. 

### Scope Isolation

`map` and `array` create isolated `$data` scopes; child fields access only explicitly passed parameters. `switch` does not create new scopes. `call` creates entirely new rendering contexts. Scope isolation prevents variable shadowing attacks, makes data flow auditable, and improves specification readability.

### Forward Compatibility

Field parameters use generic `Entry` key-value pairs and string-based format identifiers, allowing new semantic types to be added in future proposals without modifying core EIP-712 type definitions. Rejection of unknown format identifiers, enforced in the Specification, prevents downgrade attacks.

### Localization

Labels are included in the display identifier hash to prevent tampering and ensure translations carry the same cryptographic guarantees as field definitions. The bundle-based design separates translatable strings from format specifications and provides locale fallback mechanisms. Missing label keys halt rendering to prevent corrupted displays.

### Error Handling

The specification adopts halt-on-error behavior: any resolution failure, type mismatch, verification failure, or missing key halts rendering immediately. This prevents misleading displays where partial information could lead users to approve malicious transactions. Specifications must be complete and correct.

## Backwards Compatibility

This EIP introduces a new standard and does not modify any existing Ethereum protocol, ABI encoding, or ERC. It has no backward compatibility requirements with respect to previously deployed contracts or existing wallet implementations. Wallets that do not implement this standard continue to operate under existing behavior; this standard defines an opt-in display layer.

Dependency on EIP-712 is additive: this standard reuses `hashStruct` solely for identifier computation and does not alter any EIP-712 behavior or interfere with existing EIP-712 signed data flows.

## Security Considerations

### Binding Display Specifications to Contracts

This specification does not define how display identifiers bind to contracts. A companion standard addresses on-chain verification mechanisms.

Without verification, users face specification substitution attacks, phishing via stolen specifications, and downgrade attacks. Wallet implementations MUST NOT display transactions based on unverified specifications.

### Native Value Transfer Omission

Payable functions accepting `msg.value > 0` may omit native transfer display fields, hiding value transfers. Wallet implementations MUST display a prominent warning.

### Developer Responsibilities

Malicious developers can create specifications that misrepresent behavior, mislead via incorrect labels, or omit critical parameters. This threat is outside this standard's scope; users rely on product trust. Wallets MAY mitigate this risk by restricting display to Contract List-verified contracts.

Developers MUST ensure specifications accurately represent behavior and SHOULD apply security review processes equivalent to smart contract code.

### Denial of Service

Malicious specifications can exhaust wallet resources via excessive recursion (`call`), large arrays (`array`), deep nesting, or oversized labels. Wallet implementations MUST enforce platform-appropriate limits on recursion depth, array sizes, and computational complexity. Rendering MUST halt when limits are exceeded.

## Copyright

Copyright and related rights waived via [CC0](https://creativecommons.org/publicdomain/zero/1.0/).
