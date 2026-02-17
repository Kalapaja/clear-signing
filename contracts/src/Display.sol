// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

library Display {
    bytes32 constant ENTRY_TYPEHASH =
    keccak256("Entry(string key,string value)");
    bytes32 constant CHECK_TYPEHASH =
    keccak256("Check(string left,string op,string right)");
    bytes32 constant LABELS_TYPEHASH =
    keccak256(
        "Labels(string locale,Entry[] items)Entry(string key,string value)"
    );
    bytes32 constant FIELD_TYPEHASH =
    keccak256(
        "Field(string title,string description,string format,Check[][] checks,Entry[] params,Field[] fields)Check(string left,string op,string right)Entry(string key,string value)"
    );
    bytes32 constant DISPLAY_TYPEHASH =
    keccak256(
        "Display(address address,string abi,string title,string description,Field[] fields,Labels[] labels)Check(string left,string op,string right)Entry(string key,string value)Field(string title,string description,string format,Check[][] checks,Entry[] params,Field[] fields)Labels(string locale,Entry[] items)"
    );

    function display(
        address address_,
        string memory abi_,
        string memory title,
        string memory description,
        bytes memory fields,
        bytes memory labels_
    ) internal pure returns (bytes32) {
        return keccak256(
            abi.encode(
                Display.DISPLAY_TYPEHASH,
                address_,
                keccak256(bytes(abi_)),
                keccak256(bytes(title)),
                keccak256(bytes(description)),
                keccak256(fields),
                keccak256(labels_)
            )
        );
    }

    function entry(
        string memory key,
        string memory value
    ) internal pure returns (bytes32) {
        return keccak256(
            abi.encode(
                Display.ENTRY_TYPEHASH,
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
                Display.CHECK_TYPEHASH,
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
                Display.LABELS_TYPEHASH,
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
                Display.FIELD_TYPEHASH,
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
