use crate::display::{Check, Display, Field, Format};
use crate::error::ParseError;
use crate::error::ParseError::{DisplayNotFound, ParamNotFound, SmthWentWrong};
use crate::fields::{ClearCall, Direction, DisplayField, Label};
use crate::registry::Registry;
use crate::resolver::{resolve_value, Message};
use crate::sol::{SolFunction, SolType, SolValue, StateMutability};
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};
use alloy_dyn_abi::DynSolType;
use alloy_primitives::{address, Address, FixedBytes, U256};
use alloy_sol_types::sol;
use alloy_sol_types::SolCall;
use core::time::Duration;

/// Maximum recursion depth for nested calls to prevent stack overflow
const MAX_RECURSION_DEPTH: usize = 16;

/// Comparison operators for check evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckOperator {
    /// Equal (==)
    Eq,
    /// Not equal (!=)
    Ne,
}

impl CheckOperator {
    /// Parse operator from string, defaulting to Eq if empty
    fn from_str(s: &str) -> Result<Self, ParseError> {
        match s {
            "" | "eq" => Ok(CheckOperator::Eq),
            "ne" => Ok(CheckOperator::Ne),
            _ => Err(ParseError::UnknownOperator(s.to_string())),
        }
    }

    /// Evaluate the operator against two values
    fn evaluate(&self, left: &SolValue, right: &SolValue) -> Result<bool, ParseError> {
        let matches = left.matches(right)?;
        Ok(match self {
            CheckOperator::Eq => matches,
            CheckOperator::Ne => !matches,
        })
    }
}

sol! {
    function clearCall(bytes32 displayHash, bytes call) payable returns (bytes);
}

pub struct ClearCallContext {
    displays: Vec<Display>,
}

impl ClearCallContext {
    pub fn new(displays: Vec<Display>) -> Self {
        Self { displays }
    }

    pub fn parse_clear_call(
        &self,
        message: Message,
        registry: &dyn Registry,
        level: usize,
    ) -> Result<ClearCall, ParseError> {
        if level > MAX_RECURSION_DEPTH {
            return Err(ParseError::RecursionLimitExceeded);
        }

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

        self.parse_call(msg, display_hash, registry, level)
    }

    fn parse_call(
        &self,
        message: Message,
        display_hash: Option<FixedBytes<32>>,
        registry: &dyn Registry,
        level: usize,
    ) -> Result<ClearCall, ParseError> {
        if !registry.is_well_known_contract(&message.to) {
            return Err(ParseError::UnknownContract(message.to));
        }

        let display = if let Some(hash) = display_hash {

            let selector = message.selector()?;

            for display in &self.displays {
                if !display.validate() {
                    return Err(SmthWentWrong("Display validation failed".to_string()));
                }
            }

            let display = self
                .displays
                .iter()
                .find(|d| {
                    if let Ok(func) = SolFunction::parse(&d.abi) {
                        func.selector() == selector && d.hash_struct() == hash
                    } else {
                        false
                    }
                })
                .ok_or(DisplayNotFound {
                    address: message.to,
                    selector,
                    display_hash: Some(self.displays.first().unwrap().hash_struct()),
                })?
                .clone();

            display
        } else {
            let selector = message.selector()?;

            let well_known_display = registry
                .get_well_known_display(&message.to, &selector)
                .ok_or(DisplayNotFound {
                    address: message.to,
                    selector,
                    display_hash: None,
                })?;

            if !well_known_display.validate() {
                return Err(SmthWentWrong("Display validation failed".to_string()));
            }

            well_known_display
        };

        let function_call = SolFunction::parse(&display.abi)?;

        match function_call.state_mutability {
            StateMutability::Pure => return Err(ParseError::FunctionNotWriteable),
            StateMutability::View => return Err(ParseError::FunctionNotWriteable),
            StateMutability::Payable => {}
            StateMutability::NonPayable => {
                if message.value != U256::ZERO {
                    return Err(ParseError::FunctionNotPayable);
                }
            }
        }

        let locals = function_call.decode(&message.data)?;

        let fields = self.process_fields(&message, &display.fields, &locals, registry, level)?;

        Ok(ClearCall {
            title: display.title.clone(),
            description: display.description.clone(),
            payable: function_call.state_mutability == StateMutability::Payable
                && message.value != U256::ZERO,
            clear: display_hash.is_some(),
            fields,
            labels: display.labels.clone(),
        })
    }

    fn process_fields(
        &self,
        message: &Message,
        fields: &[Field],
        locals: &SolValue,
        registry: &dyn Registry,
        level: usize,
    ) -> Result<Vec<DisplayField>, ParseError> {
        if level > MAX_RECURSION_DEPTH {
            return Err(ParseError::RecursionLimitExceeded);
        }

        let mut display_fields = vec![];

        'fields: for field in fields {
            let params = entries_to_map(&field.params)?;
            let title = field.title.clone();
            let description = field.description.clone();

            // Evaluate checks
            if !field.checks.is_empty() && !evaluate_checks(&field.checks, message, locals)? {
                continue 'fields; // Skip field if checks don't pass
            }

            match Format::from(&field.format)? {
                Format::Address => {
                    let value =
                        resolve_param_value(&params, "value", message, locals)?.as_address()?;

                    display_fields.push(DisplayField::Address {
                        title,
                        description,
                        value,
                    });
                }
                Format::TokenAmount => {
                    let amount =
                        resolve_param_value(&params, "amount", message, locals)?.as_uint()?;
                    let token =
                        resolve_param_value(&params, "token", message, locals)?.as_address()?;
                    let direction =
                        resolve_optional_param_value(&params, "direction", message, locals);

                    let direction = if let Some(direction) = direction {
                        Some(Direction::from_str(direction?.as_literal()?.as_str())?)
                    } else {
                        None
                    };

                    let natives = vec![
                        address!("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
                        address!("0x0000000000000000000000000000000000000000"),
                    ];
                    if natives.contains(&token) {
                        display_fields.push(DisplayField::NativeAmount {
                            title,
                            description,
                            amount,
                            direction,
                        })
                    } else if registry.is_well_known_token(&token) {
                        display_fields.push(DisplayField::TokenAmount {
                            title,
                            description,
                            token,
                            amount,
                            direction,
                        })
                    } else {
                        return Err(ParseError::UnknownToken(token));
                    }
                }
                Format::NativeAmount => {
                    let amount =
                        resolve_param_value(&params, "amount", message, locals)?.as_uint()?;
                    let direction =
                        resolve_optional_param_value(&params, "direction", message, locals);

                    let direction = if let Some(direction) = direction {
                        Some(Direction::from_str(direction?.as_literal()?.as_str())?)
                    } else {
                        None
                    };

                    display_fields.push(DisplayField::NativeAmount {
                        title,
                        description,
                        amount,
                        direction,
                    });
                }
                Format::Contract => {
                    let contract =
                        resolve_param_value(&params, "value", message, locals)?.as_address()?;

                    if !registry.is_well_known_contract(&contract) {
                        return Err(ParseError::UnknownContract(contract));
                    }

                    display_fields.push(DisplayField::Contract {
                        title,
                        description,
                        contract,
                    });
                }
                Format::Token => {
                    let token =
                        resolve_param_value(&params, "value", message, locals)?.as_address()?;

                    if !registry.is_well_known_token(&token) {
                        return Err(ParseError::UnknownContract(token));
                    }

                    display_fields.push(DisplayField::Token {
                        title,
                        description,
                        token,
                    });
                }
                Format::Bytes => {
                    let value =
                        resolve_param_value(&params, "value", message, locals)?.as_bytes()?;

                    display_fields.push(DisplayField::Bytes {
                        title,
                        description,
                        value: value.into(),
                    });
                }
                Format::String => {
                    let value =
                        resolve_param_value(&params, "value", message, locals)?.as_string()?;

                    display_fields.push(DisplayField::String {
                        title,
                        description,
                        value,
                    });
                }
                Format::Call => {
                    let to = resolve_param_value(&params, "to", message, locals)?.as_address()?;
                    let data = resolve_param_value(&params, "data", message, locals)?.as_bytes()?;
                    let value =
                        resolve_param_value(&params, "value", message, locals)?.as_uint()?;

                    let msg = Message::new(message.to, to, value, data.into());
                    let call = self.parse_clear_call(msg, registry, level + 1)?;

                    display_fields.push(DisplayField::Call {
                        title,
                        description,
                        call,
                    });
                }
                Format::Boolean => {
                    let value =
                        resolve_param_value(&params, "value", message, locals)?.as_bool()?;
                    display_fields.push(DisplayField::Boolean {
                        title,
                        description,
                        value,
                    });
                }
                Format::Int => {
                    let value = resolve_param_value(&params, "value", message, locals)?.as_int()?;
                    display_fields.push(DisplayField::Int {
                        title,
                        description,
                        value,
                    });
                }
                Format::Uint => {
                    let value =
                        resolve_param_value(&params, "value", message, locals)?.as_uint()?;

                    display_fields.push(DisplayField::Uint {
                        title,
                        description,
                        value,
                    });
                }
                Format::Duration => {
                    let value =
                        resolve_param_value(&params, "value", message, locals)?.as_uint()?;

                    display_fields.push(DisplayField::Duration {
                        title,
                        description,
                        value: Duration::from_secs(value.try_into()?),
                    });
                }
                Format::Datetime => {
                    let value =
                        resolve_param_value(&params, "value", message, locals)?.as_uint()?;

                    display_fields.push(DisplayField::Datetime {
                        title,
                        description,
                        value: Duration::from_secs(value.try_into()?),
                    });
                }
                Format::Percentage => {
                    let value =
                        resolve_param_value(&params, "value", message, locals)?.as_uint()?;
                    let basis =
                        resolve_param_value(&params, "basis", message, locals)?.as_uint()?;

                    display_fields.push(DisplayField::Percentage {
                        title,
                        description,
                        value,
                        basis,
                    });
                }
                Format::Bitmask => {
                    let value =
                        resolve_param_value(&params, "value", message, locals)?.as_uint()?;
                    let bit_indexes = map_to_values(&params, '#');

                    let mut values: Vec<Label> = vec![];

                    for entry in bit_indexes {
                        let bit_index = entry.0.parse()?;
                        if value.bit(bit_index) {
                            values.push(entry.1.to_string());
                        }
                    }

                    display_fields.push(DisplayField::Bitmask {
                        title,
                        description,
                        values,
                    });
                }
                Format::Match => {
                    let mut match_locals = vec![];
                    decode_abi_locals(&params, message, locals, &mut match_locals)?;
                    decode_params_locals(&params, message, locals, &mut match_locals)?;

                    let new_locals = SolValue::Tuple(match_locals);

                    let new_fields = self.process_fields(
                        message,
                        &field.fields,
                        &new_locals,
                        registry,
                        level + 1,
                    )?;

                    display_fields.push(DisplayField::Match {
                        title,
                        description,
                        values: new_fields,
                    });
                }
                Format::Array => {
                    let new_locals_names = map_to_values(&params, '$');

                    let mut arrays: Vec<(String, Vec<SolValue>)> = vec![];
                    let mut length = 0;

                    for (i, (name, reference)) in new_locals_names.into_iter().enumerate() {
                        let value = resolve_value(reference, message, locals)?.as_array()?;
                        if i == 0 {
                            length = value.len();
                        } else if value.len() != length {
                            return Err(SmthWentWrong("Array length mismatch".to_string()));
                        }
                        arrays.push((name, value));
                    }

                    let mut values = vec![];

                    for i in 0..length {
                        let mut tuple: Vec<(Option<String>, SolValue)> = vec![];
                        for (name, array) in &arrays {
                            tuple.push((Some(name.clone()), array[i].clone()));
                        }
                        let new_locals = SolValue::Tuple(tuple);
                        let item_fields = self.process_fields(
                            message,
                            &field.fields,
                            &new_locals,
                            registry,
                            level + 1,
                        )?;
                        values.push(item_fields);
                    }

                    display_fields.push(DisplayField::Array {
                        title,
                        description,
                        values,
                    });
                }
            };
        }

        // if fields.is_empty() {
        //     return Err(SmthWentWrong("Fields not created".to_string()));
        // }

        Ok(display_fields)
    }
}

/// Evaluate a 2D checks array with OR-of-AND logic
///
/// Returns true if at least one check group passes (where all checks in the group must pass)
///
/// **Note**: Caller should check if checks array is empty before calling this function
fn evaluate_checks(
    checks: &[Vec<Check>],
    message: &Message,
    locals: &SolValue,
) -> Result<bool, ParseError> {
    // OR logic: at least one group must pass
    for check_group in checks {
        // AND logic: all checks in this group must pass
        if evaluate_check_group(check_group, message, locals)? {
            return Ok(true); // Short-circuit on first passing group
        }
    }

    Ok(false) // No groups passed
}

/// Evaluate a single check group (all checks must pass - AND logic)
fn evaluate_check_group(
    check_group: &[Check],
    message: &Message,
    locals: &SolValue,
) -> Result<bool, ParseError> {
    for check in check_group {
        // Resolve left operand
        let left_val = resolve_value(&check.left, message, locals);

        // Handle ReferenceNotFound - treat as check failure
        if let Err(ParseError::ReferenceNotFound(_)) = left_val {
            return Ok(false);
        }

        // Resolve right operand
        let right_val = if check.right.is_empty() {
            left_val.clone()
        } else {
            resolve_value(&check.right, message, locals)
        };

        // Parse and apply operator
        let op = CheckOperator::from_str(&check.op)?;
        let matched = op.evaluate(&left_val?, &right_val?)?;

        if !matched {
            return Ok(false); // Check failed
        }
    }

    Ok(true) // All checks passed
}

fn entries_to_map(entries: &[crate::display::Entry]) -> Result<BTreeMap<&str, &str>, ParseError> {
    let mut map = BTreeMap::new();
    for entry in entries {
        if map.contains_key(entry.key.as_str()) {
            return Err(SmthWentWrong(format!("Duplicate entry key: {}", entry.key)));
        }

        map.insert(entry.key.as_str(), entry.value.as_str());
    }
    Ok(map)
}

fn map_to_values<'a>(params: &BTreeMap<&str, &'a str>, prefix: char) -> Vec<(String, &'a str)> {
    params
        .iter()
        .filter(|(key, _)| key.starts_with(prefix))
        .map(|(key, value)| (key.strip_prefix(prefix).unwrap().to_string(), *value))
        .collect()
}

fn resolve_param_value(
    params: &BTreeMap<&str, &str>,
    key: &str,
    message: &Message,
    locals: &SolValue,
) -> Result<SolValue, ParseError> {
    let param_value = params
        .get(key)
        .ok_or_else(|| ParamNotFound(format!("Param {} not found", key)))?;

    resolve_value(param_value, message, locals)
}

fn resolve_optional_param_value(
    params: &BTreeMap<&str, &str>,
    key: &str,
    message: &Message,
    locals: &SolValue,
) -> Option<Result<SolValue, ParseError>> {
    let param_value = params.get(key);

    param_value.map(|param_value| resolve_value(param_value, message, locals))
}

fn decode_abi_locals(
    params: &BTreeMap<&str, &str>,
    message: &Message,
    locals: &SolValue,
    match_locals: &mut Vec<(Option<String>, SolValue)>,
) -> Result<(), ParseError> {
    if let (Some(abi), Some(value)) = (
        resolve_optional_param_value(params, "abi", message, locals),
        resolve_optional_param_value(params, "value", message, locals),
    ) {
        let abi_string = abi?.as_string()?;
        let bytes_value = value?.as_bytes()?;

        let sol_type = SolType::parse(&abi_string)?;

        let dyn_type = DynSolType::from(&sol_type);
        let decoded_dyn_value = dyn_type.abi_decode_params(&bytes_value)?;
        let decoded_locals = SolValue::from(decoded_dyn_value, &sol_type)?;

        if let SolValue::Tuple(t) = decoded_locals {
            match_locals.extend(t);
        }
    }
    Ok(())
}

fn decode_params_locals(
    params: &BTreeMap<&str, &str>,
    message: &Message,
    locals: &SolValue,
    match_locals: &mut Vec<(Option<String>, SolValue)>,
) -> Result<(), ParseError> {
    let new_locals_names = map_to_values(params, '$');
    if new_locals_names.is_empty() {
        return Ok(());
    }

    for (name, reference) in new_locals_names {
        let value = resolve_value(reference, message, locals)?;
        match_locals.push((Some(name), value));
    }
    Ok(())
}

#[cfg(all(test, feature = "serde"))]
#[cfg(feature = "serde_json")]
mod tests {
    use super::*;
    use crate::display::Display;
    use alloc::collections::BTreeMap;
    use alloc::vec;
    use alloy_primitives::{address, uint, Address, I256};

    pub struct LocalRegistry {
        pub well_known_displays: BTreeMap<FixedBytes<4>, Display>,
        pub well_known_contracts: Vec<Address>,
        pub well_known_tokens: Vec<Address>,
    }

    impl Registry for LocalRegistry {
        fn is_well_known_contract(&self, address: &Address) -> bool {
            self.well_known_contracts.contains(address)
                || self.well_known_tokens.contains(address)
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

    fn get_display(title: &str) -> Display {
        let json = include_str!("displays_test.json");
        let displays: crate::display::DisplaySpecFile =
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

        let context = ClearCallContext {
            displays: vec![display, f.inner_display],
        };

        let result = context
            .parse_clear_call(message, &registry, 0)
            .expect("Failed to parse clear call");

        assert_eq!(result.fields.len(), 14);

        // 0: Addr (Literal title)
        match &result.fields[0] {
            DisplayField::Address { title, value, .. } => {
                assert_eq!(title, "Addr Title");
                assert_eq!(*value, f.param_addr);
            }
            _ => panic!(),
        }
        // 1: Uint (Literal title)
        match &result.fields[1] {
            DisplayField::Uint { title, value, .. } => {
                assert_eq!(title, "Uint Title");
                assert_eq!(*value, uint!(123_U256));
            }
            _ => panic!(),
        }
        // 2: Int
        match &result.fields[2] {
            DisplayField::Int { value, .. } => assert_eq!(*value, I256::from_raw(uint!(456_U256))),
            _ => panic!(),
        }
        // 3: Bool
        match &result.fields[3] {
            DisplayField::Boolean { value, .. } => assert!(*value),
            _ => panic!(),
        }
        // 4: String
        match &result.fields[4] {
            DisplayField::String { value, .. } => assert_eq!(value, "Hello"),
            _ => panic!(),
        }
        // 5: Bytes
        match &result.fields[5] {
            DisplayField::Bytes { value, .. } => assert_eq!(value.as_ref(), &[0xca, 0xfe]),
            _ => panic!(),
        }
        // 6: Percentage
        match &result.fields[6] {
            DisplayField::Percentage { value, basis, .. } => {
                assert_eq!(*value, uint!(75_U256));
                assert_eq!(*basis, uint!(100_U256));
            }
            _ => panic!(),
        }
        // 7: Duration
        match &result.fields[7] {
            DisplayField::Duration { value, .. } => assert_eq!(value.as_secs(), 3600),
            _ => panic!(),
        }
        // 8: Datetime
        match &result.fields[8] {
            DisplayField::Datetime { value, .. } => assert_eq!(value.as_secs(), 3600),
            _ => panic!(),
        }
        // 9: Bitmask (Binary 1001 -> Bit 0 and Bit 3)
        match &result.fields[9] {
            DisplayField::Bitmask { values, .. } => {
                assert_eq!(values.len(), 2);
                assert_eq!(values[0], "Bit0");
                assert_eq!(values[1], "Bit3 Label");
            }
            _ => panic!(),
        }
        // 10: Token
        match &result.fields[10] {
            DisplayField::Token { token, .. } => assert_eq!(*token, f.main_token),
            _ => panic!(),
        }
        // 11: Contract
        match &result.fields[11] {
            DisplayField::Contract { contract, .. } => assert_eq!(*contract, f.param_addr),
            _ => panic!(),
        }
        // 12: TokenAmt
        match &result.fields[12] {
            DisplayField::TokenAmount { token, amount, .. } => {
                assert_eq!(*token, f.main_token);
                assert_eq!(*amount, uint!(123_U256));
            }
            _ => panic!(),
        }
        // 13: NativeAmt
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

        // Setup Registry
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

        let context = ClearCallContext {
            displays: vec![f.inner_display.clone()],
        };

        let result = context
            .parse_clear_call(message, &registry, 0)
            .expect("Failed to parse");

        assert_eq!(result.fields.len(), 2);

        // 0: Call (Wrapped)
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
        // 1: ERC20 (Unwrapped from Registry)
        match &result.fields[1] {
            DisplayField::Call { call, .. } => {
                assert_eq!(call.title, "ERC20");
                assert_eq!(call.fields.len(), 3);
                // Sender: $msg.sender (which is message.sender = display_addr in this context)
                match &call.fields[0] {
                    DisplayField::Address { value, .. } => assert_eq!(*value, display_addr),
                    _ => panic!(),
                }
                // Receiver: $locals.to
                match &call.fields[1] {
                    DisplayField::Address { value, .. } => assert_eq!(*value, f.receiver_addr),
                    _ => panic!(),
                }
                // Amount: tokenAmount
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

        // Setup Registry
        let registry = setup_test_registry(vec![display_addr], vec![], vec![(display)]);

        let call_data = matchCallCall { val: uint!(1_U256) }.abi_encode();
        let message = Message {
            sender: Address::ZERO,
            to: display_addr,
            value: U256::ZERO,
            data: call_data.into(),
        };

        let context = ClearCallContext { displays: vec![] };

        let result = context
            .parse_clear_call(message, &registry, 0)
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

        let context = ClearCallContext { displays: vec![] };

        let result = context
            .parse_clear_call(message, &registry, 0)
            .expect("Failed to parse");

        assert_eq!(result.fields.len(), 1, "Expected 1 top-level field (Array)");

        match &result.fields[0] {
            DisplayField::Array { values, .. } => {
                assert_eq!(values.len(), 2, "Expected 2 items in array");
                // Item 0
                assert_eq!(values[0].len(), 1, "Expected 1 field in first item");
                match &values[0][0] {
                    DisplayField::Uint { value, .. } => assert_eq!(*value, uint!(10_U256)),
                    _ => panic!("Expected Uint 10"),
                }
                // Item 1
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

        // (address, uint256)
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

        let context = ClearCallContext { displays: vec![] };

        let result = context
            .parse_clear_call(message, &registry, 0)
            .expect("Failed to parse");

        assert_eq!(result.fields.len(), 1, "Expected 1 field in result");

        // Field 0: Match (from Format::Match with abi)
        match &result.fields[0] {
            DisplayField::Match { values, .. } => {
                assert_eq!(values.len(), 2, "Expected 2 inner fields in Abi Match");

                // Inner Field 0: Recipient
                match &values[0] {
                    DisplayField::Address { value, .. } => {
                        assert_eq!(*value, recipient);
                    }
                    _ => panic!("Expected Address field as first inner field"),
                }

                // Inner Field 1: Amount
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

        // (address, uint256)
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

        let context = ClearCallContext { displays: vec![] };

        let result = context
            .parse_clear_call(message, &registry, 0)
            .expect("Failed to parse");

        assert_eq!(result.fields.len(), 1, "Expected 1 field in result");

        // Field 0: Match (from Format::Match with abi)
        match &result.fields[0] {
            DisplayField::Match { values, .. } => {
                assert_eq!(values.len(), 3, "Expected 3 inner fields in Abi Match");

                // Inner Field 0: Sender Copy (from parent locals via $msg.sender)
                match &values[0] {
                    DisplayField::Address { value, .. } => {
                        assert_eq!(*value, sender);
                    }
                    _ => panic!("Expected Address field as first inner field"),
                }

                // Inner Field 1: Decoded Recipient (from ABI decode)
                match &values[1] {
                    DisplayField::Address { value, .. } => {
                        assert_eq!(*value, recipient);
                    }
                    _ => panic!("Expected Address field as second inner field"),
                }

                // Inner Field 2: Decoded Amount (from ABI decode)
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

        let context = ClearCallContext { displays: vec![] };

        let result = context.parse_clear_call(message, &registry, 0).unwrap();

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

                            // Field 0: Sender
                            match &call.fields[0] {
                                DisplayField::Address { value, .. } => {
                                    assert_eq!(*value, multicall_address);
                                }
                                _ => panic!("Expected Address field for Sender"),
                            }

                            // Field 1: Receiver
                            match &call.fields[1] {
                                DisplayField::Address { value, .. } => {
                                    assert_eq!(*value, token);
                                }
                                _ => panic!("Expected Address field for Receiver"),
                            }

                            // Field 2: Amount
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
