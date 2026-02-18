// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

import "./Display.sol";

library SwapExactTokensForTokensDisplayHash {

    bytes32 public constant SWAP_EXACT_TOKENS_FOR_TOKENS_DISPLAY_HASH = keccak256(
        abi.encode(
            DISPLAY_TH,
            keccak256(bytes("function swapExactTokensForTokens(uint256 amountIn, uint256 amountOutMin, address[] path, address to, uint256 deadline)")),
            keccak256(bytes("$labels.swap")),
            keccak256(bytes("$labels.swap_description")),
            keccak256(abi.encodePacked(
                // Field 1: tokenAmountField for sending
                keccak256(abi.encode(
                    FIELD_TH,
                    keccak256(bytes("$labels.sending")),
                    keccak256(bytes("$labels.sending_description")),
                    keccak256(bytes("tokenAmount")),
                    keccak256(bytes("")), // checks
                    keccak256(abi.encodePacked(
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("token")), keccak256(bytes("$locals.path[0]")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("amount")), keccak256(bytes("$locals.amountIn"))))
                    )),
                    keccak256(bytes("")) // fields
                )),
                // Field 2: tokenAmountField for receiving
                keccak256(abi.encode(
                    FIELD_TH,
                    keccak256(bytes("$labels.receiving_min")),
                    keccak256(bytes("$labels.receiving_min_description")),
                    keccak256(bytes("tokenAmount")),
                    keccak256(bytes("")), // checks
                    keccak256(abi.encodePacked(
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("token")), keccak256(bytes("$locals.path[-1]")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("amount")), keccak256(bytes("$locals.amountOutMin"))))
                    )),
                    keccak256(bytes("")) // fields
                )),
                // Field 3: addressField for recipient
                keccak256(abi.encode(
                    FIELD_TH,
                    keccak256(bytes("$labels.recipient")),
                    keccak256(bytes("$labels.recipient_description")),
                    keccak256(bytes("address")),
                    keccak256(bytes("")), // checks
                    keccak256(abi.encodePacked(
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("value")), keccak256(bytes("$locals.to"))))
                    )),
                    keccak256(bytes("")) // fields
                )),
                // Field 4: datetimeField for deadline
                keccak256(abi.encode(
                    FIELD_TH,
                    keccak256(bytes("$labels.deadline")),
                    keccak256(bytes("$labels.deadline_description")),
                    keccak256(bytes("datetime")),
                    keccak256(bytes("")), // checks
                    keccak256(abi.encodePacked(
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("value")), keccak256(bytes("$locals.deadline"))))
                    )),
                    keccak256(bytes("")) // fields
                ))
            )),
            keccak256(abi.encodePacked(
                // Labels for "en"
                keccak256(abi.encode(
                    LABELS_TH,
                    keccak256(bytes("en")),
                    keccak256(abi.encodePacked(
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("swap")), keccak256(bytes("Swap Tokens")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("swap_description")), keccak256(bytes("Exchange one token for another at the current market rate")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("sending")), keccak256(bytes("You're Sending")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("sending_description")), keccak256(bytes("Exact amount of tokens you're swapping")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("receiving_min")), keccak256(bytes("You're Receiving (minimum)")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("receiving_min_description")), keccak256(bytes("Minimum amount you'll receive - protects against price slippage")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("recipient")), keccak256(bytes("Recipient")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("recipient_description")), keccak256(bytes("Address that will receive the output tokens")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("deadline")), keccak256(bytes("Deadline")))),
                        keccak256(abi.encode(ENTRY_TH, keccak256(bytes("deadline_description")), keccak256(bytes("Transaction must complete before this time"))))
                    ))
                ))
            ))
        )
    );

}
