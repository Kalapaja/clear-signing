// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import "./IUniswapV2Router.sol";
import "./SwapExactTokensForTokensDisplayHash.sol";

contract ClearCallRouter is IUniswapV2Router {
    using SwapExactTokensForTokensDisplayHash for ClearCallRouter;

    ClearCallRouter internal immutable ROUTER = ClearCallRouter(this);
    bytes32 public immutable SWAP_EXACT_TOKENS_FOR_TOKENS_DISPLAY_HASH =
        ROUTER.getSwapExactTokensForTokenDisplayHash();

    function clearCall(
        bytes32 displayHash,
        bytes calldata call
    ) external payable returns (bytes memory) {
        bytes4 selector = bytes4(call[:4]);

        _validateDisplayHash(selector, displayHash);

        // 2. Execute the actual call & bubble up errors (OpenZeppelin Address.sol pattern)
        (bool success, bytes memory returndata) = address(this).delegatecall(call);
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
