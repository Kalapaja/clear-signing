// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

library Display {
    bytes32 public constant ENTRY_TH = keccak256("Entry(string key,string value)");
    bytes32 public constant CHECK_TH = keccak256("Check(string left,string op,string right)");
    bytes32 public constant LABELS_TH = keccak256(
        "Labels(string locale,Entry[] items)Entry(string key,string value)"
    );
    bytes32 public constant FIELD_TH = keccak256(
        "Field(string title,string description,string format,Check[][] checks,Entry[] params,Field[] fields)Check(string left,string op,string right)Entry(string key,string value)"
    );
    bytes32 public constant DISPLAY_TH = keccak256(
        "Display(string abi,string title,string description,Field[] fields,Labels[] labels)Check(string left,string op,string right)Entry(string key,string value)Field(string title,string description,string format,Check[][] checks,Entry[] params,Field[] fields)Labels(string locale,Entry[] items)"
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

    function check(
        string memory left,
        string memory op,
        string memory right
    ) internal pure returns (bytes32) {
        return keccak256(
            abi.encode(
                CHECK_TH,
                keccak256(bytes(left)),
                keccak256(bytes(op)),
                keccak256(bytes(right))
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
        bytes memory checks,
        bytes memory params,
        bytes memory fields
    ) internal pure returns (bytes32) {
        return keccak256(
            abi.encode(
                FIELD_TH,
                keccak256(bytes(title)),
                keccak256(bytes(description)),
                keccak256(bytes(format)),
                keccak256(checks),
                keccak256(params),
                keccak256(fields)
            )
        );
    }

    function booleanField(
        string memory title,
        string memory description,
        bytes memory checks,
        string memory value
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "boolean",
            checks,
            abi.encodePacked(entry("value", value)),
            ""
        );
    }

    function tokenAmountField(
        string memory title,
        string memory description,
        bytes memory checks,
        string memory token,
        string memory amount
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "tokenAmount",
            checks,
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
        bytes memory checks,
        string memory amount
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "nativeAmount",
            checks,
            abi.encodePacked(entry("amount", amount)),
            ""
        );
    }

    function callField(
        string memory title,
        string memory description,
        bytes memory checks,
        string memory target,
        string memory value,
        string memory data
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "call",
            checks,
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
        bytes memory checks,
        string memory address_
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "address",
            checks,
            abi.encodePacked(entry("value", address_)),
            ""
        );
    }

    function datetimeField(
        string memory title,
        string memory description,
        bytes memory checks,
        string memory timestamp
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "datetime",
            checks,
            abi.encodePacked(entry("value", timestamp)),
            ""
        );
    }

    function durationField(
        string memory title,
        string memory description,
        bytes memory checks,
        string memory seconds_
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "duration",
            checks,
            abi.encodePacked(entry("value", seconds_)),
            ""
        );
    }

    function percentageField(
        string memory title,
        string memory description,
        bytes memory checks,
        string memory value,
        string memory basis
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "percentage",
            checks,
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
        bytes memory checks,
        string memory value,
        bytes memory bitLabels
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "bitmask",
            checks,
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
        bytes memory checks,
        bytes memory params,
        bytes memory fields
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "match",
            checks,
            params,
            fields
        );
    }

    function arrayField(
        string memory title,
        string memory description,
        bytes memory checks,
        bytes memory params,
        bytes memory fields
    ) internal pure returns (bytes32) {
        return field(
            title,
            description,
            "array",
            checks,
            params,
            fields
        );
    }
}