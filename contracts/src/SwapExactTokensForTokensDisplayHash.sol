// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

import "./Display.sol";

library SwapExactTokensForTokensDisplayHash {
    bytes32 public constant SWAP_EXACT_TOKENS_FOR_TOKENS_DISPLAY_HASH = 0xc0f1de3c8662b8c5152998e4e528e0995cf67090535306c810bfe2c964d72bbe;

    function SWAP_EXACT_TOKENS_FOR_TOKENS() pure public returns (bytes32) {
        return Display.display(
            "function swapExactTokensForTokens(uint256 amountIn, uint256 amountOutMin, address[] path, address to, uint256 deadline)",
            "$labels.swap",
            "$labels.swap_description",
            abi.encodePacked(
                Display.tokenAmountField(
                    "$labels.sending",
                    "$labels.sending_description",
                    "",
                    "$args.path[0]",
                    "$args.amountIn"
                ),
                Display.tokenAmountField(
                    "$labels.receiving_min",
                    "$labels.receiving_min_description",
                    "",
                    "$args.path[-1]",
                    "$args.amountOutMin"
                ),
                Display.addressField(
                    "$labels.recipient",
                    "$labels.recipient_description",
                    "",
                    "$args.to"
                ),
                Display.datetimeField(
                    "$labels.deadline",
                    "$labels.deadline_description",
                    "",
                    "$args.deadline"
                )
            ),
            abi.encodePacked(
                Display.labels(
                    "en",
                    abi.encodePacked(
                        Display.entry("swap", "Swap Tokens"),
                        Display.entry("swap_description", "Exchange one token for another at the current market rate"),
                        Display.entry("sending", "You're Sending"),
                        Display.entry("sending_description", "Exact amount of tokens you're swapping"),
                        Display.entry("receiving_min", "You're Receiving (minimum)"),
                        Display.entry("receiving_min_description", "Minimum amount you'll receive - protects against price slippage"),
                        Display.entry("recipient", "Recipient"),
                        Display.entry("recipient_description", "Address that will receive the output tokens"),
                        Display.entry("deadline", "Deadline"),
                        Display.entry("deadline_description", "Transaction must complete before this time")
                    )
                )
            )
        );
    }
}
