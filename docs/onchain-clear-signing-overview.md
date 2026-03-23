# Onchain Clear Signing

Blind signing is the state of Ethereum today: users approve transactions they cannot meaningfully read, trusting the requesting application to accurately describe what they are authorizing. In February 2025, Bybit suffered a loss of approximately $1.5 billion when Safe multisig signers approved a transaction that, contrary to its display, replaced the wallet implementation and transferred control to attacker-controlled addresses. Existing approaches — curated off-chain registries such as ERC-7730 — bring clear signing to a small set of well-known contracts, but they depend on third-party curation, cover only the most prominent protocols, and cannot scale to the long tail of contracts deployed every day.

Any solution must satisfy three requirements. **Trustlessness**: the display is verified by math and contract logic alone — no party needs to be trusted. **Decentralization**: any developer can publish their own display specification with no registry, no curator, and no approval gate. **Security**: the display must be cryptographically bound to execution, not advisory metadata that can be substituted.

We propose **Onchain Clear Signing** — a clear signing architecture defined across three interdependent ERCs that satisfies these requirements. Contract developers embed display specifications directly in their contract code. The function selector is extended with a display identifier, binding the display to on-chain execution with no external registry. Transaction requests carry the display specification alongside the calldata, making full verification possible even on air-gapped hardware wallets. The goal is that clear signing becomes the baseline expectation for every newly deployed contract.

---

## The Architecture

The architecture is built on three layers: a **semantic type system** that defines what data means, an **on-chain enforcement mechanism** that binds the display to execution, and a **transport layer** that delivers the display specification to the wallet.

**[Onchain Clear Signing Specification]** defines the type system and display identifier. Developers write display specifications that describe how a function's calldata should be interpreted and shown, and embed them directly in the contract. Each specification is uniquely identified by a compact 32-byte digest that any device can compute locally from the spec itself, without network access.

**[Onchain Clear Signing Verification]** defines how a contract binds its display specification to its own execution. The function selector is extended with a 32-byte display identifier; the contract commits to the expected identifier at deployment and verifies it on every call, ensuring the display the user approved is the display bound to that execution. If the identifier does not match, the transaction reverts. This transforms display specifications from advisory metadata into enforced preconditions of execution — **"display is law"**.

**[wallet_sendTransaction]** defines the transport. The dApp bundles the display specification with the transaction request — a **push model** that eliminates live network dependencies at signing time. No registry to query; no external service that can be unavailable or compromised. The method works on air-gapped hardware.

Display specifications compose. A smart account calling a multisig calling a swap contract renders each layer independently — the wallet locates the display specification for each nested call, renders it in full, and presents the composed result before the user signs. This has direct implications for account abstraction: wallets do not need built-in knowledge of every smart account entrypoint or execution framework. Any contract that embeds a display specification is fully renderable and verifiable, regardless of how it is invoked.

---

## Transaction Flow

```
dApp
 │
 │  wallet_sendTransaction(tx, display_spec) (Transport)
 ▼
Wallet
 ├─ Renders calldata using the display specification (Spec)
 ├─ Verifies designated addresses against Contract Lists [optional] (Spec)
 ├─ Computes and verifies display identifier locally (Spec)
 └─ User approves the rendered display
 │
 │  clearCall(display_identifier || selector || calldata) (Verification)
 ▼
Contract
 └─ Verifies identifier matches committed value → executes (Verification)
```

Verification is independent at each layer; no layer trusts another. A display identifier mismatch results in wallet rejection before submission. A call submitted without a valid identifier — bypassing the wallet entirely — reverts at the contract. The mechanism carries a small, fixed gas overhead: the 32-byte display identifier prepended to calldata increases calldata cost, while the on-chain validation contributes a constant cost to execution.

---

## Contract Lists: The Social Layer

Cryptographic binding guarantees that execution matches the committed display specification. It does not guarantee the contract's identity — a malicious contract can commit to an accurate display of its own malicious behavior.

The primary attack vector is a transaction targeting a legitimate contract with parameters that delegate authority to a malicious address — for example, `approve(maliciousSpender, maxAmount)` on an ERC-20 token. To prevent this, the architecture includes **Contract Lists**: curated mappings of contract addresses to verified contract identities, following the same model as Token Lists.

Display specifications designate which addresses require verification using typed formats (`address`, `token`, `contract`). When a wallet encounters an address marked for verification, it checks trusted Contract Lists and halts rendering if the address is not recognized. Contract Lists can be published by wallet providers, DAOs, security councils, or maintained locally by users. Full details are in the Onchain Clear Signing Specification.

---

## Request for Feedback

These three ERCs are at the idea stage. We plan to submit to the EIP repository once we have addressed community feedback. We are looking for feedback on:

**Architecture.** Where might the design break or create unexpected complexity? 

**Integration.** What adoption friction do you anticipate? For dApps: build-time tooling burden. For wallets: rendering engine complexity on constrained devices. For contracts: gas overhead and proxy compatibility.

**Security.** Attack vectors we have not addressed? 

**Format coverage.** Missing types or rendering requirements for your use case.

**Backward compatibility.** Migration paths for non-upgradeable contracts. See the Backwards Compatibility section in the Onchain Clear Signing Verification ERC for proposed options.

**Early adoption.** If you are building a wallet, dApp, or contract and want to be an early adopter, let us know.

Thanks for reading — looking forward to the discussion.
