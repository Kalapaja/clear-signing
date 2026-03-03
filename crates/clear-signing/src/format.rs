use crate::clear_call::{parse_clear_call_with_level, process_nested_fields};
use crate::display::Display;
use crate::display::{Entry, Field};
use crate::fields::{Direction, DisplayField, Label};
use crate::registry::Registry;
use crate::resolver::{resolve_value, Message};
use crate::sol::{SolType, SolValue};
use crate::ResultExt;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use alloy_dyn_abi::DynSolType;
use alloy_primitives::address;
use core::time::Duration;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};


pub(crate) struct FieldParams {
    params: BTreeMap<String, String>,
}

impl FieldParams {
    pub fn new(entries: &[Entry]) -> crate::Result<Self> {
        let mut params = BTreeMap::new();
        for entry in entries {
            if params.contains_key(&entry.key) {
                anyhow::bail!("Duplicate entry key: {}", entry.key);
            }
            params.insert(entry.key.clone(), entry.value.clone());
        }
        Ok(Self { params })
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    pub fn get_with_prefix(&self, prefix: char) -> BTreeMap<String, String> {
        self.params
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .map(|(k, v)| (k.strip_prefix(prefix).unwrap().to_string(), v.clone()))
            .collect()
    }

    pub fn resolve(&self, key: &str, message: &Message, data: &SolValue) -> crate::Result<SolValue> {
        let value = self.get(key)
            .ok_or_else(|| anyhow::anyhow!("Param {} not found", key))?;
        resolve_value(value, message, data)
    }

    pub fn resolve_optional(&self, key: &str, message: &Message, data: &SolValue) -> crate::Result<Option<SolValue>> {
        match self.get(key) {
            Some(value) => resolve_value(value, message, data).map(Some),
            None => Ok(None),
        }
    }
}


pub(crate) struct ProcessingContext<'a> {
    pub(crate) field: &'a Field,
    pub(crate) params: FieldParams,
    pub(crate) message: &'a Message,
    pub(crate) data: &'a SolValue,
    pub(crate) registry: &'a dyn Registry,
    pub(crate) displays: &'a [Display],
    pub(crate) level: usize,
}

impl<'a> ProcessingContext<'a> {
    pub fn new(
        field: &'a Field,
        message: &'a Message,
        data: &'a SolValue,
        registry: &'a dyn Registry,
        displays: &'a [Display],
        level: usize,
    ) -> crate::Result<Self> {
        Ok(Self {
            field,
            params: FieldParams::new(&field.params)?,
            message,
            data,
            registry,
            displays,
            level,
        })
    }

    pub fn title(&self) -> &str {
        &self.field.title
    }

    pub fn description(&self) -> &str {
        &self.field.description
    }

    pub fn nested_fields(&self) -> &[Field] {
        &self.field.fields
    }

    pub fn resolve_param(&self, key: &str) -> crate::Result<SolValue> {
        self.params.resolve(key, self.message, self.data)
    }

    pub fn resolve_optional_param(&self, key: &str) -> crate::Result<Option<SolValue>> {
        self.params.resolve_optional(key, self.message, self.data)
    }

    pub fn labels(&self) -> (crate::fields::Label, crate::fields::Label) {
        (
            self.field.title.clone(),
            self.field.description.clone(),
        )
    }
}



#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub(crate) enum Format {
    TokenAmount,
    NativeAmount,
    Contract,
    Token,
    Address,
    Bytes,
    String,
    Call,
    Boolean,
    Int,
    Uint,
    Percentage,
    Duration,
    Datetime,
    Bitmask,
    Match,
    Array,
    Switch,
}

impl Format {
    pub fn from(format: &str) -> crate::Result<Self> {
        match format {
            "tokenAmount" => Ok(Format::TokenAmount),
            "nativeAmount" => Ok(Format::NativeAmount),
            "contract" => Ok(Format::Contract),
            "token" => Ok(Format::Token),
            "address" => Ok(Format::Address),
            "bytes" => Ok(Format::Bytes),
            "string" => Ok(Format::String),
            "call" => Ok(Format::Call),
            "boolean" => Ok(Format::Boolean),
            "int" => Ok(Format::Int),
            "uint" => Ok(Format::Uint),
            "percentage" => Ok(Format::Percentage),
            "duration" => Ok(Format::Duration),
            "datetime" => Ok(Format::Datetime),
            "bitmask" => Ok(Format::Bitmask),
            "match" => Ok(Format::Match),
            "array" => Ok(Format::Array),
            "switch" => Ok(Format::Switch),
            _ => anyhow::bail!("Unknown format: {}", format),
        }
    }

    pub fn process(&self, ctx: &ProcessingContext) -> crate::Result<DisplayField> {
        match self {
            Format::Address => process_address(ctx),
            Format::TokenAmount => process_token_amount(ctx),
            Format::NativeAmount => process_native_amount(ctx),
            Format::Contract => process_contract(ctx),
            Format::Token => process_token(ctx),
            Format::Bytes => process_bytes(ctx),
            Format::String => process_string(ctx),
            Format::Call => process_call(ctx),
            Format::Boolean => process_boolean(ctx),
            Format::Int => process_int(ctx),
            Format::Uint => process_uint(ctx),
            Format::Percentage => process_percentage(ctx),
            Format::Duration => process_duration(ctx),
            Format::Datetime => process_datetime(ctx),
            Format::Bitmask => process_bitmask(ctx),
            Format::Match => process_match(ctx),
            Format::Array => process_array(ctx),
            Format::Switch => process_switch(ctx),
        }
    }
}


pub(crate) fn process_address(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let value = ctx.resolve_param("value")?.as_address()?;
    let (title, description) = ctx.labels();

    Ok(DisplayField::Address {
        title,
        description,
        value,
    })
}

pub(crate) fn process_token_amount(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let amount = ctx.resolve_param("amount")?.as_uint()?;
    let token = ctx.resolve_param("token")?.as_address()?;
    let direction = ctx.resolve_optional_param("direction")?
        .map(Direction::try_from_sol_value)
        .transpose()?;

    let (title, description) = ctx.labels();

    let natives = vec![
        address!("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
        address!("0x0000000000000000000000000000000000000000"),
    ];

    if natives.contains(&token) {
        Ok(DisplayField::NativeAmount {
            title,
            description,
            amount,
            direction,
        })
    } else {
        anyhow::ensure!(ctx.registry.is_well_known_token(&token), "Unknown token: {:?}", token);

        Ok(DisplayField::TokenAmount {
            title,
            description,
            token,
            amount,
            direction,
        })
    }
}

pub(crate) fn process_native_amount(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let amount = ctx.resolve_param("amount")?.as_uint()?;
    let direction = ctx.resolve_optional_param("direction")?
        .map(Direction::try_from_sol_value)
        .transpose()?;

    let (title, description) = ctx.labels();

    Ok(DisplayField::NativeAmount {
        title,
        description,
        amount,
        direction,
    })
}

pub(crate) fn process_contract(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let contract = ctx.resolve_param("value")?.as_address()?;

    anyhow::ensure!(
        ctx.registry.is_well_known_contract(&contract),
        "Unknown contract: {}", contract
    );

    Ok(DisplayField::Contract {
        title: ctx.title().to_string(),
        description: ctx.description().to_string(),
        contract,
    })
}

pub(crate) fn process_token(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let token = ctx.resolve_param("value")?.as_address()?;

    anyhow::ensure!(
        ctx.registry.is_well_known_token(&token),
        "Unknown token: {}", token
    );

    Ok(DisplayField::Token {
        title: ctx.title().to_string(),
        description: ctx.description().to_string(),
        token,
    })
}

pub(crate) fn process_bytes(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let value = ctx.resolve_param("value")?.as_bytes()?;

    Ok(DisplayField::Bytes {
        title: ctx.title().to_string(),
        description: ctx.description().to_string(),
        value: value.into(),
    })
}

pub(crate) fn process_string(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let value = ctx.resolve_param("value")?.as_string()?;

    Ok(DisplayField::String {
        title: ctx.title().to_string(),
        description: ctx.description().to_string(),
        value,
    })
}

pub(crate) fn process_call(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let to = ctx.resolve_param("to")?.as_address()?;
    let value = ctx.resolve_param("value")?.as_uint()?;
    let data = ctx.resolve_param("data")?.as_bytes()?;

    let msg = Message::new(ctx.message.to, to, value, data.into());
    let call = parse_clear_call_with_level(ctx.displays, msg, ctx.registry, ctx.level + 1)?;

    Ok(DisplayField::Call {
        title: ctx.title().to_string(),
        description: ctx.description().to_string(),
        call,
    })
}

pub(crate) fn process_boolean(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let value = ctx.resolve_param("value")?.as_bool()?;

    Ok(DisplayField::Boolean {
        title: ctx.title().to_string(),
        description: ctx.description().to_string(),
        value,
    })
}

pub(crate) fn process_int(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let value = ctx.resolve_param("value")?.as_int()?;

    Ok(DisplayField::Int {
        title: ctx.title().to_string(),
        description: ctx.description().to_string(),
        value,
    })
}

pub(crate) fn process_uint(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let value = ctx.resolve_param("value")?.as_uint()?;

    Ok(DisplayField::Uint {
        title: ctx.title().to_string(),
        description: ctx.description().to_string(),
        value,
    })
}

pub(crate) fn process_percentage(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let value = ctx.resolve_param("value")?.as_uint()?;
    let basis = ctx.resolve_param("basis")?.as_uint()?;

    Ok(DisplayField::Percentage {
        title: ctx.title().to_string(),
        description: ctx.description().to_string(),
        value,
        basis,
    })
}

pub(crate) fn process_duration(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let value = ctx.resolve_param("value")?.as_uint()?;

    Ok(DisplayField::Duration {
        title: ctx.title().to_string(),
        description: ctx.description().to_string(),
        value: Duration::from_secs(value.try_into().err_ctx("Can't parse uint into u64")?),
    })
}

pub(crate) fn process_datetime(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let value = ctx.resolve_param("value")?.as_uint()?;

    Ok(DisplayField::Datetime {
        title: ctx.title().to_string(),
        description: ctx.description().to_string(),
        value: Duration::from_secs(value.try_into().err_ctx("Can't parse uint into u64")?),
    })
}

pub(crate) fn process_bitmask(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let value = ctx.resolve_param("value")?.as_uint()?;
    let bit_indexes = ctx.params.get_with_prefix('#');

    let mut values: Vec<Label> = vec![];

    for (bit_str, label) in bit_indexes {
        let bit_index: usize = bit_str.parse()?;
        if value.bit(bit_index) {
            values.push(label);
        }
    }

    Ok(DisplayField::Bitmask {
        title: ctx.title().to_string(),
        description: ctx.description().to_string(),
        values,
    })
}

pub(crate) fn process_match(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let mut match_data = vec![];
    decode_abi_data(&ctx.params, ctx.message, ctx.data, &mut match_data)?;
    decode_params_data(&ctx.params, ctx.message, ctx.data, &mut match_data)?;

    let new_data = SolValue::Tuple(match_data);

    let new_fields = process_nested_fields(
        ctx.displays,
        ctx.message,
        ctx.nested_fields(),
        &new_data,
        ctx.registry,
        ctx.level + 1,
        None,
    )?;

    Ok(DisplayField::Match {
        title: ctx.title().to_string(),
        description: ctx.description().to_string(),
        values: new_fields,
    })
}

pub(crate) fn process_array(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let new_data_names = ctx.params.get_with_prefix('$');

    let mut arrays: Vec<(String, Vec<SolValue>)> = vec![];
    let mut length = 0;

    for (i, (name, reference)) in new_data_names.into_iter().enumerate() {
        let value = resolve_value(&reference, ctx.message, ctx.data)?.as_array()?;
        if i == 0 {
            length = value.len();
        }
        anyhow::ensure!(value.len() == length, "Array length mismatch");
        arrays.push((name, value));
    }

    let mut values = vec![];

    for i in 0..length {
        let mut tuple: Vec<(Option<String>, SolValue)> = vec![];
        for (name, array) in &arrays {
            tuple.push((Some(name.clone()), array[i].clone()));
        }
        let new_data = SolValue::Tuple(tuple);
        let item_fields = process_nested_fields(
            ctx.displays,
            ctx.message,
            ctx.nested_fields(),
            &new_data,
            ctx.registry,
            ctx.level + 1,
            None,
        )?;
        values.push(item_fields);
    }

    Ok(DisplayField::Array {
        title: ctx.title().to_string(),
        description: ctx.description().to_string(),
        values,
    })
}

pub(crate) fn process_switch(ctx: &ProcessingContext) -> crate::Result<DisplayField> {
    let switch_val = ctx.resolve_param("value")?;

    let mut switch_data = vec![];
    decode_abi_data(&ctx.params, ctx.message, ctx.data, &mut switch_data)?;
    decode_params_data(&ctx.params, ctx.message, ctx.data, &mut switch_data)?;

    let new_data = if !switch_data.is_empty() {
        SolValue::Tuple(switch_data)
    } else {
        ctx.data.clone()
    };

    let new_fields = process_nested_fields(
        ctx.displays,
        ctx.message,
        ctx.nested_fields(),
        &new_data,
        ctx.registry,
        ctx.level + 1,
        Some(switch_val.clone()),
    )?;

    Ok(DisplayField::Switch {
        title: ctx.title().to_string(),
        description: ctx.description().to_string(),
        fields: new_fields,
    })
}


pub(crate) fn decode_abi_data(
    params: &FieldParams,
    message: &Message,
    data: &SolValue,
    match_data: &mut Vec<(Option<String>, SolValue)>,
) -> crate::Result<()> {
    let abi = params.resolve_optional("abi", message, data)?;
    let value = params.resolve_optional("value", message, data)?;

    if let (Some(abi), Some(value)) = (abi, value) {
        let abi_string = abi.as_string()?;
        let bytes_value = value.as_bytes()?;

        let sol_type = SolType::parse(&abi_string)?;

        let dyn_type = DynSolType::from(&sol_type);
        let decoded_dyn_value = dyn_type.abi_decode_params(&bytes_value)?;
        let decoded_data = SolValue::from(decoded_dyn_value, &sol_type)?;

        if let SolValue::Tuple(t) = decoded_data {
            match_data.extend(t);
        }
    }
    Ok(())
}

pub(crate) fn decode_params_data(
    params: &FieldParams,
    message: &Message,
    data: &SolValue,
    match_data: &mut Vec<(Option<String>, SolValue)>,
) -> crate::Result<()> {
    let new_data_names = params.get_with_prefix('$');
    if new_data_names.is_empty() {
        return Ok(());
    }

    for (name, reference) in new_data_names {
        let value = resolve_value(&reference, message, data)?;
        match_data.push((Some(name), value));
    }
    Ok(())
}
