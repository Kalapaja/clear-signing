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
    - [Field Formats](#field-formats)
    - [Localization](#localization)
- [Rationale](#rationale)
- [Security Considerations](#security-considerations)
- [Copyright](#copyright)

## Abstract

Smart contract ABIs define parameter encoding but carry no semantic meaning. This standard defines a structured display specification that maps raw calldata fields to rich semantic types (token amounts, timestamps, addresses, nested calls). Each specification is uniquely identified by a 32-byte display identifier computed as an EIP-712 `hashStruct` digest, enabling resource-constrained devices to cryptographically verify specifications without network access. The specification defines the type system, rendering rules, and identifier computation; a companion standard addresses on-chain verification mechanisms.

## Motivation

The Ethereum ABI encodes function arguments but carries no semantic meaning. A timestamp and a price are both `uint256`; a token transfer and delegated execution are both `bytes`. Wallets cannot distinguish what values represent without external metadata.

This absence of machine-parseable semantics causes blind signing. Users authorize transactions they cannot interpret, trusting the dApp interface completely. This trust model contradicts the security guarantees of self-custodial wallets and hardware devices.

A practical display specification requires two properties: (1) rich semantic types covering real contract patterns—token amounts, timestamps, durations, percentages, nested calls; (2) a compact identifier computable by resource-constrained devices without network access. This standard defines the type system and EIP-712-based identifier computation. A companion standard defines on-chain verification mechanisms.

## Specification

The keywords "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119 and RFC 8174.

This specification defines semantic presentation: what data represents and how it relates, not visual rendering. Wallets interpret the same specification differently based on device constraints. A `tokenAmount` field identifies "an amount in a token"; one wallet shows plain text, another adds icons and fiat conversion. This separation ensures device-agnostic specifications.

### Type Definitions

A display specification consists of four composable types. The `displayHash` is `hashStruct(Display)` per EIP-712. Implementations MUST use these type strings verbatim.

**`Display`** — root type for one function's display specification.

- `abi` — Solidity function signature for selector matching
- `title` — label reference or literal string for transaction title
- `description` — human-readable operation description
- `fields` — ordered array of `Field` definitions
- `labels` — array of `Labels` bundles

**`Field`** — single display item definition.

- `title` — label reference or literal for field name
- `description` — optional human-readable description
- `format` — display format identifier
- `case` — conditional visibility array (empty means always shown)
- `params` — `Entry` array supplying formatter arguments (format-specific keys, variable references or literals)
- `fields` — nested `Field` definitions for structural formats

**`Labels`** — locale-specific string bundle.

- `locale` — locale identifier (e.g., `en`, `fr`)
- `items` — `Entry` array mapping label keys to translated strings

**`Entry`** — generic key-value pair.

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

### Function Signature Format

The `Display.abi` field MUST be a full Solidity function signature: `function <name>(<type> <name>, ...) [<modifier>]`.

The `function` keyword is required. Each parameter MUST include type and name; names populate `$data` for field parameter references (e.g., `$data.amount`). The optional state mutability modifier MUST be `pure`, `view`, `payable`, or `nonpayable` (default if omitted). Wallets MUST reject calls where `msg.value > 0` and modifier is not `payable`.

Examples:
```solidity
function transfer(address to, uint256 amount)
function approve(address spender, uint256 amount) nonpayable
function deposit() payable
```

The function selector for matching derives from the canonical ABI signature (types only, names and modifier stripped) via `keccak256`.

### Variable References

`Entry` values in `Field.params` are either a variable reference (starts with `$`, resolves at render time) or a literal (constant string like `"6"`, `"10000"`, `"Read"`).

#### Literals

Literals are untyped strings coerced at render time per these rules:

| Target type | Coercion rule                                                        |
|-------------|----------------------------------------------------------------------|
| `bool`      | `"true"` or `"false"` (case-insensitive). Any other value MUST halt. |
| `uint`      | Decimal integer string parseable as `uint256`.                       |
| `int`       | Decimal integer string parseable as `int256`.                        |
| `address`   | Hex string parseable as a 20-byte address.                           |
| `bytes`     | Hex string with optional `0x` prefix.                                |
| `string`    | Used as-is.                                                          |

If the literal cannot be coerced to the required type, resolution MUST halt.

#### Reference Containers

**`$msg`** — transaction context, constant throughout rendering (one level of access, no nesting):
- `$msg.sender` — caller address
- `$msg.to` — contract receiving the call
- `$msg.value` — native value (`uint256`)
- `$msg.data` — raw calldata bytes

**`$data`** — decoded function arguments for current scope. Top-level: decoded from `$msg.data` per `Display.abi`. Structural formats (`map`, `array`, `switch`) create new isolated `$data` scope; nested fields do NOT inherit parent's `$data`. Access patterns:
- Named: `$data.amount`, `$data.to`
- Positional: `$data.0`, `$data.1`
- Nested: `$data.order.token`
- Array index: `$data.items[0]`, `$data.items[-1]` (negative from end)
- Slice: `$data.items[1:3]`, `$data.data[:]`

#### Resolution Failure

Resolution MUST halt if:

- The container is unknown.
- The referenced path does not exist in the current `$data` scope.
- An array index is out of bounds.
- The resolved value type is incompatible with what the formatter requires.
- A literal cannot be coerced to the required type.

### Field Formats

#### Raw Solidity Types

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

#### Rich Formats

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

#### Address Formats

Three address types with different verification requirements: `address` (informational, best-effort name resolution), `token` and `contract` (verified against lists or rendering halts). Developers MUST choose the semantically appropriate format.

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

**`token`** — token address verified against Token List (rendering halts if not found). Use when token identity is critical to transaction assessment. Supports NFTs via optional `tokenId`.

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

**`contract`** — contract address verified against Contract List (rendering halts if not found). Use when contract receives delegated authority or executes on user's behalf (e.g., ERC-20 `approve` spender).

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

#### Value Formats

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

#### Structural Formats

Structural formats carry nested `fields` and modify rendering context:
- **`map`, `array`** — create isolated `$data` scope (nested fields access only explicitly passed data; `$msg` constant)
- **`call`** — creates new `$msg` context (independent rendering with own `$msg` and `$data`)
- **`switch`** — no new scope (child fields inherit parent's `$data`)

Wallets MUST enforce strict scope boundaries.

**`map`** — creates isolated `$data` scope, renders nested `fields`. Scope populated via:
- `$<name>` params: bind values to child scope (`$token` → `$data.token`)
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

**`array`** — iterates parallel arrays, renders nested `fields` per element with fresh isolated `$data` scope. All `$`-prefixed params MUST be equal-length arrays (halt if mismatch). Binds `array[i]` for each param per iteration.

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

**`call`** — renders nested contract call display specification. Creates new `$msg` from `to`, `value`, `data` params; `$msg.sender` propagates from parent. Wallet matches specification, renders, then resumes parent rendering.

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

### Localization

User-facing strings MAY use `$labels.key` references for internationalization.

**Label Resolution:** Wallet selects `Labels` bundle (exact locale match preferred, language-only fallback, then `en` default). Searches bundle's `items` for matching `key`. Rendering MUST halt if key not found.

**Labels Structure:** `locale` (identifier like `en`, `fr`, `en-US`) and `items` (array of `Entry` pairs mapping keys to strings).

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


## Rationale

### Design Goals

The specification addresses calldata interpretation through local decoding without network dependencies, verifiable display identifiers via EIP-712, and censorship-resistant metadata access. The core principle is "display is law": display specifications are security-critical artifacts that cryptographically commit developers to the semantics shown to users.

### EIP-712 Display Identifier

EIP-712 `hashStruct` provides a compact 32-byte identifier suitable for resource-constrained devices, leverages battle-tested infrastructure across the ecosystem, and enables both static precomputation and dynamic on-chain generation while maintaining deterministic verification.

### Semantic vs Visual Separation

The specification defines semantic meaning and data hierarchy, not visual presentation. This separation ensures specifications remain valid across different wallet implementations and device form factors while allowing wallets to optimize rendering for their specific constraints.

### Structural Formats

Structural formats (`map`, `array`, `switch`, `call`) enable composition of complex transaction patterns. The `map` format supports inline ABI decoding for bytes-encoded parameters, avoiding unwieldy flattened signatures. These formats compose to handle batch executors and universal routers while maintaining scope isolation for security.

### Scope Isolation

`map` and `array` create isolated `$data` scopes; child fields access only explicitly passed parameters. `switch` does not create new scopes. `call` creates entirely new rendering contexts. Scope isolation prevents variable shadowing attacks, makes data flow auditable, and improves specification readability. Wallets MUST enforce strict scope boundaries.

### Forward Compatibility

Field parameters use generic `Entry` key-value pairs and string-based format identifiers, allowing new semantic types to be added in future proposals without modifying core EIP-712 type definitions. Wallets MUST reject unknown format types to prevent downgrade attacks.

### Localization

Labels are included in the display identifier hash to prevent tampering and ensure translations carry the same cryptographic guarantees as field definitions. The bundle-based design separates translatable strings from format specifications and provides locale fallback mechanisms. Missing label keys halt rendering to prevent corrupted displays.

### Error Handling

The specification adopts halt-on-error behavior: any resolution failure, type mismatch, verification failure, or missing key halts rendering immediately. This prevents misleading displays where partial information could lead users to approve malicious transactions. Specifications must be complete and correct.

## Security Considerations

### Binding Display Specifications to Contracts

This specification does NOT define how display identifiers bind to contracts. A companion standard addresses on-chain verification mechanisms.

Without verification, users face specification substitution attacks, phishing via stolen specifications, and downgrade attacks. Wallet implementations MUST NOT display transactions based on unverified specifications.

### Native Value Transfer Omission

Payable functions accepting `msg.value > 0` may omit native transfer display fields, hiding value transfers. Wallet implementations MUST display a prominent warning or synthetic transfer field when `msg.value > 0` is undisclosed.

### Developer Responsibilities

Malicious developers can create specifications that misrepresent behavior, mislead via incorrect labels, or omit critical parameters. This threat is outside this standard's scope; users rely on developer trust. Mitigation: wallets may restrict transactions to Contract List-verified contracts.

Developers MUST ensure specifications accurately represent behavior and SHOULD apply security review processes equivalent to smart contract code.

### Denial of Service

Malicious specifications can exhaust wallet resources via excessive recursion (`call`), large arrays (`array`), deep nesting, or oversized labels. Wallet implementations MUST enforce platform-appropriate limits on recursion depth, array sizes, and computational complexity. Rendering MUST halt when limits are exceeded.

## Copyright

Copyright and related rights waived via [CC0](../LICENSE).
