// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import "./IUniswapV2Router.sol";
import "./SwapExactTokensForTokensDisplayHash.sol";

contract ClearCallRouter is IUniswapV2Router {

    ClearCallRouter internal immutable ROUTER = ClearCallRouter(this);
    bytes32 public constant SWAP_EXACT_TOKENS_FOR_TOKENS_DISPLAY_HASH = SwapExactTokensForTokensDisplayHash.SWAP_EXACT_TOKENS_FOR_TOKENS_DISPLAY_HASH;

    function clearCall() external payable returns (bytes memory) {
        // Extract displayHash from bytes 4-35 (after the clearCall selector)
        bytes32 displayHash = bytes32(msg.data[4:36]);
        // Extract call selector from bytes 36-39
        bytes4 callSelector = bytes4(msg.data[36:40]);

        _validateDisplayHash(callSelector, displayHash);

        // Execute the actual call using msg.data[36:] (selector + params)
        (bool success, bytes memory returndata) = address(this).delegatecall(msg.data[36:]);
        if (!success) {
            if (returndata.length > 0) {
                // Bubble up the revert reason
                assembly {
                    let returndata_size := mload(returndata)
                    revert(add(32, returndata), returndata_size)
                }
            } else {
                revert("Call failed");
            }
        }

        return returndata;
    }

    function _validateDisplayHash(
        bytes4 selector,
        bytes32 displayHash
    ) private view {
        if (selector == IUniswapV2Router.swapExactTokensForTokens.selector) {
            require(
                displayHash == SWAP_EXACT_TOKENS_FOR_TOKENS_DISPLAY_HASH,
                "Invalid display hash for swap exact tokens for tokens"
            );
        } else {
            revert("Invalid selector");
        }
    }

    function swapExactTokensForTokens(
        uint /* amountIn */,
        uint /* amountOutMin */,
        address[] calldata path,
        address /* to */,
        uint /* deadline */
    ) external virtual override returns (uint[] memory amounts) {
        // This is a stub implementation. The actual logic should be implemented in the derived contract.
        return new uint[](path.length);
    }
}
