// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import {Test} from "forge-std/Test.sol";
import {GasMeasurement} from "../src/GasMeasurement.sol";

contract GasMeasurementTest is Test {
    GasMeasurement router;
    uint256 amount;
    address to;
    bytes callData;
    bytes clearCallDelegatecallData;
    bytes clearCallInternalData;
    bytes callDataSwap;
    bytes clearCallDelegatecallSwapData;
    bytes clearCallInternalSwapData;
    address[] path;

    function setUp() public {
        router = new GasMeasurement();
        address(router).call("");
        amount = 100;
        to = address(0x1234);
        callData = _getTransferCallData(amount, to);
        clearCallDelegatecallData = _wrapClearDelegate(router.TARGET_TRANSFER_DISPLAY_HASH(), callData);
        clearCallInternalData = _wrapClearInternal(router.TARGET_TRANSFER_DISPLAY_HASH(), callData);

        path = new address[](5);
        path[0] = address(0x1);
        path[1] = address(0x2);
        path[2] = address(0x3);
        path[3] = address(0x4);
        path[4] = address(0x5);
        callDataSwap = _getSwapCallData(100, 90, path, to, 123);
        clearCallDelegatecallSwapData = _wrapClearDelegate(router.TARGET_SWAP_DISPLAY_HASH(), callDataSwap);
        clearCallInternalSwapData = _wrapClearInternal(router.TARGET_SWAP_DISPLAY_HASH(), callDataSwap);
    }

    function test_Gas_Transfer_DirectCall() public {
        address(router).call(callData);
    }

    function test_Gas_Transfer_ClearCall_Delegatecall() public {
        address(router).call(clearCallDelegatecallData);
    }

    function test_Gas_Transfer_ClearCall_Internal() public {
        address(router).call(clearCallInternalData);
    }

    function test_Gas_Swap_DirectCall() public {
        address(router).call(callDataSwap);
    }

    function test_Gas_Swap_ClearCall_Delegatecall() public {
        address(router).call(clearCallDelegatecallSwapData);
    }

    function test_Gas_Swap_ClearCall_Internal() public {
        address(router).call(clearCallInternalSwapData);
    }

    function _wrapClearDelegate(bytes32 displayHash, bytes memory innerCall) internal view returns (bytes memory) {
        return abi.encodeWithSelector(router.clearCallDelegatecall.selector, displayHash, innerCall);
    }

    function _wrapClearInternal(bytes32 displayHash, bytes memory innerCall) internal view returns (bytes memory) {
        return abi.encodeWithSelector(router.clearCallInternal.selector, displayHash, innerCall);
    }

    function _getTransferCallData(uint256 amount, address to) internal pure returns (bytes memory) {
        return abi.encodeWithSelector(GasMeasurement.transfer.selector, amount, to);
    }

    function _getSwapCallData(
        uint256 amountIn,
        uint256 amountOutMin,
        address[] memory path,
        address to,
        uint256 deadline
    ) internal pure returns (bytes memory) {
        return abi.encodeWithSelector(GasMeasurement.swap.selector, amountIn, amountOutMin, path, to, deadline);
    }
}
