// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.10;

interface IAggregationRouterV6 {

    struct SwapDescription {
        address srcToken;
        address dstToken;
        address payable srcReceiver;
        address payable dstReceiver;
        uint256 amount;
        uint256 minReturnAmount;
        uint256 flags;
    }

    function swap(address executor, SwapDescription calldata desc, bytes calldata data) external;
}
