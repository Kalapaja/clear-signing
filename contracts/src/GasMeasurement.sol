// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

contract GasMeasurement {
    error InvalidSelector();
    error InvalidDisplayHash();
    error CallFailed();

    bytes4 private constant TARGET_TRANSFER_SELECTOR = GasMeasurement.transfer.selector;
    bytes4 private constant TARGET_SWAP_SELECTOR = GasMeasurement.swap.selector;

    bytes32 public constant TARGET_TRANSFER_DISPLAY_HASH =
    0x1111111111111111111111111111111111111111111111111111111111111111;
    bytes32 public constant TARGET_SWAP_DISPLAY_HASH =
    0x2222222222222222222222222222222222222222222222222222222222222222;

    function transfer(uint256 amount, address to) external pure returns (uint256) {
        return _target(amount, to);
    }

    function swap(
        uint256 amountIn,
        uint256 amountOutMin,
        address[] calldata path,
        address to,
        uint256 deadline
    ) external pure returns (uint256) {
        return _swap(amountIn, amountOutMin, path.length, to, deadline);
    }

    function clearCallDelegatecall(bytes32 displayHash, bytes calldata call) external {
        bytes4 selector = bytes4(call[0:4]);

        if (selector == TARGET_TRANSFER_SELECTOR) {
            if (displayHash != TARGET_TRANSFER_DISPLAY_HASH) {
                revert InvalidDisplayHash();
            }
        } else if (selector == TARGET_SWAP_SELECTOR) {
            if (displayHash != TARGET_SWAP_DISPLAY_HASH) {
                revert InvalidDisplayHash();
            }
        } else {
            revert InvalidSelector();
        }

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
        } else {
            assembly {
                let returndata_size := mload(returndata)
                return(add(32, returndata), returndata_size)
            }
        }
    }

    function clearCall() external payable {
        bytes32 displayHash = bytes32(msg.data[4:36]);
        bytes4 callSelector = bytes4(msg.data[36:40]);

        if (callSelector == TARGET_TRANSFER_SELECTOR) {
            if (displayHash != TARGET_TRANSFER_DISPLAY_HASH) {
                revert InvalidDisplayHash();
            }
        } else if (callSelector == TARGET_SWAP_SELECTOR) {
            if (displayHash != TARGET_SWAP_DISPLAY_HASH) {
                revert InvalidDisplayHash();
            }
        } else {
            revert InvalidSelector();
        }

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
        } else {
            assembly {
                let returndata_size := mload(returndata)
                return(add(32, returndata), returndata_size)
            }
        }
    }

    function clearCallInternal(bytes32 displayHash, bytes calldata call) external returns (uint256) {
        bytes4 selector = bytes4(call[0:4]);

        if (selector == TARGET_TRANSFER_SELECTOR) {
            if (displayHash != TARGET_TRANSFER_DISPLAY_HASH) {
                revert InvalidDisplayHash();
            }
            uint256 amount;
            address to;
            assembly {
                let dataOffset := add(call.offset, 4)
                amount := calldataload(dataOffset)
                to := calldataload(add(dataOffset, 32))
            }
            return _target(amount, to);
        }
        if (selector == TARGET_SWAP_SELECTOR) {
            if (displayHash != TARGET_SWAP_DISPLAY_HASH) {
                revert InvalidDisplayHash();
            }
            uint256 amountIn;
            uint256 amountOutMin;
            address to;
            uint256 deadline;
            uint256 pathLen;
            assembly {
                let dataOffset := add(call.offset, 4)
                amountIn := calldataload(dataOffset)
                amountOutMin := calldataload(add(dataOffset, 32))
                let pathOffset := calldataload(add(dataOffset, 64))
                to := calldataload(add(dataOffset, 96))
                deadline := calldataload(add(dataOffset, 128))
                pathLen := calldataload(add(add(call.offset, 4), pathOffset))
            }
            return _swap(amountIn, amountOutMin, pathLen, to, deadline);
        }
        revert InvalidSelector();
    }

    function _swap(
        uint256 amountIn,
        uint256 amountOutMin,
        uint256 pathLen,
        address to,
        uint256 deadline
    ) internal pure returns (uint256) {
        return amountIn + amountOutMin + pathLen + uint256(uint160(to)) + deadline;
    }

    function _target(uint256 amount, address to) internal pure returns (uint256) {
        return amount + uint256(uint160(to));
    }
}
