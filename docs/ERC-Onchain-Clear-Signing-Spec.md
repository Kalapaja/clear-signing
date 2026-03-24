---
eip: TBD
title: Onchain Clear Signing Specification
description: A standardized on-chain clear signing format for rendering human-readable transaction context from smart contract calldata.
author: TBD (At least one author must include GitHub username)
discussions-to: TBD (Ethereum Magicians forum URL required)
status: Draft
type: Standards Track
category: ERC
created: 2026-03-10
requires: 712
---

## Table of Contents

- [Abstract](#abstract)
- [Motivation](#motivation)
- [Specification](#specification)
    - [Type Definitions](#type-definitions)
    - [Display Identifier](#display-identifier)
    - [Function Signature Format](#function-signature-format)
    - [Variable References](#variable-references)
    - [Rendering](#rendering)
    - [Field Formats](#field-formats)
        - [Raw Solidity Types](#raw-solidity-types)
        - [Rich Formats](#rich-formats)
        - [Address Formats](#address-formats)
        - [Value Formats](#value-formats)
        - [Structural Formats](#structural-formats)
    - [Localization](#localization)
    - [Contract Lists](#contract-lists)
    - [JSON Representation](#json-representation)
- [Rationale](#rationale)
- [Backwards Compatibility](#backwards-compatibility)
- [Security Considerations](#security-considerations)
- [Copyright](#copyright)

## Abstract

This standard defines a structured display specification for smart contract functions, associating ABI-decoded calldata parameters with semantic display fields covering types such as token amounts, date and time values, percentages, and addresses. Each display specification is uniquely identified by a 32-byte digest computed as an EIP-712 structured data hash. This compact identifier enables resource-constrained devices to deterministically compute and verify the integrity of a specification without network access.

## Motivation

The Ethereum ABI encodes function call parameters as typed byte sequences but carries no semantic meaning: a Unix timestamp and a token amount are indistinguishable representations of `uint256`, and a `bytes` parameter encoding an inner token transfer is displayed as an opaque hex string. This absence of machine-parseable semantics produces blind signing — users authorize transactions whose effects they cannot independently verify, relying entirely on the originating application interface to describe what they are approving. This trust model is incompatible with the security properties expected of self-custodial wallets and hardware signing devices, where the integrity of displayed information cannot be delegated to connected software.

Hardware signing devices are the most constrained signing environment: limited memory and no network connectivity preclude fetching or validating external metadata at signing time. Existing off-chain metadata registries require live network access and provide no cryptographic binding between displayed metadata and the contract being called — absent such a link, the description presented to the signer carries no protocol-level guarantee. This standard operates within hardware constraints and binds display to execution by construction, providing uniform integrity guarantees across all wallet environments.

This standard defines an expressive semantic type system — covering token amounts, timestamps, durations, percentages, addresses, and nested calls — and a compact, deterministic display identifier that any device can derive locally from a complete specification without network access. The companion Onchain Clear Signing Verification standard (EIP-TBD) defines how display identifiers are committed to by contracts at deployment and verified on every call, making the displayed specification an enforced precondition of execution.

## Specification

The keywords "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119 and RFC 8174.

### Type Definitions

A display specification is composed of four EIP-712 compatible structs: `Display`, `Field`, `Labels`, and `Entry`. Implementations MUST use the type strings below verbatim — any deviation produces a different display identifier.

**`Display`** — root type for one function's display specification.

- `abi` — Solidity function signature for selector matching, inspired by the ethers.js Human-Readable ABI format
- `title` — label reference or literal string for transaction title
- `description` — label reference or literal string for human-readable operation description
- `fields` — ordered array of `Field` definitions
- `labels` — array of `Labels` bundles

```json
{
  "Display": [
    {"name": "abi", "type": "string"},
    {"name": "title", "type": "string"},
    {"name": "description", "type": "string"},
    {"name": "fields", "type": "Field[]"},
    {"name": "labels", "type": "Labels[]"}
  ]
}
```

**`Field`** — single display item definition.

- `title` — label reference or literal for field name
- `description` — label reference or literal string for field description; an empty string indicates no description
- `format` — display format identifier
- `case` — array of discriminant values for conditional rendering; an empty array indicates unconditional rendering
- `params` — `Entry` array supplying formatter arguments (format-specific keys, variable references, or literals)
- `fields` — nested `Field` definitions for structural formats

```json
{
  "Field": [
    {"name": "title", "type": "string"},
    {"name": "description", "type": "string"},
    {"name": "format", "type": "string"},
    {"name": "case", "type": "string[]"},
    {"name": "params", "type": "Entry[]"},
    {"name": "fields", "type": "Field[]"}
  ]
}
```

**`Labels`** — locale-specific string bundle.

- `locale` — locale identifier (e.g., `en`, `fr`)
- `items` — `Entry` array mapping label keys to translated strings

```json
{
  "Labels": [
    {"name": "locale", "type": "string"},
    {"name": "items", "type": "Entry[]"}
  ]
}
```

**`Entry`** — generic key-value pair.

- `key` — string identifier
- `value` — string value; interpreted as a variable reference or literal depending on context

```json
{
  "Entry": [
    {"name": "key", "type": "string"},
    {"name": "value", "type": "string"}
  ]
}
```

#### Example: ERC-20 Transfer

The following shows a complete display identifier computation for the ERC-20 `transfer` function. It uses the `Display` helper library, which encapsulates the EIP-712 `hashStruct` encoding — `Display.display(...)` is equivalent to calling `keccak256(abi.encode(DISPLAY_TH, ...))` with required params:

```solidity
bytes32 constant TRANSFER_ID = Display.display(
    "transfer(address to, uint256 amount)",  // abi
    "$labels.title",               // title — resolved from the labels bundle defined below
    "$labels.description",         // description — resolved from the labels bundle defined below
    abi.encodePacked(              // fields
        Display.addressField(
            "$labels.sender",      // title
            "$labels.senderDesc",  // description
            "",                    // case (empty)
            "$msg.sender"          // value — call context
        ),
        Display.tokenAmountField(
            "$labels.amount",      // title
            "$labels.amountDesc",  // description
            "",                    // case (empty)
            "$msg.to",             // token
            "$args.amount"         // amount — decoded from calldata per the abi signature above
        ),
        Display.addressField(
            "$labels.recipient",      // title
            "$labels.recipientDesc",  // description
            "",                       // case (empty)
            "$args.to"                // value
        )
    ),
    abi.encodePacked(              // labels
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

### Display Identifier

The **display identifier** is the 32-byte value produced by `hashStruct(Display)` as defined in EIP-712, computed from a complete display specification. Because `hashStruct` is deterministic and collision-resistant, the identifier uniquely commits to every field, format, label, and parameter in the specification. Any modification — including whitespace in the `abi` string or reordering of `labels` entries — produces a different identifier.

### Function Signature Format

The `Display.abi` field MUST be a function signature of the form `<name>(<type> <name>, ...) [<modifier>]`.

Each parameter MUST include a type and SHOULD include a name. Named parameters are accessible via `$args.<name>` (e.g., `$args.amount`); unnamed parameters are accessible by zero-based positional index via `$args.<index>` (e.g., `$args.0`). The optional state mutability modifier MUST be `payable` or `nonpayable` (default if omitted). Wallets MUST reject calls where `msg.value > 0` and the modifier is not `payable`.

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

A variable reference is any string value that starts with `$`. References can appear in `Entry` values (e.g., in `Field.params`) and in `Field.case` arrays. References are resolved at render time by looking up the path in the appropriate container. The resolved value retains its ABI type from the decoded calldata or transaction context; type casting is applied subsequently as described in [Type Casting](#type-casting).

#### Reference Containers

**`$msg`** — transaction context. This container is read-only and constant for the duration of rendering. Only one level of property access is permitted; nested access is not supported.
- `$msg.sender` — transaction originator (the signing account — EOA or smart account — at the top-level call; set to the parent `$msg.to` for nested `call` contexts)
- `$msg.to` — contract receiving the call
- `$msg.value` — native value (`uint256`)
- `$msg.data` — raw calldata bytes

**`$args`** — decoded function arguments for the current rendering scope. At the top level, arguments are decoded from `$msg.data` per `Display.abi`.

Access patterns:
- **Named**: `$args.amount`, `$args.to` — access parameters by their name from the function signature
- **Positional**: `$args.0`, `$args.1` — access parameters by zero-based index when names are not provided
- **Nested**: `$args.order.token` — access nested struct fields
- **Array index**: `$args.items[0]`, `$args.items[-1]` — access array or bytes element (negative indices count from end)
- **Slice**: `$args.items[1:3]`, `$args.data[:]` — extract array or bytes subranges

Scope behavior:
- **`map` and `array` formats**: Create a new isolated `$args` scope populated from their `params`. Nested fields within these formats do not inherit the parent `$args`.
- **`switch` format**: Does not create a new scope; child fields inherit the parent `$args`.
- **`call` format**: Creates an entirely new rendering context with its own independent `$msg` and `$args`.

**`$labels`** — localized string bundle selected for the current locale. This container is read-only and constant for the duration of rendering. Only one level of property access is permitted. `$labels.<key>` resolves to the string value associated with `key` in the active `Labels` bundle. Rendering MUST halt if the key is not found. Full locale selection and fallback rules are defined in [Localization](#localization). `$labels` references are valid in `title` and `description` fields of `Display` and `Field`, as well as in `Field.params` values.

#### Type Casting

After a value is resolved — whether from a literal or a variable reference — it is cast to the type expected by the consuming formatter parameter. For variable references, the resolved ABI type MUST be compatible with the expected type; if it is not, resolution MUST halt. For literals, the coercion rules in the [Literals](#literals) table apply.

Examples:
- A `percentage` formatter expects `basis: "10000"` (literal) → parsed as decimal and cast to `uint256`
- A `tokenAmount` formatter expects `amount: "$args.amount"` (reference to `uint256`) → types match, rendering continues
- An `address` formatter expects `value: "0x742d...bEb1"` (literal) → parsed as hex and cast to `address`
- A `switch` with `value: "$args.command"` (`uint8`) and child `case: ["8"]` (literal) → `"8"` parsed as decimal, cast to `uint8`, compared for equality

#### Resolution Failure

Resolution MUST halt if:

- The container is unknown.
- The referenced path does not exist in the current `$args` scope.
- An array index is out of bounds.
- The resolved value type is incompatible with what the formatter requires.
- A literal cannot be coerced to the required type.
- A `case` entry cannot be cast to the type of the `switch` `value` parameter.

### Rendering

A wallet renders a display specification by executing the following steps in order. Any failure at any step MUST halt rendering.

**Step 1 — Specification Location and Context Initialization**

The display specification is located either by display identifier (trustless) or by chain, address, and selector (trusted registry). Once located, two rendering contexts are initialized:

- `$msg` is populated from the transaction envelope: `sender`, `to`, `value`, and `data`. This context is read-only and constant for the entire top-level rendering scope.
- `$args` is initialized by ABI-decoding `$msg.data` per `Display.abi`. Named parameters are accessible by name (e.g., `$args.amount`); unnamed parameters are accessible by zero-based positional index (e.g., `$args.0`).

**Step 2 — Native Value Check**

If `$msg.value > 0`, the wallet MUST display a warning to the user that includes the exact native value amount being transferred.

**Step 3 — Field Iteration**

Iterate `Display.fields` in declaration order. For each `Field`:

1. **Conditional visibility**: If `Field.case` is non-empty, the field is rendered only if the enclosing `switch` value matches at least one `case` entry after type casting; otherwise the field is skipped. Fields with an empty `case` are always rendered.
2. **Reference resolution**: Resolve all `title`, `description`, and `params` values per [Variable References](#variable-references).
3. **Formatting**: Cast and format the resolved parameter values per the rules of `Field.format` defined in [Field Formats](#field-formats).
4. **Structural recursion**: For structural formats (`map`, `array`, `switch`), process nested `fields` with the scope rules specified for each format in [Field Formats](#field-formats).
5. **Nested call**: The `call` format is a special case and does not process nested `fields` within the current specification. Instead, the wallet constructs a new `$msg` from the `call` field's `to`, `value`, and `data` parameters — with `$msg.sender` set to the parent `$msg.to` — and locates an independent display specification for the inner call. The inner call uses its own spec's `labels` bundles; it does not inherit the outer `$labels` context. Outer rendering is paused; rendering restarts from Step 1 for the inner specification. Once the inner rendering completes, outer rendering resumes from where it was paused. Wallets MUST enforce a maximum recursion depth. Rendering MUST halt if the limit is exceeded.

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
    "$args.approved"         // value
)

// Equivalent raw hash computation
bytes32 approvedField = keccak256(abi.encode(
    FIELD_TH,
    keccak256(bytes("$labels.approved")),     // title
    keccak256(bytes("$labels.approvedDesc")), // description
    keccak256(bytes("boolean")),              // format
    keccak256(bytes("")),                     // case (empty)
    keccak256(abi.encodePacked(
        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("value")), keccak256(bytes("$args.approved"))))
    )),                                       // params
    keccak256(bytes(""))                      // fields (empty)
));
```

#### Rich Formats

These formats interpret a raw Solidity value into a human-readable semantic representation.

**`datetime`** — displays a Unix timestamp as an absolute, locale-formatted date and time (e.g., "13 May 2025, 14:30" or "May 13, 2025 2:30 PM" depending on locale). The `value` parameter contains a timestamp stored as an unsigned integer; the optional `units` parameter specifies how to interpret this integer (seconds since epoch, minutes since epoch, etc.). If `units` is omitted, the value is interpreted as seconds since Unix epoch (January 1, 1970, 00:00:00 UTC).

| Param   | Required | Description                                                                    |
|---------|----------|--------------------------------------------------------------------------------|
| `value` | yes      | Reference resolving to a `uintN` timestamp (count of time units since epoch)   |
| `units` | no       | How to interpret the input value: `"seconds"` (default), `"minutes"`, `"hours"`, `"days"`, `"weeks"` |

```solidity
Display.datetimeField(
    "$labels.deadline",      // title
    "$labels.deadlineDesc",  // description
    "",                      // case (empty)
    "$args.deadline"         // value
)
```

---

**`duration`** — displays a relative time span as a human-readable duration (e.g., "2 weeks", "3 days", "14 hours"). The `value` parameter contains a duration stored as an unsigned integer; the optional `units` parameter specifies how to interpret this integer. If `units` is omitted, the value is interpreted as seconds.

| Param   | Required | Description                                                                    |
|---------|----------|--------------------------------------------------------------------------------|
| `value` | yes      | Reference resolving to a `uintN` duration (count of time units)                |
| `units` | no       | How to interpret the input value: `"seconds"` (default), `"minutes"`, `"hours"`, `"days"`, `"weeks"` |

```solidity
Display.durationField(
    "$labels.lockPeriod",      // title
    "$labels.lockPeriodDesc",  // description
    "",                        // case (empty)
    "$args.lockPeriod"         // value
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
    "$args.feeBps",     // value
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
    "$args.permissions",        // value
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
    "$args.amount",        // value
    "6"                    // decimals
)
// Renders: "1.234567" for amount=1234567
```

#### Address Formats

Three address formats are defined, each with different verification requirements. `address` performs best-effort name resolution and is informational. `token` and `contract` require verification against a Token List and Contract List respectively; rendering MUST halt if the address is not found in the applicable list (see [Contract Lists](#contract-lists)). Developers MUST choose the semantically appropriate format.

**`address`** — 20-byte address with best-effort name resolution (local contacts, ENS). Use when identity is informational, not a security precondition.

| Param   | Required | Description                       |
|---------|----------|-----------------------------------|
| `value` | yes      | Reference resolving to an address |

```solidity
Display.addressField(
    "$labels.recipient",      // title
    "$labels.recipientDesc",  // description
    "",                       // case (empty)
    "$args.to"                // value
)
```

---

**`token`** — token address verified against a trusted Token List; rendering MUST halt if the address is not found. Use when token identity is critical to transaction assessment. An optional `tokenId` parameter enables display of non-fungible token identities.

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

**`contract`** — contract address verified against a trusted Contract List; rendering MUST halt if the address is not found. Use when the contract receives delegated authority or executes on the user's behalf (e.g., the spender in an ERC-20 `approve` call).

| Param   | Required | Description                               |
|---------|----------|-------------------------------------------|
| `value` | yes      | Reference resolving to a contract address |

```solidity
// Full display specification for ERC-20 approve
bytes32 constant APPROVE_DISPLAY_ID = Display.display(
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
            "$args.spender"         // value
        ),
        Display.tokenAmountField(
            "$labels.amount",      // title
            "$labels.amountDesc",  // description
            "",                    // case (empty)
            "$msg.to",             // token
            "$args.amount"         // amount
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

**`tokenAmount`** — displays a token amount denominated in a specific token, resolved against a trusted Token List. An optional `tokenId` parameter enables display of non-fungible token amounts. When `direction` is omitted, the amount is displayed without directional indication.

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
    "$args.amount"         // amount
)

// NFT
Display.tokenAmountField(
    "$labels.nft",            // title
    "$labels.nftDesc",        // description
    "",                       // case (empty)
    "$msg.to",                // token
    "1",                      // amount
    "$args.tokenId",          // tokenId
    Display.Direction.Out     // direction
)
```

#### Structural Formats

Structural formats carry nested `fields` and modify rendering context (see [Rationale: Structural Formats](#structural-formats-1) for design rationale):
- **`map`, `array`** — create isolated `$args` scope (nested fields access only explicitly passed data; `$msg` constant)
- **`call`** — creates new `$msg` context (independent rendering with own `$msg` and `$args`)
- **`switch`** — no new scope (child fields inherit parent's `$args`)

Wallets MUST enforce strict scope boundaries.

**`map`** — creates isolated `$args` scope, renders nested `fields`. Scope populated via:
- `$<name>` parameters: bind values to child scope (`$token` → `$args.token`)
- `abi` + `value`: ABI-decode bytes, merge fields into child scope

| Param     | Required | Description                                                                                                                                                                                         |
|-----------|----------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `$<name>` | no       | `$`-prefixed entries bind values into child `$args` scope (e.g. `$token` → `$args.token`)                                                                                                           |
| `abi`     | no       | Solidity type signature string (e.g. `"(address token,uint256 amount)"`) that ABI-decodes the bytes in `value` param and populates the child `$args` scope with the decoded fields as new variables |
| `value`   | no       | Reference resolving to bytes to be ABI-decoded using `abi` signature into child `$args` scope                                                                                                       |

```solidity
// Example 1: Using $-prefixed params to bind values
Display.mapField(
    "$labels.transfer",      // title
    "$labels.transferDesc",  // description
    "",                      // case (empty)
    abi.encodePacked(        // params
        Display.entry("$token", "$msg.to"),
        Display.entry("$amount", "$args.value")
    ),
    abi.encodePacked(        // fields
        Display.tokenAmountField(
            "$labels.amount",      // title
            "$labels.amountDesc",  // description
            "",                    // case (empty)
            "$args.token",         // token
            "$args.amount"         // amount
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
        Display.entry("value", "$args.orderData")
    ),
    abi.encodePacked(         // fields
        Display.tokenAmountField(
            "$labels.amount",      // title
            "",                    // description (empty)
            "",                    // case (empty)
            "$args.token",         // token (from ABI-decoded orderData)
            "$args.amount"         // amount (from ABI-decoded orderData)
        ),
        Display.datetimeField(
            "$labels.deadline",    // title
            "",                    // description (empty)
            "",                    // case (empty)
            "$args.deadline"       // deadline (from ABI-decoded orderData)
        )
    )
)
```

---

**`array`** — iterates parallel arrays, renders nested `fields` per element with fresh isolated `$args` scope. All `$`-prefixed parameters MUST be equal-length arrays; rendering MUST halt if lengths differ. Binds `array[i]` for each parameter per iteration.

Each `$<name>` parameter MUST resolve to one of the following iterable types:

| Input type  | Element type per iteration |
|-------------|----------------------------|
| `T[]`       | `T` (element of dynamic array) |
| `T[N]`      | `T` (element of fixed-size array) |
| `bytes`     | `bytes1` (single byte) |

Resolution MUST halt if a parameter resolves to a non-iterable type.

| Param     | Required           | Description                                                                            |
|-----------|--------------------|----------------------------------------------------------------------------------------|
| `$<name>` | yes (at least one) | Reference resolving to an iterable type; each element is bound as `$args.<name>` per iteration |

```solidity
Display.arrayField(
    "$labels.transfers",      // title
    "$labels.transfersDesc",  // description
    "",                       // case (empty)
    abi.encodePacked(         // params
        Display.entry("$to", "$args.recipients"),
        Display.entry("$amount", "$args.amounts")
    ),
    abi.encodePacked(         // fields
        Display.contractField(
            "$labels.recipient",  // title
            "",                   // description (empty)
            "",                   // case (empty)
            "$args.to"            // value
        ),
        Display.tokenAmountField(
            "$labels.amount",  // title
            "",                // description (empty)
            "",                // case (empty)
            "$msg.to",         // token
            "$args.amount"     // amount
        )
    )
)
```

---

**`call`** — renders a nested contract call. Creates a new `$msg` from `to`, `value`, and `data` parameters; `$msg.sender` of the inner context is set to the parent `$msg.to`. Outer rendering is paused; the wallet locates an independent display specification for the inner call and renders it from Step 1. Once the inner rendering completes, outer rendering resumes.

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
    "$args.to",               // to
    "$args.value",            // value
    "$args.data"              // data
)
```

---

**`switch`** — conditionally renders nested `fields` based on discriminant `value`. No new scope; child fields inherit parent's `$args`. Empty `case` array: always render. Non-empty `case`: render only if `value` matches at least one entry (equality comparison).

| Param   | Required | Description                                   |
|---------|----------|-----------------------------------------------|
| `value` | yes      | Reference or literal used as the discriminant |

```solidity
// Universal router: execute(uint8 command, bytes data)
// Different commands interpret the bytes parameter differently
Display.switchField(
    "$labels.operation",       // title
    "",                        // description (empty)
    "",                        // case (empty)
    "$args.command",           // value - uint8 command byte
    abi.encodePacked(          // fields
        // Command 0: TRANSFER - data contains (address recipient, uint256 amount)
        Display.mapField(
            "$labels.transfer",                      // title
            "",                                      // description (empty)
            abi.encodePacked(keccak256(bytes("0"))), // case: ["0"] — EIP-712 pre-encoded string[]
            abi.encodePacked(
                Display.entry("abi", "(address recipient,uint256 amount)"),
                Display.entry("value", "$args.data")
            ),
            abi.encodePacked(
                Display.addressField("$labels.recipient", "", "", "$args.recipient"),
                Display.unitsField("$labels.amount", "", "", "$args.amount", "18")
            )
        ),
        // Command 1: APPROVE - data contains (address spender, uint256 amount)
        Display.mapField(
            "$labels.approve",                       // title
            "",                                      // description (empty)
            abi.encodePacked(keccak256(bytes("1"))), // case: ["1"] — EIP-712 pre-encoded string[]
            abi.encodePacked(
                Display.entry("abi", "(address spender,uint256 amount)"),
                Display.entry("value", "$args.data")
            ),
            abi.encodePacked(
                Display.contractField("$labels.spender", "", "", "$args.spender"),
                Display.unitsField("$labels.amount", "", "", "$args.amount", "18")
            )
        ),
        // Command 2: SWAP - data contains (address tokenIn, address tokenOut, uint256 amountIn)
        Display.mapField(
            "$labels.swap",                          // title
            "",                                      // description (empty)
            abi.encodePacked(keccak256(bytes("2"))), // case: ["2"] — EIP-712 pre-encoded string[]
            abi.encodePacked(
                Display.entry("abi", "(address tokenIn,address tokenOut,uint256 amountIn)"),
                Display.entry("value", "$args.data")
            ),
            abi.encodePacked(
                Display.addressField("$labels.tokenIn", "", "", "$args.tokenIn"),
                Display.addressField("$labels.tokenOut", "", "", "$args.tokenOut"),
                Display.unitsField("$labels.amountIn", "", "", "$args.amountIn", "18")
            )
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

### Contract Lists

A **Contract List** is a JSON document maintained by a trusted party that maps contract addresses to their verified names and identities. It answers the question "is this address the contract I think it is?" — not "is this contract safe?". When a wallet processes a `contract` field, it MUST verify the resolved address against at least one trusted Contract List. If the address is not found, the wallet MUST halt rendering.

#### List Sources

Since any party may publish a Contract List, wallets determine which lists to trust. Which sources a wallet accepts is an implementation decision. Common sources include:

- **Contact List**: A Contract List maintained locally by the wallet on behalf of the user, populated through explicit user action. The wallet MUST treat the Contact List as trusted for verification purposes.
- **Wallet Provider**: A list curated and maintained by the wallet's own team.
- **Community**: Lists curated by DAOs, security councils, or open governance processes.
- **Auditor or Security Firm**: Lists published by professional security organizations based on contract review.
- **Block Explorer**: Lists derived from verified contract metadata published by block explorer operators.

#### User-Initiated Verification

If a resolved address is not present in any trusted Contract List, the wallet MAY offer the user an explicit manual verification flow. If the user confirms the contract's identity through this flow, the wallet MAY add the address to the user's Contact List. Subsequent interactions with this address MUST then pass verification against the Contact List.

### JSON Representation

Display specifications MAY be represented in JSON format for transmission via `wallet_sendTransaction` (EIP-TBD). The JSON representation omits EIP-712 type definitions (which are static across all displays) and uses the following normalization rules:

**Normalization Rules:**
- **Empty arrays**: Absent `case` and `fields` arrays MUST be represented as `[]`
- **Empty strings**: Absent `description` fields MUST be represented as `""`

The JSON structure maps directly to the EIP-712 structs:

```json
{
  "abi": "transfer(address to, uint256 amount)",
  "title": "$labels.title",
  "description": "$labels.description",
  "fields": [
    {
      "title": "$labels.sender",
      "description": "",
      "format": "address",
      "case": [],
      "params": [{"key": "value", "value": "$msg.sender"}],
      "fields": []
    },
    {
      "title": "$labels.amount",
      "description": "",
      "format": "tokenAmount",
      "case": [],
      "params": [
        {"key": "token", "value": "$msg.to"},
        {"key": "amount", "value": "$args.amount"}
      ],
      "fields": []
    },
    {
      "title": "$labels.recipient",
      "description": "",
      "format": "address",
      "case": [],
      "params": [{"key": "value", "value": "$args.to"}],
      "fields": []
    }
  ],
  "labels": [
    {
      "locale": "en",
      "items": [
        {"key": "title", "value": "Transfer"},
        {"key": "description", "value": "Transfer ERC-20 tokens"},
        {"key": "sender", "value": "From"},
        {"key": "amount", "value": "Amount"},
        {"key": "recipient", "value": "To"}
      ]
    }
  ]
}
```

Implementations MUST compute the display identifier by converting the JSON to EIP-712 structs and applying `hashStruct(Display)` as defined in EIP-712.

## Rationale

### Design Goals

The specification addresses calldata interpretation through local decoding without network dependencies, verifiable display identifiers via EIP-712, and censorship-resistant metadata access. The core principle is "display is law": display specifications are security-critical artifacts that cryptographically commit developers to the semantics shown to users.

This standard is bounded by what is present in the calldata at signing time. Contracts whose execution is driven by on-chain state rather than calldata parameters — such as a bare `execute()` — can hold a display specification but cannot surface dynamic values. Additionally, some rendering requires on-chain metadata absent from calldata: ERC-20 symbol and decimal precision must be queried from the token contract by the wallet. Both cases are outside the scope of this specification.

### EIP-712 Display Identifier

The EIP-712 `hashStruct` provides a compact, 32-byte identifier compatible with established ecosystem infrastructure and resource-constrained devices. This mechanism enables deterministic verification through both static precomputation and dynamic, on-chain generation.

Adopting EIP-712 for identifier computation means that improvements to the EIP-712 algorithm and its Solidity tooling accrue to this standard without requiring specification changes. Currently, display identifiers must be expressed as nested `keccak256(abi.encode(...))` chains — correct but verbose. Proposed Solidity compiler enhancements, including native `type(S).typehash` and `type(S).hashStruct(s)` support, will allow these to be replaced with direct type-level expressions evaluated at compile time. Solidity does not yet support compile-time constant evaluation (`constexpr`); as this capability is added to the compiler, display identifier constants will be expressible as simple compiler-verified declarations rather than manually assembled hash computations, further reducing boilerplate and eliminating a class of encoding errors.

### Semantic vs Visual Separation

The specification defines semantic meaning and data hierarchy, not visual presentation. This separation ensures specifications remain valid across different wallet implementations and device form factors while allowing wallets to optimize rendering for their specific constraints.

Interpolated string templates (e.g., `"Transfer {amount} to {destination}"`) are deliberately absent. They hard-code presentation, obscure type information (`{amount}` carries no indication that decimal scaling and symbol resolution are required), and are incompatible with natural language rendering, where grammatical agreement, word endings, and noun cases depend on the numeric value and surrounding context. Typed, named field definitions delegate all string composition to the wallet, keeping the specification language-agnostic and presentation-agnostic.

### Structural Formats

Structural formats (`map`, `array`, `switch`, `call`) enable display specifications to cover transaction patterns that cannot be expressed as flat field listings:

**`map`** decodes bytes-encoded subcommands. Universal routers encode complex swap paths as packed bytes; `map` decodes them using an ABI signature, making nested fields (token addresses, amounts, slippage) accessible by name instead of appearing as opaque hex.

**`array`** handles batch operations. Multicall contracts and batch transfers process variable-length lists; `array` renders them with a single field template that iterates over parallel arrays (recipients and amounts) without per-element duplication.

**`switch`** supports command dispatch. Universal routers multiplex operations via command bytes or enums; `switch` renders different fields based on the command value using `case` matching, covering opcode branching and mode selection without separate display specifications per command.

**`call`** renders nested execution. Smart contract wallets, multisigs, and DAOs wrap inner transactions as ABI parameters; `call` recursively renders the inner call using its own display specification, enabling complete call tree visualization for account abstraction (ERC-4337), multisig execution, and DAO proposals.

### Scope Isolation

`map` and `array` create isolated `$args` scopes; child fields access only explicitly passed parameters. `switch` does not create new scopes. `call` creates entirely new rendering contexts. Scope isolation prevents variable shadowing attacks, makes data flow auditable, and improves specification readability.

### Forward Compatibility

Field parameters use generic `Entry` key-value pairs and string-based format identifiers, allowing new semantic types to be added in future proposals without modifying core EIP-712 type definitions. Rejection of unknown format identifiers, enforced in the Specification, prevents downgrade attacks.

### Localization

Labels are included in the display identifier hash to prevent tampering and ensure translations carry the same cryptographic guarantees as field definitions. The bundle-based design separates translatable strings from format specifications and provides locale fallback mechanisms. Missing label keys halt rendering to prevent corrupted displays.

### Error Handling

The specification adopts halt-on-error behavior: any resolution failure, type mismatch, verification failure, or missing key halts rendering immediately. This prevents misleading displays where partial information could lead users to approve malicious transactions. A specification that is incomplete or incorrect will produce halt-on-error failures at render time.

## Backwards Compatibility

This ERC introduces a new standard and does not modify any existing Ethereum protocol, ABI encoding, or ERC. It has no backward compatibility requirements with respect to previously deployed contracts or existing wallet implementations. Wallets that do not implement this standard continue to operate under existing behavior; this standard defines an opt-in display layer.

Dependency on EIP-712 is additive: this standard reuses `hashStruct` solely for identifier computation and does not alter any EIP-712 behavior or interfere with existing EIP-712 signed data flows.

## Security Considerations

### Binding Display Specifications to Contracts

This specification does not define how display identifiers bind to contracts. The companion Onchain Clear Signing Verification standard (EIP-TBD) addresses on-chain verification mechanisms.

Without verification, users face specification substitution attacks, phishing via stolen specifications, and downgrade attacks. The on-chain verification mechanism is defined in the companion Onchain Clear Signing Verification standard (EIP-TBD); wallet implementations SHOULD NOT render display specifications without a verified binding to the target contract.

### Native Value Transfer Omission

Payable functions accepting `msg.value > 0` may omit native transfer display fields, hiding value transfers. Wallet implementations MUST display a prominent warning that includes the exact native value amount being transferred, so the user can assess the transfer independently of the display specification. Wallets MAY additionally require that `$msg.to` is present in a trusted Contract List when `$msg.value > 0`, ensuring native value is only transferred to a contract with a verified identity.

### Developer Responsibilities

This standard guarantees that execution matches the committed display specification; it cannot verify that the specification truthfully describes the contract's behavior. Developers MUST ensure specifications accurately represent behavior and SHOULD apply security review processes equivalent to smart contract code.

### Malicious Displays

A malicious developer may author a specification that misrepresents an operation through misleading labels or omitted parameters. Directly calling a malicious contract has limited damage scope: contracts are isolated, and a malicious contract cannot access a user's assets in other contracts without permissions the user has previously granted. The primary attack surface is not the malicious contract itself, but the act of granting it rights.

The principal phishing vector is a transaction targeting a legitimate contract — for example, an ERC-20 token — with parameters that delegate authority to a malicious address (e.g., `approve(maliciousSpender, maxAmount)`). The `contract` field format addresses this directly: the resolved spender address is verified against trusted Contract Lists, and rendering MUST halt if the address is absent. A malicious address cannot appear in a reputable Contract List without the list maintainer's knowledge, making this class of attack detectable before the user signs.

Wallets MAY additionally restrict clear signing display to transactions where the `$msg.to` address is itself verified by a trusted Contract List. This reduces the social engineering surface further, at the cost of limiting which contracts users can interact with.

### Denial of Service

Malicious specifications can exhaust wallet resources via excessive recursion (`call`), large arrays (`array`), deep nesting, or oversized labels. Wallet implementations MUST enforce platform-appropriate limits on recursion depth, array sizes, and computational complexity. Rendering MUST halt when limits are exceeded.

## Copyright

Copyright and related rights waived via [CC0](../LICENSE.md).
