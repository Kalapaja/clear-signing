// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import {Test, console2} from "forge-std/Test.sol";
import {ClearCallRouter} from "../src/ClearCallRouter.sol";
import {IUniswapV2Router} from "../src/IUniswapV2Router.sol";
import {SwapExactTokensForTokensDisplayHash} from "../src/SwapExactTokensForTokensDisplayHash.sol";

// Helper contract to receive calls from the router
contract ExternalContract {
    event CallReceived(address indexed sender);

    function callMe() external {
        emit CallReceived(msg.sender);
    }
}

contract TestableClearCallRouter is ClearCallRouter {
    enum TestMode {
        EmitEvent,
        ExternalCall,
        Revert,
        ReturnSpecific
    }

    // Mode for controlling swapExactTokensForTokens behavior
    TestMode public testMode;
    
    address public externalContract;

    event SenderLog(address indexed sender);

    function setTestMode(TestMode mode) external {
        testMode = mode;
    }

    function setExternalContract(address _addr) external {
        externalContract = _addr;
    }

    function swapExactTokensForTokens(
        uint /* amountIn */,
        uint /* amountOutMin */,
        address[] calldata /* path */,
        address /* to */,
        uint /* deadline */
    ) external override returns (uint[] memory amounts) {
        // EmitEvent: Emit event
        if (testMode == TestMode.EmitEvent) {
            emit SenderLog(msg.sender);
            uint[] memory ret = new uint[](2);
            return ret;
        }
        // ExternalCall: External Call
        else if (testMode == TestMode.ExternalCall) {
            ExternalContract(externalContract).callMe();
            uint[] memory ret = new uint[](2);
            return ret;
        }
        // Revert: Revert
        else if (testMode == TestMode.Revert) {
            revert("Custom Revert Message");
        }
        // ReturnSpecific: Return specific data 
        else if (testMode == TestMode.ReturnSpecific) {
             uint[] memory ret = new uint[](2);
             ret[0] = 123;
             ret[1] = 456;
             return ret;
        }
        
        return new uint[](0);
    }
}

contract ClearCallRouterTest is Test {
    TestableClearCallRouter router;
    ExternalContract externalContract;

    bytes32 displayHash;

    event SenderLog(address indexed sender);
    event CallReceived(address indexed sender);

    function setUp() public {
        router = new TestableClearCallRouter();
        externalContract = new ExternalContract();
        router.setExternalContract(address(externalContract));

        // Router calculates its own display hash on construction/init
        displayHash = router.SWAP_EXACT_TOKENS_FOR_TOKENS_DISPLAY_HASH();

        // Warm up the router address to avoid cold account access costs skewing results
        address(router).call("");
    }

    // Helper to construct valid clearCall calldata
    function getClearCallData(
        uint256 amountIn,
        uint256 amountOutMin,
        address[] memory path,
        address to,
        uint256 deadline
    ) internal view returns (bytes memory) {
        bytes memory innerCall = abi.encodeWithSelector(
            IUniswapV2Router.swapExactTokensForTokens.selector, amountIn, amountOutMin, path, to, deadline
        );

        return abi.encodeWithSelector(ClearCallRouter.clearCall.selector, displayHash, innerCall);
    }

    // 1. msg.sender Preservation
    function test_MsgSenderInEventEmission() public {
        router.setTestMode(TestableClearCallRouter.TestMode.EmitEvent);

        address[] memory path = new address[](2);
        bytes memory callData = getClearCallData(100, 90, path, address(this), block.timestamp);

        vm.expectEmit(true, false, false, false);
        emit SenderLog(address(this));

        (bool success,) = address(router).call(callData);
        assertTrue(success, "Call failed");
    }

    // 2. msg.value Preservation
    function test_MsgValueAccessible() public {
        router.setTestMode(TestableClearCallRouter.TestMode.EmitEvent);

        address[] memory path = new address[](2);
        bytes memory callData = getClearCallData(100, 90, path, address(this), block.timestamp);

        // Case 1: Value = 0
        vm.expectEmit(true, false, false, false);
        emit SenderLog(address(this));
        
        (bool success,) = address(router).call{value: 0}(callData);
        assertTrue(success, "Call failed with 0 value");

        // Case 2: Value > 0
        // Because swapExactTokensForTokens is non-payable, sending value MUST revert.
        // This confirms that msg.value WAS preserved and passed to the delegate.
        // If msg.value was lost (became 0), this call would succeed.
        uint256 valueToSend = 1 ether;
        (success,) = address(router).call{value: valueToSend}(callData);
        assertFalse(success, "Call should fail with value > 0 due to non-payable target");
    }

    // 3. External Calls (The Real Gotcha)
    function test_ExternalCallMsgSenderIsContract() public {
        router.setTestMode(TestableClearCallRouter.TestMode.ExternalCall);

        address[] memory path = new address[](2);
        bytes memory callData = getClearCallData(100, 90, path, address(this), block.timestamp);

        // We expect CallReceived emitted by ExternalContract (address(externalContract))
        // The sender logged should be the Router
        vm.expectEmit(true, false, false, false, address(externalContract));
        emit CallReceived(address(router)); 
        
        (bool success,) = address(router).call(callData);
        assertTrue(success, "Call failed");
    }

    // 4. Return/Revert Propagation
    function test_ReturnValueBubblesUp() public {
        router.setTestMode(TestableClearCallRouter.TestMode.ReturnSpecific);

        address[] memory path = new address[](2);
        bytes memory callData = getClearCallData(100, 90, path, address(this), block.timestamp);

        (bool success, bytes memory data) = address(router).call(callData);
        assertTrue(success, "Call failed");

        // clearCall returns (bytes memory), so we must decode bytes first
        bytes memory innerData = abi.decode(data, (bytes));
        uint256[] memory retAmounts = abi.decode(innerData, (uint256[]));
        assertEq(retAmounts[0], 123);
        assertEq(retAmounts[1], 456);
    }

    function test_RevertReasonBubblesUp() public {
        router.setTestMode(TestableClearCallRouter.TestMode.Revert);

        address[] memory path = new address[](2);
        bytes memory callData = getClearCallData(100, 90, path, address(this), block.timestamp);

        // We expect the revert to bubble up exactly
        (bool success, bytes memory returnData) = address(router).call(callData);
        assertFalse(success, "Call should have failed");

        // Error string "Custom Revert Message"
        // selector for Error(string) is 0x08c379a0
        bytes4 selector = bytes4(returnData);
        assertEq(selector, bytes4(keccak256("Error(string)")));

        bytes memory reasonBytes = new bytes(returnData.length - 4);
        for (uint256 i = 0; i < returnData.length - 4; i++) {
            reasonBytes[i] = returnData[i + 4];
        }
        string memory reason = abi.decode(reasonBytes, (string));
        assertEq(reason, "Custom Revert Message");
    }

    function test_DisplayHashMatchesBaseline() public pure {
        // The baseline hash was calculated using the updated EIP-712 structure with Check[][] for checks field
        // verifyingContract (router) was set to address(0) for the baseline calculation.
        // We must ensure the router used for hashing here is also address(0).
        bytes32 calculatedHash = SwapExactTokensForTokensDisplayHash.SWAP_EXACT_TOKENS_FOR_TOKENS_DISPLAY_HASH;
        bytes32 expectedHash = 0xa6f768cfe4f70ca48cf05bff1e1fe78173e5694a7c19ac87736239185ac97b80;

        assertEq(calculatedHash, expectedHash, "Display hash should match updated EIP-712 structure");
    }
}
