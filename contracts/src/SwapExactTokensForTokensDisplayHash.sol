// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

import "./ClearCallRouter.sol";
import "./Display.sol";

library SwapExactTokensForTokensDisplayHash {
    function getSwapExactTokensForTokenDisplayHash(ClearCallRouter router) internal pure returns (bytes32) {
        return Display.display(
            address(router),
            "function swapExactTokensForTokens(uint256 amountIn, uint256 amountOutMin, address[] path, address to, uint256 deadline)",
            "$labels.swap",
            "$labels.swap_description",
            abi.encode(
                // Field 1: Sending
                Display.tokenAmountField(
                    "$labels.sending",
                    "$labels.sending_description",
                    hex"",
                    "$locals.path[0]",
                    "$locals.amountIn"
                ),
                // Field 2: Receiving (minimum)
                Display.tokenAmountField(
                    "$labels.receiving_min",
                    "$labels.receiving_min_description",
                    hex"",
                    "$locals.path[-1]",
                    "$locals.amountOutMin"
                ),
                // Field 3: Recipient
                Display.addressField(
                    "$labels.recipient",
                    "$labels.recipient_description",
                    hex"",
                    "$locals.to"
                ),
                // Field 4: Deadline
                Display.datetimeField(
                    "$labels.deadline",
                    "$labels.deadline_description",
                    hex"",
                    "$locals.deadline"
                )
            ),
            // Labels
            abi.encode(
                // en
                Display.labels(
                    "en",
                    abi.encode(
                        Display.entry(
                            "swap",
                            "Swap Tokens"
                        ),
                        Display.entry(
                            "swap_description",
                            "Exchange one token for another at the current market rate"
                        ),
                        Display.entry(
                            "sending",
                            "You're Sending"
                        ),
                        Display.entry(
                            "sending_description",
                            "Exact amount of tokens you're swapping"
                        ),
                        Display.entry(
                            "receiving_min",
                            "You're Receiving (minimum)"
                        ),
                        Display.entry(
                            "receiving_min_description",
                            "Minimum amount you'll receive - protects against price slippage"
                        ),
                        Display.entry(
                            "recipient",
                            "Recipient"
                        ),
                        Display.entry(
                            "recipient_description",
                            "Address that will receive the output tokens"
                        ),
                        Display.entry(
                            "deadline",
                            "Deadline"
                        ),
                        Display.entry(
                            "deadline_description",
                            "Transaction must complete before this time"
                        )
                    )
                )
            )
        );
    }
}
