use near_sdk::env;
use near_sys as sys;

const ECRECOVER_MESSAGE_SIZE: u64 = 32;
const ECRECOVER_SIGNATURE_LENGTH: u64 = 64;
const ECRECOVER_MALLEABILITY_FLAG: u64 = 0;

pub fn validate_evm_address(
    address: &str,
    content: &Option<String>,
    signature: &str,
) -> Result<(), String> {
    let address = if address.starts_with("0x") {
        &address[2..]
    } else {
        address
    };
    let address = hex::decode(address).map_err(|_| "Invalid address")?;
    let signature = hex::decode(signature).map_err(|_| "Invalid signature")?;
    let data = match content {
        Some(c) => c.as_bytes(),
        None => "".as_bytes(),
    };
    let recovered_address = ecrecover(env::keccak256(data), &signature)
        .map_err(|_| "Failed to recover address from signature")?;
    if recovered_address != address {
        return Err("Incorrect signature".to_string());
    }
    Ok(())
}

fn ecrecover(hash: Vec<u8>, signature: &[u8]) -> Result<Vec<u8>, ()> {
    unsafe {
        let hash_ptr = hash.as_ptr() as u64;
        let sig_ptr = signature.as_ptr() as u64;
        const RECOVER_REGISTER_ID: u64 = 1;
        const KECCACK_REGISTER_ID: u64 = 2;
        let result = sys::ecrecover(
            ECRECOVER_MESSAGE_SIZE,
            hash_ptr,
            ECRECOVER_SIGNATURE_LENGTH,
            sig_ptr,
            signature[64] as u64,
            ECRECOVER_MALLEABILITY_FLAG,
            RECOVER_REGISTER_ID,
        );
        if result == (true as u64) {
            // The result from the ecrecover call is in a register; we can use this
            // register directly for the input to keccak256. This is why the length is
            // set to `u64::MAX`.
            sys::keccak256(u64::MAX, RECOVER_REGISTER_ID, KECCACK_REGISTER_ID);
            let keccak_hash_bytes = [0u8; 32];
            sys::read_register(KECCACK_REGISTER_ID, keccak_hash_bytes.as_ptr() as u64);
            let mut result = Vec::with_capacity(20);
            result.extend_from_slice(&keccak_hash_bytes[12..]);
            Ok(result)
        } else {
            Err(())
        }
    }
}
