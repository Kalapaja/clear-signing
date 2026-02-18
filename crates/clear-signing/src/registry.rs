use crate::display::Display;
use alloy_primitives::{Address, FixedBytes};

pub trait Registry {
    fn is_well_known_contract(&self, address: &Address) -> bool;
    fn is_well_known_token(&self, address: &Address) -> bool;
    fn get_well_known_display(
        &self,
        address: &Address,
        selector: &FixedBytes<4>,
    ) -> Option<Display>;
}

