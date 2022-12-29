use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault};

use crate::validation::validate_evm_address;

mod validation;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKeys {
    Mappings,
    Delegates,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum Id {
    AccountId(AccountId),
    EvmAddress(String),
}

impl ToString for Id {
    fn to_string(&self) -> String {
        match self {
            Id::AccountId(account_id) => format!("near|{}", account_id),
            Id::EvmAddress(address) => format!("evm|{}", address),
        }
    }
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
struct Contract {
    mappings: LookupMap<(String, String), String>,
    delegates: LookupMap<AccountId, AccountId>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            mappings: LookupMap::new(StorageKeys::Mappings),
            delegates: LookupMap::new(StorageKeys::Delegates),
        }
    }

    /// Returns account id that must be modified or crashes if there is no permissions to write to given account.
    fn resolve_id(
        &self,
        id: Option<Id>,
        content: &Option<String>,
        proof: Option<String>,
    ) -> String {
        match id {
            Some(id) => {
                // Validate ids.
                match &id {
                    Id::AccountId(account_id) => {
                        if &env::predecessor_account_id() != account_id {
                            assert_eq!(
                                self.delegates.get(&account_id).expect("ERR_NOT_DELEGATE"),
                                env::predecessor_account_id(),
                                "ERR_NOT_DELEGATE"
                            );
                        }
                    }
                    Id::EvmAddress(address) => {
                        let signature = proof.expect("ERR_MISSING_SIGNATURE");
                        validate_evm_address(&address, content, &signature)
                            .expect("ERR_EVM_INVALID_SIGNATURE");
                    }
                }
                id.to_string()
            }
            None => env::predecessor_account_id().to_string(),
        }
    }

    pub fn set(
        &mut self,
        id: Option<Id>,
        label: String,
        content: Option<String>,
        proof: Option<String>,
    ) {
        let id = self.resolve_id(id, &content, proof);
        if let Some(content) = content {
            self.mappings.insert(&(id, label), &content);
        } else {
            self.mappings.remove(&(id, label));
        }
    }

    pub fn get(&self, id: Id, label: String) -> Option<String> {
        self.mappings.get(&(id.to_string(), label))
    }

    pub fn delegate(&mut self, account_id: Option<AccountId>) {
        if let Some(account_id) = account_id {
            self.delegates
                .insert(&env::predecessor_account_id(), &account_id);
        } else {
            self.delegates.remove(&env::predecessor_account_id());
        }
    }
}

#[cfg(test)]
mod tests {
    use near_crypto::{InMemorySigner, KeyType, PublicKey, Signature, Signer};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    use super::*;

    fn get_context(account_id: &AccountId) -> VMContext {
        VMContextBuilder::new()
            .predecessor_account_id(account_id.clone())
            .build()
    }

    #[test]
    fn test_basics() {
        let account1 = AccountId::new_unchecked("acc1".to_string());
        let account2 = AccountId::new_unchecked("acc2".to_string());
        testing_env!(get_context(&account1));
        let mut c = Contract::new();
        c.set(None, "label".to_string(), Some("test".to_string()), None);
        assert_eq!(
            c.get(Id::AccountId(account1.clone()), "label".to_string()),
            Some("test".to_string())
        );
        c.set(
            Some(Id::AccountId(account1.clone())),
            "label".to_string(),
            Some("test2".to_string()),
            None,
        );
        assert_eq!(
            c.get(Id::AccountId(account1.clone()), "label".to_string()),
            Some("test2".to_string())
        );
        c.delegate(Some(account2.clone()));
        testing_env!(get_context(&account2));
        c.set(
            Some(Id::AccountId(account1.clone())),
            "label".to_string(),
            Some("test3".to_string()),
            None,
        );
        assert_eq!(
            c.get(Id::AccountId(account1.clone()), "label".to_string()),
            Some("test3".to_string())
        );
    }

    fn public_key_to_address(public_key: PublicKey) -> String {
        match public_key {
            PublicKey::ED25519(_) => panic!("Wrong PublicKey"),
            PublicKey::SECP256K1(pubkey) => {
                let pk: [u8; 64] = pubkey.into();
                let bytes = env::keccak256(&pk.to_vec());
                format!("0x{}", hex::encode(&bytes[12..]))
            }
        }
    }

    fn signature_to_hex(signature: Signature) -> String {
        match signature {
            Signature::ED25519(_) => unimplemented!(),
            Signature::SECP256K1(sig) => hex::encode(&Into::<[u8; 65]>::into(sig)),
        }
    }

    #[test]
    fn test_evm_validation() {
        let signer = InMemorySigner::from_seed("no_account", KeyType::SECP256K1, "not real seed");
        let address = public_key_to_address(signer.public_key().clone());
        let account1 = AccountId::new_unchecked("acc1".to_string());
        testing_env!(get_context(&account1));
        let mut c = Contract::new();
        let content = "[\"a.near\"]".to_string();
        let signature = signature_to_hex(signer.sign(&env::keccak256(content.as_bytes())));
        println!("{:?}", signature);
        c.set(
            Some(Id::EvmAddress(address.clone())),
            "near".to_string(),
            Some(content),
            Some(signature),
        );
        assert_eq!(
            c.get(Id::EvmAddress(address), "near".to_string()),
            Some("[\"a.near\"]".to_string())
        );
    }
}
