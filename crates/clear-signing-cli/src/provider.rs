use alloy_core::primitives::{Address, FixedBytes};
use clear_signing::{display::Display, registry::Registry, sol::SolFunction};
use clear_signing_format::{Contract, MetadataProvider, NativeToken, Token};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Contact {
    pub address: Address,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ClearSigningProvider {
    pub tokens: Vec<Token>,
    pub contracts: Vec<Contract>,
    pub contacts: Vec<Contact>,
    pub displays: Vec<Display>,
    pub native_token: NativeToken,
}

pub struct ProviderRegistry<'a> {
    pub provider: &'a ClearSigningProvider,
}

impl<'a> Registry for ProviderRegistry<'a> {
    fn is_well_known_contract(&self, address: &Address) -> bool {
        self.provider
            .contracts
            .iter()
            .any(|c| c.address == *address)
    }

    fn is_well_known_token(&self, address: &Address) -> bool {
        self.provider.tokens.iter().any(|t| t.address == *address)
    }

    fn get_well_known_display(
        &self,
        address: &Address,
        selector: &FixedBytes<4>,
    ) -> Option<Display> {
        self.provider
            .displays
            .iter()
            .find(|d| {
                if d.address != *address && d.address != Address::ZERO {
                    return false;
                }
                if let Ok(func) = SolFunction::parse(&d.abi) {
                    func.selector() == *selector
                } else {
                    false
                }
            })
            .map(|d| d.clone())
    }
}

impl MetadataProvider for ClearSigningProvider {
    fn get_token(&self, address: Address) -> Option<Token> {
        self.tokens.iter().find(|t| t.address == address).cloned()
    }

    fn get_contract(&self, address: Address) -> Option<Contract> {
        self.contracts
            .iter()
            .find(|c| c.address == address)
            .cloned()
    }

    fn get_native_token(&self) -> NativeToken {
        self.native_token.clone()
    }

    fn get_address_name(&self, address: Address) -> Option<String> {
        if let Some(token) = self.get_token(address) {
            return Some(token.name);
        }
        if let Some(contract) = self.get_contract(address) {
            return Some(contract.name);
        }
        if let Some(contact) = self.contacts.iter().find(|c| c.address == address) {
            return Some(contact.name.clone());
        }
        None
    }
}

impl ClearSigningProvider {
    // Helper to allow convenient access for MetadataProvider impl
    // Actually MetadataProvider is implemented on ClearSigningProvider itself, so we can access fields directly via self.
    // The previous implementation used `self.provider.tokens` which is wrong because `self` IS the provider.
    // Wait, let me fix that in the CodeContent before writing.
}
