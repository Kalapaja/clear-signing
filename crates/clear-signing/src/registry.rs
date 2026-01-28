use crate::display::Display;
use crate::error::ParseError;
use crate::sol::SolFunction;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloy_core::primitives::{Address, FixedBytes};

pub trait Registry {
    fn is_well_known_contract(&self, address: &Address) -> bool;
    fn is_well_known_token(&self, address: &Address) -> bool;
    fn get_well_known_display(
        &self,
        address: &Address,
        selector: &FixedBytes<4>,
    ) -> Option<Display>;
}

pub struct LocalRegistry {
    pub well_known_displays: BTreeMap<(Address, FixedBytes<4>), Display>,
    pub well_known_contracts: Vec<Address>,
    pub well_known_tokens: Vec<Address>,
}

impl LocalRegistry {
    pub fn new(
        well_known_contracts_json: &str,
        well_known_tokens_json: &str,
        displays_json: &str,
    ) -> Result<Self, ParseError> {
        let displays: Vec<Display> = serde_json::from_str(displays_json)?;

        let mut display_address_selector = BTreeMap::new();
        for display in displays {
            let addr = display.address;
            let selector = SolFunction::parse(&display.abi)?.selector();
            display_address_selector.insert((addr, selector), display);
        }

        Ok(Self {
            well_known_displays: display_address_selector,
            well_known_contracts: serde_json::from_str(well_known_contracts_json)?,
            well_known_tokens: serde_json::from_str(well_known_tokens_json)?,
        })
    }
}

impl Registry for LocalRegistry {
    fn is_well_known_contract(&self, address: &Address) -> bool {
        self.well_known_contracts.contains(address)
    }

    fn is_well_known_token(&self, address: &Address) -> bool {
        self.well_known_tokens.contains(address)
    }

    fn get_well_known_display(
        &self,
        address: &Address,
        selector: &FixedBytes<4>,
    ) -> Option<Display> {
        self.well_known_displays
            .get(&(*address, *selector))
            .or_else(|| self.well_known_displays.get(&(Address::ZERO, *selector)))
            .map(|d| d.clone())
    }
}
