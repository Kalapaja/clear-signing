// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import "./IUniswapV2Router.sol";
import "./SwapExactTokensForTokensDisplayHash.sol";

contract ClearCallRouter is IUniswapV2Router {

    ClearCallRouter internal immutable ROUTER = ClearCallRouter(this);
    bytes32 public constant SWAP_EXACT_TOKENS_FOR_TOKENS_DISPLAY_HASH = SwapExactTokensForTokensDisplayHash.SWAP_EXACT_TOKENS_FOR_TOKENS_DISPLAY_HASH;

    function clearCall() external payable returns (bytes memory) {
        require(msg.data.length >= 40, "clearCall: payload too short");

        bytes32 displayId = bytes32(msg.data[4:36]);
        bytes4 selector = bytes4(msg.data[36:40]);

        bytes32 expected = _expectedDisplayId(selector);
        require(expected != bytes32(0), "clearCall: unknown selector");
        require(displayId == expected, "clearCall: display identifier mismatch");

        (bool success, bytes memory result) = address(this).delegatecall(msg.data[36:]);
        if (!success) {
            assembly {revert(add(32, result), mload(result))}
        }
        return result;
    }

    function _expectedDisplayId(
        bytes4 selector
    ) private pure returns (bytes32 displayId) {
        if (selector == IUniswapV2Router.swapExactTokensForTokens.selector) {
            displayId = SWAP_EXACT_TOKENS_FOR_TOKENS_DISPLAY_HASH;
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
