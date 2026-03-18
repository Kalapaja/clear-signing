# Onchain Display: Clear Signing for Every Contract

Blind signing is the state of Ethereum today: users approve transactions they cannot meaningfully read, trusting the requesting application to accurately describe what they are authorizing. In February 2025, Bybit suffered a loss of approximately $1.5 billion: Safe multisig signers approved a transaction that, contrary to its display, replaced the wallet implementation and transferred control to attacker-controlled addresses. Post-incident analysis attributed the breach to a compromised signing interface that presented falsified transaction context; no mechanism existed at the signing layer to verify that the display matched what would execute. Clear signing addresses this class of vulnerability by cryptographically binding the transaction display to on-chain execution, reducing reliance on the integrity of the application layer.

The community has recognized this problem. Existing approaches — curated off-chain registries such as ERC-7730 — bring clear signing to a small set of well-known contracts. These efforts are valuable, but they are structurally limited: they depend on third-party curation, cover only the most prominent protocols, and cannot scale to the long tail of contracts deployed every day.

We propose **Onchain Display** — a clear signing architecture defined across three interdependent EIPs. Each contract developer embeds a display specification directly in their contract code. The function selector is extended with a display identifier, binding the display to on-chain execution with no external registry and no third party that can become a bottleneck or point of failure. Transaction requests carry the display specification alongside the calldata, making full verification possible even on air-gapped hardware wallets. All three EIPs are required for the architecture to work; none is sufficient on its own. The goal is that within a few years, clear signing is not a feature of a curated set of well-known contracts — it is the baseline expectation for every newly deployed contract. This post introduces the architecture and how the three layers compose. The full EIP drafts follow in subsequent posts.

---

## The Architecture

The architecture is built on three layers: a **semantic type system** that defines what data means, an **on-chain enforcement mechanism** that binds the display to execution, and a **transport layer** that delivers the display specification to the wallet.

**[EIP 1 — Onchain Display Specification]** defines the type system and display identifier. Developers write display specifications that describe how a function's calldata should be interpreted and shown, and embed them directly in the contract. Each specification is uniquely identified by a compact 32-byte digest that any device can compute locally from the spec itself, without network access.

**[EIP 2 — Onchain Display Verification]** defines how a contract binds its display specification to its own execution. The function selector is extended with a 32-byte display identifier; the contract commits to the expected identifier at deployment and verifies it on every call, ensuring the display the user approved is the display bound to that execution. If the identifier does not match, the transaction reverts. This transforms display specifications from advisory metadata into enforced preconditions of execution — **"display is law"**.

**[EIP 3 — wallet_sendTransaction]** defines the transport. The dApp bundles the display specification with the transaction request — a **push model** that eliminates live network dependencies at signing time. No registry to query; no external service that can be unavailable or compromised. The method works on air-gapped hardware.

Display specifications compose. A smart account calling a multisig calling a swap contract renders each layer independently — the wallet locates the display specification for each nested call, renders it in full, and presents the composed result before the user signs. This has direct implications for account abstraction: wallets do not need built-in knowledge of every smart account entrypoint or execution framework. Any contract that embeds a display specification is fully renderable and verifiable, regardless of how it is invoked.

---

## How They Compose

```
dApp
 │
 │  wallet_sendTransaction(tx, display_spec) (EIP 3)
 ▼
Wallet
 ├─ Renders calldata using the display specification (EIP 1)
 ├─ Verifies designated addresses against Contract Lists [optional] (EIP 1)
 ├─ Computes and verifies display identifier locally (EIP 1)
 └─ User approves the rendered display
 │
 │  clearCall(display_identifier || selector || calldata) (EIP 2)
 ▼
Contract
 └─ Verifies identifier matches committed value → executes (EIP 2)
```

Each layer verifies independently and trusts none of the others. A mismatched identifier causes wallet rejection before submission; a call submitted directly, bypassing the wallet, reverts on-chain.

---

## Address Verification: The Social Layer

Cryptographic binding guarantees that execution matches the committed display specification. It does not guarantee the contract's identity — a malicious contract can commit to an accurate display of its own malicious behavior.

Address identity requires a complementary social trust layer. We adopt **Contract Lists**: lists mapping contract addresses to verified identities, following the same model as Token Lists. Any party may publish a Contract List; common sources include wallet providers, DAOs, and security councils. Display specifications designate which address parameters require verification; when the wallet encounters such an address — the spender in an `approve` call being the primary example — it checks it against trusted Contract Lists and halts rendering if the address is not recognized.

A user's **Contact List** is a local Contract List managed by the wallet and populated through explicit user action. It requires no external trust — the user is the authority.

---

## Request for Feedback

The full EIP drafts follow in the next posts. We are looking for feedback on:

**Architecture.** Where might the three-layer design break or create unexpected complexity?

**Integration.** What friction do you anticipate on the dApp, wallet, and contract sides?

**Format coverage.** Missing types or rendering requirements for your use case — let us know before the spec is finalized.

**Backward compatibility.** EIP 2 cannot be adopted by non-upgradeable contracts. What migration paths are realistic?

**Early adoption.** If you are building a wallet, dApp, or contract and want to be an early adopter, we would like to hear from you.

Thanks for reading — looking forward to the discussion.
