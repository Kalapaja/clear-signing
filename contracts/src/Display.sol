// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

library Display {
    bytes32 public constant ENTRY_TH = keccak256("Entry(string key,string value)");
    bytes32 public constant LABELS_TH = keccak256(
        "Labels(string locale,Entry[] items)Entry(string key,string value)"
    );
    bytes32 public constant FIELD_TH = keccak256(
        "Field(string title,string description,string format,string[] case,Entry[] params,Field[] fields)Entry(string key,string value)"
    );
    bytes32 public constant DISPLAY_TH = keccak256(
        "Display(string abi,string title,string description,Field[] fields,Labels[] labels)Entry(string key,string value)Field(string title,string description,string format,string[] case,Entry[] params,Field[] fields)Labels(string locale,Entry[] items)"
    );

    function display(
        string memory abi_,
        string memory title,
        string memory description,
        bytes memory fields,
        bytes memory labels
    ) internal pure returns (bytes32) {
        return keccak256(
            abi.encode(
                DISPLAY_TH,
                keccak256(bytes(abi_)),
                keccak256(bytes(title)),
                keccak256(bytes(description)),
                keccak256(fields),
                keccak256(labels)
            )
        );
    }

    function entry(
        string memory key,
        string memory value
    ) internal pure returns (bytes32) {
        return keccak256(
            abi.encode(
                ENTRY_TH,
                keccak256(bytes(key)),
                keccak256(bytes(value))
            )
        );
    }

    function labels(
        string memory locale,
        bytes memory items
    ) internal pure returns (bytes32) {
        return keccak256(
            abi.encode(
                LABELS_TH,
                keccak256(bytes(locale)),
                keccak256(items)
            )
        );
    }

    function field(
        string memory title,
        string memory description,
        string memory format,
        bytes memory case_,
        bytes memory params,
        bytes memory fields
    ) internal pure returns (bytes32) {
        return keccak256(
            abi.encode(
                FIELD_TH,
                keccak256(bytes(title)),
                keccak256(bytes(description)),
                keccak256(bytes(format)),
                keccak256(case_),
                keccak256(params),
                keccak256(fields)
            )
        );
    }

    function booleanField(
        string memory title,
        string memory description,
        bytes memory case_,
        string memory value
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "boolean",
            case_,
            abi.encodePacked(entry("value", value)),
            ""
        );
    }

    function tokenAmountField(
        string memory title,
        string memory description,
        bytes memory case_,
        string memory token,
        string memory amount
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "tokenAmount",
            case_,
            abi.encodePacked(
                entry("token", token),
                entry("amount", amount)
            ),
            ""
        );
    }

    function nativeAmountField(
        string memory title,
        string memory description,
        bytes memory case_,
        string memory amount
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "nativeAmount",
            case_,
            abi.encodePacked(entry("amount", amount)),
            ""
        );
    }

    function callField(
        string memory title,
        string memory description,
        bytes memory case_,
        string memory target,
        string memory value,
        string memory data
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "call",
            case_,
            abi.encodePacked(
                entry("target", target),
                entry("value", value),
                entry("data", data)
            ),
            ""
        );
    }

    function addressField(
        string memory title,
        string memory description,
        bytes memory case_,
        string memory address_
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "address",
            case_,
            abi.encodePacked(entry("value", address_)),
            ""
        );
    }

    function datetimeField(
        string memory title,
        string memory description,
        bytes memory case_,
        string memory timestamp
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "datetime",
            case_,
            abi.encodePacked(entry("value", timestamp)),
            ""
        );
    }

    function durationField(
        string memory title,
        string memory description,
        bytes memory case_,
        string memory seconds_
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "duration",
            case_,
            abi.encodePacked(entry("value", seconds_)),
            ""
        );
    }

    function percentageField(
        string memory title,
        string memory description,
        bytes memory case_,
        string memory value,
        string memory basis
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "percentage",
            case_,
            abi.encodePacked(
                entry("value", value),
                entry("basis", basis)
            ),
            ""
        );
    }

    function bitmaskField(
        string memory title,
        string memory description,
        bytes memory case_,
        string memory value,
        bytes memory bitLabels
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "bitmask",
            case_,
            abi.encodePacked(
                entry("value", value),
                bitLabels
            ),
            ""
        );
    }

    function matchField(
        string memory title,
        string memory description,
        bytes memory case_,
        bytes memory params,
        bytes memory fields
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "match",
            case_,
            params,
            fields
        );
    }

    function arrayField(
        string memory title,
        string memory description,
        bytes memory case_,
        bytes memory params,
        bytes memory fields
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "array",
            case_,
            params,
            fields
        );
    }
}