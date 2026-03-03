use crate::display::{Display, Field};
use crate::fields::{ClearCall, DisplayField};
use crate::format::{Format, ProcessingContext};
use crate::registry::Registry;
use crate::resolver::{Message, resolve_value};
use crate::sol::{SolFunction, SolValue, StateMutability};
use alloc::vec;
use alloc::vec::Vec;
use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::SolCall;
use alloy_sol_types::sol;

const MAX_RECURSION_DEPTH: usize = 16;

sol! {
    function clearCall(bytes32 displayHash, bytes call) payable returns (bytes);
}

fn parse_call(
    displays: &[Display],
    message: Message,
    display_hash: Option<FixedBytes<32>>,
    registry: &dyn Registry,
    level: usize,
) -> crate::Result<ClearCall> {
    anyhow::ensure!(
        registry.is_well_known_contract(&message.to),
        "Unknown contract: {}",
        message.to
    );

    let selector = message.selector()?;
    let display = find_display(displays, selector, display_hash, &message.to, registry)?;

    let function_call = SolFunction::parse(&display.abi)?;

    match function_call.state_mutability {
        StateMutability::Pure => anyhow::bail!("Function is not writeable"),
        StateMutability::View => anyhow::bail!("Function is not writeable"),
        StateMutability::NonPayable => {
            anyhow::ensure!(message.value == U256::ZERO, "Function is not payable");
        }
        StateMutability::Payable => {}
    }

    let data = function_call.decode(&message.data)?;

    let fields =
        process_fields_impl(displays, &message, &display.fields, &data, registry, level, None)?;

    Ok(ClearCall {
        title: display.title.into(),
        description: display.description.into(),
        payable: function_call.state_mutability == StateMutability::Payable
            && message.value != U256::ZERO,
        clear: display_hash.is_some(),
        fields,
        labels: display.labels.into(),
    })
}

fn find_display(
    displays: &[Display],
    selector: FixedBytes<4>,
    display_hash: Option<FixedBytes<32>>,
    address: &Address,
    registry: &dyn Registry,
) -> crate::Result<Display> {
    let display = if let Some(hash) = display_hash {
        let mut found_display = None;
        for d in displays {
            let func = SolFunction::parse(&d.abi)?;
            if func.selector() == selector && d.hash_struct() == hash {
                found_display = Some(d.clone());
                break;
            }
        }

        found_display.ok_or_else(|| {
            anyhow::anyhow!(
                "Display not found: selector {:?} at address {} with display hash {:?}",
                selector,
                address,
                hash
            )
        })?
    } else {
        registry
            .get_well_known_display(address, &selector)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Well Known Display not found: selector {:?} at address {}",
                    selector,
                    address
                )
            })?
    };

    anyhow::ensure!(display.validate(), "Display validation failed");
    Ok(display)
}

fn process_fields_impl(
    displays: &[Display],
    message: &Message,
    fields: &[Field],
    data: &SolValue,
    registry: &dyn Registry,
    level: usize,
    switch_value: Option<SolValue>,
) -> crate::Result<Vec<DisplayField>> {
    anyhow::ensure!(level <= MAX_RECURSION_DEPTH, "Max recursion depth exceeded");

    let mut display_fields = vec![];

    'fields: for field in fields {
        if let Some(ref switch_val) = switch_value {
            if !field.case.is_empty() {
                let mut matched = false;
                for case_str in &field.case {
                    let case_value = resolve_value(case_str, message, data)?;

                    match switch_val.matches(&case_value) {
                        Ok(true) => {
                            matched = true;
                            break;
                        }
                        Ok(false) => continue,
                        Err(e) => {
                            anyhow::bail!(
                                "Error matching switch value {:?} against case '{}': {}",
                                switch_val,
                                case_str,
                                e
                            );
                        }
                    }
                }
                if !matched {
                    continue 'fields;
                }
            }
        } else {
            if !field.case.is_empty() {
                anyhow::bail!(
                    "Field '{}' has case array but no switch context. Fields with case must be children of a switch format.",
                    field.title
                );
            }
        }

        let format = Format::from(&field.format)?;
        let ctx = ProcessingContext::new(
            field,
            message,
            data,
            registry,
            displays,
            level,
        )?;
        let display_field = format.process(&ctx)?;
        display_fields.push(display_field);
    }

    Ok(display_fields)
}

pub(crate) fn parse_clear_call_with_level(
    displays: &[Display],
    message: Message,
    registry: &dyn Registry,
    level: usize,
) -> crate::Result<ClearCall> {
    anyhow::ensure!(level <= MAX_RECURSION_DEPTH, "Max recursion depth exceeded");

    let (display_hash, msg) = match message.selector()? {
        FixedBytes(clearCallCall::SELECTOR) => {
            let decoded = clearCallCall::abi_decode(&message.data)?;
            (
                Some(decoded.displayHash),
                message.replace_data(decoded.call),
            )
        }
        _ => (None, message),
    };

    parse_call(displays, msg, display_hash, registry, level)
}

pub(crate) fn process_nested_fields(
    displays: &[Display],
    message: &Message,
    fields: &[Field],
    data: &SolValue,
    registry: &dyn Registry,
    level: usize,
    switch_value: Option<SolValue>,
) -> crate::Result<Vec<DisplayField>> {
    process_fields_impl(
        displays,
        message,
        fields,
        data,
        registry,
        level,
        switch_value,
    )
}


pub fn parse_clear_call(
    message: Message,
    displays: Vec<Display>,
    registry: &dyn Registry,
) -> crate::Result<ClearCall> {
    parse_clear_call_with_level(&displays, message, registry, 0)
}

#[allow(dead_code)]
pub(crate) fn entries_to_map(
    entries: &[crate::display::Entry],
) -> crate::Result<alloc::collections::BTreeMap<&str, &str>> {
    use alloc::collections::BTreeMap;
    let mut map = BTreeMap::new();
    for entry in entries {
        if map.contains_key(entry.key.as_str()) {
            anyhow::bail!("Duplicate entry key: {}", entry.key);
        }

        map.insert(entry.key.as_str(), entry.value.as_str());
    }
    Ok(map)
}

#[allow(dead_code)]
pub(crate) fn resolve_param_value(
    params: &alloc::collections::BTreeMap<&str, &str>,
    key: &str,
    message: &Message,
    data: &SolValue,
) -> crate::Result<SolValue> {
    let param_value = params
        .get(key)
        .ok_or_else(|| anyhow::anyhow!("Param {} not found", key))?;

    resolve_value(param_value, message, data)
}

#[allow(dead_code)]
pub(crate) fn resolve_optional_param_value(
    params: &alloc::collections::BTreeMap<&str, &str>,
    key: &str,
    message: &Message,
    data: &SolValue,
) -> Option<crate::Result<SolValue>> {
    let param_value = params.get(key);

    param_value.map(|param_value| resolve_value(param_value, message, data))
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;
    use crate::display::Display;
    use alloc::collections::BTreeMap;
    use alloc::string::{String, ToString};
    use alloc::vec;
    use alloy_primitives::{Address, I256, address, uint};

    pub struct LocalRegistry {
        pub well_known_displays: BTreeMap<FixedBytes<4>, Display>,
        pub well_known_contracts: Vec<Address>,
        pub well_known_tokens: Vec<Address>,
    }

    impl Registry for LocalRegistry {
        fn is_well_known_contract(&self, address: &Address) -> bool {
            self.well_known_contracts.contains(address) || self.well_known_tokens.contains(address)
        }

        fn is_well_known_token(&self, address: &Address) -> bool {
            self.well_known_tokens.contains(address)
        }

        fn get_well_known_display(
            &self,
            _address: &Address,
            selector: &FixedBytes<4>,
        ) -> Option<Display> {
            self.well_known_displays
                .get(&(*selector))
                .or_else(|| self.well_known_displays.get(&(*selector)))
                .map(|d| d.clone())
        }
    }

    fn setup_test_registry(
        contracts: Vec<Address>,
        tokens: Vec<Address>,
        displays: Vec<Display>,
    ) -> LocalRegistry {
        let mut display_map = BTreeMap::new();
        for display in displays {
            let selector = SolFunction::parse(&display.abi).unwrap().selector();
            display_map.insert(selector, display);
        }

        LocalRegistry {
            well_known_displays: display_map,
            well_known_contracts: contracts,
            well_known_tokens: tokens,
        }
    }

    sol! {
        function simpleCall(uint256 val) payable;
        function transfer(address to, uint256 value);
        function arrayCall(uint256[] nums) payable;
        function matchCall(uint256 val) payable;
        function matchIsolation(uint256 parent_val) payable;
        function abiCall(bytes data) payable;
        function aggregate((address, bytes)[] calls);
        function nestedCall(
            address call_target,
            bytes call_data,
            uint256 call_value,
            address erc20_target,
            bytes erc20_data
        ) payable;
        function complexCall(
            address addr,
            uint256 uint_val,
            int256 int_val,
            bool bool_val,
            string string_val,
            bytes bytes_val,
            uint256 percentage_val,
            uint256 basis_val,
            uint256 duration_val,
            uint256 bitmask_val
        ) payable;
    }

    #[derive(serde::Deserialize)]
    struct DisplaySpecFile {
        displays: Vec<Display>,
    }

    fn get_display(title: &str) -> Display {
        let json = include_str!("displays_test.json");
        let displays: DisplaySpecFile =
            serde_json::from_str(json).expect("Failed to parse display_test.json");
        displays
            .displays
            .into_iter()
            .find(|d| d.title == title)
            .unwrap_or_else(|| panic!("Display with title '{}' not found", title))
    }

    struct TestFixtures {
        target: Address,
        param_addr: Address,
        main_token: Address,
        inner_target: Address,
        erc20_token: Address,
        receiver_addr: Address,
        inner_display: Display,
        erc20_display: Display,
        inner_abi: String,
    }

    impl TestFixtures {
        fn new() -> Self {
            let target = address!("0000000000000000000000000000000000000001");
            let param_addr = address!("0000000000000000000000000000000000000002");
            let main_token = address!("0000000000000000000000000000000000000003");
            let inner_target = address!("0000000000000000000000000000000000000004");
            let erc20_token = address!("0000000000000000000000000000000000000005");
            let receiver_addr = address!("0000000000000000000000000000000000000006");

            let inner_display = get_display("Inner");
            let inner_abi = inner_display.abi.clone();

            let erc20_display = get_display("ERC20");

            Self {
                target,
                param_addr,
                main_token,
                inner_target,
                erc20_token,
                receiver_addr,
                inner_display,
                erc20_display,
                inner_abi,
            }
        }
    }

    #[test]
    fn test_parse_clear_call_comprehensive() {
        let f = TestFixtures::new();

        let call_args = complexCallCall {
            addr: f.param_addr,
            uint_val: uint!(123_U256),
            int_val: I256::from_raw(uint!(456_U256)),
            bool_val: true,
            string_val: "Hello".to_string(),
            bytes_val: vec![0xca, 0xfe].into(),
            percentage_val: uint!(75_U256),
            basis_val: uint!(100_U256),
            duration_val: uint!(3600_U256),
            bitmask_val: uint!(9_U256),
        };

        let display = get_display("Complex");
        let abi = display.abi.clone();
        let complex_data = call_args.abi_encode();
        let display_hash = display.hash_struct();
        let clear_data = clearCallCall {
            displayHash: display_hash,
            call: complex_data.into(),
        }
        .abi_encode();

        let sol_func = SolFunction::parse(&abi).unwrap();
        assert_eq!(
            sol_func.selector(),
            complexCallCall::SELECTOR,
            "Top-level selector mismatch"
        );

        let inner_sol_func = SolFunction::parse(&f.inner_abi).unwrap();
        assert_eq!(
            inner_sol_func.selector(),
            simpleCallCall::SELECTOR,
            "Nested selector mismatch"
        );

        let message = Message::new(
            address!("0000000000000000000000000000000000000000"),
            f.target,
            U256::ZERO,
            clear_data.into(),
        );

        let registry = setup_test_registry(
            vec![f.target, f.main_token, f.param_addr],
            vec![f.main_token],
            vec![],
        );

        let result = parse_clear_call(message, vec![display, f.inner_display], &registry)
            .expect("Failed to parse clear call");

        assert_eq!(result.fields.len(), 14);

        match &result.fields[0] {
            DisplayField::Address { title, value, .. } => {
                assert_eq!(title, "Addr Title");
                assert_eq!(*value, f.param_addr);
            }
            _ => panic!(),
        }
        match &result.fields[1] {
            DisplayField::Uint { title, value, .. } => {
                assert_eq!(title, "Uint Title");
                assert_eq!(*value, uint!(123_U256));
            }
            _ => panic!(),
        }
        match &result.fields[2] {
            DisplayField::Int { value, .. } => assert_eq!(*value, I256::from_raw(uint!(456_U256))),
            _ => panic!(),
        }
        match &result.fields[3] {
            DisplayField::Boolean { value, .. } => assert!(*value),
            _ => panic!(),
        }
        match &result.fields[4] {
            DisplayField::String { value, .. } => assert_eq!(value, "Hello"),
            _ => panic!(),
        }
        match &result.fields[5] {
            DisplayField::Bytes { value, .. } => assert_eq!(value.as_ref(), &[0xca, 0xfe]),
            _ => panic!(),
        }
        match &result.fields[6] {
            DisplayField::Percentage { value, basis, .. } => {
                assert_eq!(*value, uint!(75_U256));
                assert_eq!(*basis, uint!(100_U256));
            }
            _ => panic!(),
        }
        match &result.fields[7] {
            DisplayField::Duration { value, .. } => assert_eq!(value.as_secs(), 3600),
            _ => panic!(),
        }
        match &result.fields[8] {
            DisplayField::Datetime { value, .. } => assert_eq!(value.as_secs(), 3600),
            _ => panic!(),
        }
        match &result.fields[9] {
            DisplayField::Bitmask { values, .. } => {
                assert_eq!(values.len(), 2);
                assert_eq!(values[0], "Bit0");
                assert_eq!(values[1], "Bit3 Label");
            }
            _ => panic!(),
        }
        match &result.fields[10] {
            DisplayField::Token { token, .. } => assert_eq!(*token, f.main_token),
            _ => panic!(),
        }
        match &result.fields[11] {
            DisplayField::Contract { contract, .. } => assert_eq!(*contract, f.param_addr),
            _ => panic!(),
        }
        match &result.fields[12] {
            DisplayField::TokenAmount { token, amount, .. } => {
                assert_eq!(*token, f.main_token);
                assert_eq!(*amount, uint!(123_U256));
            }
            _ => panic!(),
        }
        match &result.fields[13] {
            DisplayField::NativeAmount { amount, .. } => assert_eq!(*amount, uint!(123_U256)),
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_clear_call_nested() {
        let f = TestFixtures::new();
        let display_addr = f.target;

        let display = get_display("Nested");

        let registry = setup_test_registry(
            vec![display_addr, f.inner_target],
            vec![f.erc20_token],
            vec![(display), (f.erc20_display)],
        );

        let inner_call_data = simpleCallCall {
            val: uint!(42_U256),
        }
        .abi_encode();

        let inner_display_hash = f.inner_display.hash_struct();
        let inner_wrapped_data = clearCallCall {
            displayHash: inner_display_hash,
            call: inner_call_data.into(),
        }
        .abi_encode();

        let erc20_call_data = transferCall {
            to: f.receiver_addr,
            value: uint!(999_U256),
        }
        .abi_encode();

        let call_args = nestedCallCall {
            call_target: f.inner_target,
            call_data: inner_wrapped_data.into(),
            call_value: uint!(1000_U256),
            erc20_target: f.erc20_token,
            erc20_data: erc20_call_data.into(),
        };
        let nested_data = call_args.abi_encode();

        let message = Message {
            sender: display_addr,
            to: display_addr,
            value: U256::ZERO,
            data: nested_data.into(),
        };

        let result = parse_clear_call(message, vec![f.inner_display.clone()], &registry)
            .expect("Failed to parse");

        assert_eq!(result.fields.len(), 2);

        match &result.fields[0] {
            DisplayField::Call { call, .. } => {
                assert_eq!(call.title, "Inner");
                match &call.fields[0] {
                    DisplayField::Uint { value, .. } => assert_eq!(*value, uint!(42_U256)),
                    _ => panic!(),
                }
            }
            _ => panic!("Expected Call field"),
        }
        match &result.fields[1] {
            DisplayField::Call { call, .. } => {
                assert_eq!(call.title, "ERC20");
                assert_eq!(call.fields.len(), 3);
                match &call.fields[0] {
                    DisplayField::Address { value, .. } => assert_eq!(*value, display_addr),
                    _ => panic!(),
                }
                match &call.fields[1] {
                    DisplayField::Address { value, .. } => assert_eq!(*value, f.receiver_addr),
                    _ => panic!(),
                }
                match &call.fields[2] {
                    DisplayField::TokenAmount { token, amount, .. } => {
                        assert_eq!(*token, f.erc20_token);
                        assert_eq!(*amount, uint!(999_U256));
                    }
                    _ => panic!(),
                }
            }
            _ => panic!("Expected ERC20 Call field"),
        }
    }

    #[test]
    fn test_parse_clear_call_match() {
        let f = TestFixtures::new();
        let display_addr = f.target;

        let display = get_display("Match Test");

        let registry = setup_test_registry(vec![display_addr], vec![], vec![(display)]);

        let call_data = matchCallCall { val: uint!(1_U256) }.abi_encode();
        let message = Message {
            sender: Address::ZERO,
            to: display_addr,
            value: U256::ZERO,
            data: call_data.into(),
        };

        let result = parse_clear_call(message, vec![], &registry)
            .expect("Failed to parse");

        assert_eq!(result.fields.len(), 1, "Expected 1 field in result");

        match &result.fields[0] {
            DisplayField::Match { values, .. } => {
                assert_eq!(values.len(), 1, "Expected 1 inner field in Match");
                match &values[0] {
                    DisplayField::Uint { value, .. } => {
                        assert_eq!(*value, uint!(100_U256));
                    }
                    _ => panic!("Expected Uint field inside Match"),
                }
            }
            _ => panic!("Expected Match field"),
        }
    }

    #[test]
    fn test_parse_clear_call_array() {
        let display_addr = address!("0000000000000000000000000000000000000001");

        let display = get_display("Array");

        let registry = setup_test_registry(vec![display_addr], vec![], vec![display]);

        let call_data = arrayCallCall {
            nums: vec![uint!(10_U256), uint!(20_U256)],
        }
        .abi_encode();
        let message = Message {
            sender: Address::ZERO,
            to: display_addr,
            value: U256::ZERO,
            data: call_data.into(),
        };

        let result = parse_clear_call(message, vec![], &registry)
            .expect("Failed to parse");

        assert_eq!(result.fields.len(), 1, "Expected 1 top-level field (Array)");

        match &result.fields[0] {
            DisplayField::Array { values, .. } => {
                assert_eq!(values.len(), 2, "Expected 2 items in array");
                assert_eq!(values[0].len(), 1, "Expected 1 field in first item");
                match &values[0][0] {
                    DisplayField::Uint { value, .. } => assert_eq!(*value, uint!(10_U256)),
                    _ => panic!("Expected Uint 10"),
                }
                assert_eq!(values[1].len(), 1, "Expected 1 field in second item");
                match &values[1][0] {
                    DisplayField::Uint { value, .. } => assert_eq!(*value, uint!(20_U256)),
                    _ => panic!("Expected Uint 20"),
                }
            }
            _ => panic!("Expected Array field"),
        }
    }

    #[test]
    fn test_parse_clear_call_abi() {
        let display_addr = address!("0000000000000000000000000000000000000001");
        let recipient = address!("0000000000000000000000000000000000000002");
        let amount = uint!(100_U256);

        let display = get_display("Abi Test");

        let registry = setup_test_registry(vec![display_addr], vec![], vec![display]);

        let mut encoded_params = vec![0u8; 64];
        encoded_params[12..32].copy_from_slice(recipient.as_slice());
        encoded_params[32..64].copy_from_slice(&amount.to_be_bytes::<32>());

        let call_data = abiCallCall {
            data: encoded_params.into(),
        }
        .abi_encode();
        let message = Message {
            sender: Address::ZERO,
            to: display_addr,
            value: U256::ZERO,
            data: call_data.into(),
        };

        let result = parse_clear_call(message, vec![], &registry)
            .expect("Failed to parse");

        assert_eq!(result.fields.len(), 1, "Expected 1 field in result");

        match &result.fields[0] {
            DisplayField::Match { values, .. } => {
                assert_eq!(values.len(), 2, "Expected 2 inner fields in Abi Match");

                match &values[0] {
                    DisplayField::Address { value, .. } => {
                        assert_eq!(*value, recipient);
                    }
                    _ => panic!("Expected Address field as first inner field"),
                }

                match &values[1] {
                    DisplayField::Uint { value, .. } => {
                        assert_eq!(*value, amount);
                    }
                    _ => panic!("Expected Uint field as second inner field"),
                }
            }
            _ => panic!("Expected Match field at index 0"),
        }
    }

    #[test]
    fn test_parse_clear_call_abi_new_locals() {
        let display_addr = address!("0000000000000000000000000000000000000001");
        let sender = Address::ZERO;
        let recipient = address!("0000000000000000000000000000000000000002");
        let amount = uint!(100_U256);

        let display = get_display("Abi Test New Locals");

        let registry = setup_test_registry(vec![display_addr], vec![], vec![display]);

        let mut encoded_params = vec![0u8; 64];
        encoded_params[12..32].copy_from_slice(recipient.as_slice());
        encoded_params[32..64].copy_from_slice(&amount.to_be_bytes::<32>());

        let call_data = abiCallCall {
            data: encoded_params.into(),
        }
        .abi_encode();
        let message = Message {
            sender,
            to: display_addr,
            value: U256::ZERO,
            data: call_data.into(),
        };

        let result = parse_clear_call(message, vec![], &registry)
            .expect("Failed to parse");

        assert_eq!(result.fields.len(), 1, "Expected 1 field in result");

        match &result.fields[0] {
            DisplayField::Match { values, .. } => {
                assert_eq!(values.len(), 3, "Expected 3 inner fields in Abi Match");

                match &values[0] {
                    DisplayField::Address { value, .. } => {
                        assert_eq!(*value, sender);
                    }
                    _ => panic!("Expected Address field as first inner field"),
                }

                match &values[1] {
                    DisplayField::Address { value, .. } => {
                        assert_eq!(*value, recipient);
                    }
                    _ => panic!("Expected Address field as second inner field"),
                }

                match &values[2] {
                    DisplayField::Uint { value, .. } => {
                        assert_eq!(*value, amount);
                    }
                    _ => panic!("Expected Uint field as third inner field"),
                }
            }
            _ => panic!("Expected Match field at index 0"),
        }
    }

    #[test]
    fn test_parse_clear_call_multicall() {
        let token = address!("0000000000000000000000000000000000000001");
        let multicall_address = address!("0000000000000000000000000000000000000002");

        let multilcall_display = get_display("Multicall3");
        let erc20_display = get_display("ERC20");

        let multicall = aggregateCall {
            calls: vec![
                (
                    token,
                    transferCall {
                        to: token,
                        value: uint!(10_U256),
                    }
                    .abi_encode()
                    .into(),
                ),
                (
                    token,
                    transferCall {
                        to: token,
                        value: uint!(20_U256),
                    }
                    .abi_encode()
                    .into(),
                ),
                (
                    token,
                    transferCall {
                        to: token,
                        value: uint!(30_U256),
                    }
                    .abi_encode()
                    .into(),
                ),
            ],
        };

        let registry = setup_test_registry(
            vec![multicall_address],
            vec![token],
            vec![multilcall_display.clone(), erc20_display],
        );

        let message = Message {
            sender: Address::ZERO,
            to: multicall_address,
            value: U256::ZERO,
            data: multicall.abi_encode().into(),
        };

        let result = parse_clear_call(message, vec![], &registry).unwrap();

        assert_eq!(result.fields.len(), 1);
        match &result.fields[0] {
            DisplayField::Array { values, .. } => {
                assert_eq!(values.len(), 3);
                for (i, item_fields) in values.iter().enumerate() {
                    assert_eq!(item_fields.len(), 1);
                    match &item_fields[0] {
                        DisplayField::Call { call, .. } => {
                            assert_eq!(call.title, "ERC20");
                            assert_eq!(call.fields.len(), 3);

                            match &call.fields[0] {
                                DisplayField::Address { value, .. } => {
                                    assert_eq!(*value, multicall_address);
                                }
                                _ => panic!("Expected Address field for Sender"),
                            }

                            match &call.fields[1] {
                                DisplayField::Address { value, .. } => {
                                    assert_eq!(*value, token);
                                }
                                _ => panic!("Expected Address field for Receiver"),
                            }

                            match &call.fields[2] {
                                DisplayField::TokenAmount {
                                    token: t, amount, ..
                                } => {
                                    assert_eq!(*t, token);
                                    assert_eq!(*amount, U256::from((i + 1) * 10));
                                }
                                _ => panic!("Expected TokenAmount field"),
                            }
                        }
                        _ => panic!("Expected Call field"),
                    }
                }
            }
            _ => panic!("Expected Array field"),
        }
    }
}
