// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

library Display {
    bytes32 constant ENTRY_TYPEHASH =
    keccak256("Entry(string key,string value)");
    bytes32 constant LABELS_TYPEHASH =
    keccak256(
        "Labels(string locale,Entry[] items)Entry(string key,string value)"
    );
    bytes32 constant FIELD_TYPEHASH =
    keccak256(
        "Field(string title,string description,string format,Entry[] checks,Entry[] params)Entry(string key,string value)"
    );
    bytes32 constant DISPLAY_TYPEHASH =
    keccak256(
        "Display(address address,string abi,string title,string description,Field[] fields,Labels[] labels)Entry(string key,string value)Field(string title,string description,string format,Entry[] checks,Entry[] params)Labels(string locale,Entry[] items)"
    );

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

    function field(
        string memory title,
        string memory description,
        string memory format,
        bytes memory checks,
        bytes memory fields
    ) internal pure returns (bytes32) {
        return keccak256(
            abi.encode(
                Display.FIELD_TYPEHASH,
                keccak256(bytes(title)),
                keccak256(bytes(description)),
                keccak256(bytes(format)),
                keccak256(checks),
                keccak256(fields)
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
}
