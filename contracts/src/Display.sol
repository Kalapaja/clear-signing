// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

bytes32 constant ENTRY_TH = keccak256("Entry(string key,string value)");
bytes32 constant CHECK_TH = keccak256("Check(string left,string op,string right)");
bytes32 constant LABELS_TH = keccak256(
    "Labels(string locale,Entry[] items)Entry(string key,string value)"
);
bytes32 constant FIELD_TH = keccak256(
    "Field(string title,string description,string format,Check[][] checks,Entry[] params,Field[] fields)Check(string left,string op,string right)Entry(string key,string value)"
);
bytes32 constant DISPLAY_TH = keccak256(
    "Display(string abi,string title,string description,Field[] fields,Labels[] labels)Check(string left,string op,string right)Entry(string key,string value)Field(string title,string description,string format,Check[][] checks,Entry[] params,Field[] fields)Labels(string locale,Entry[] items)"
);
