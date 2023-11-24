use axiom_eth::{batch_query::response::native::FullStorageQuery, providers::get_full_storage_queries};
use ethers_core::{
    types::{Address, H256},
    utils::{keccak256, hex},
};
use ethers_providers::{Http, Provider, RetryClient};

pub const ACCOUNT_PROOF_MAX_DEPTH: usize = 10;
pub const STORAGE_PROOF_MAX_DEPTH: usize = 9;

fn setup_provider() -> Provider<RetryClient<Http>> {
    let provider_url =
        format!("https://eth-mainnet.g.alchemy.com/v2/__wuL0POxev-7Byt4495_GrL3_y4JyTH");
    Provider::new_client(&provider_url, 10, 500).expect("could not instantiate HTTP Provider")
}

fn increment_h256(mut value: H256) -> H256 {
    let mut overflow = true;
    let bytes = value.as_mut();

    for byte in bytes.iter_mut().rev() {
        if overflow {
            if *byte == 0xff_u8 {
                *byte = 0;
            } else {
                *byte += 1;
                overflow = false;
            }
        } else {
            break;
        }
    }

    // If overflow is still true here, it means we've had an overflow
    // across all bytes (e.g., incrementing the maximum value).
    value
}

pub fn get_storage_queries_single_block(
    block_number: u64,
    ilk: H256,
    urn: Address,
) -> Vec<FullStorageQuery> {
    let address = "0x35D1b3F3D7966A1DFe207aa4514C12a259A0492B"; // VAT
    let address: Address = address.parse().unwrap();
    let mut queries = vec![];

    // slot = keccak256(address | keccak256(ilk | 3))
    let mut bytes = [0u8; 64];
    bytes[..32].copy_from_slice(ilk.as_ref()); // ilk
    bytes[63] = 3; // slot 3 is urns base slot
    let inner_slot = H256::from_slice(&keccak256(bytes));

    let mut final_bytes = [0u8; 64];
    final_bytes[12..32].copy_from_slice(urn.as_ref());
    final_bytes[32..64].copy_from_slice(inner_slot.as_ref());

    let ink_slot = H256::from_slice(&keccak256(final_bytes));
    let art_slot = increment_h256(ink_slot);

    queries.push(FullStorageQuery { block_number, addr_slots: Some((address, vec![ink_slot])) });
    queries.push(FullStorageQuery { block_number, addr_slots: Some((address, vec![art_slot])) });

    queries
}

#[test]
fn test_query() {
    let ilk = H256::from_slice(
        &hex::decode("4554482d41000000000000000000000000000000000000000000000000000000")
            .expect("Invalid hex string"),
    );
    let urn = "0x655e761c941cdbae43514f65e57865f9bc3f54ab"; // VAT
    let urn: Address = urn.parse().unwrap();

    let latest_block = 18509825;
    let mut queries = vec![];

    for i in 0..24 {
        let mut per_block_query =
            get_storage_queries_single_block(latest_block - 300 * i, ilk, urn);
        queries.append(&mut per_block_query);
    }
    println!("{:?}", queries);

    let mainnet_provider = setup_provider();

    let input = get_full_storage_queries(
        &mainnet_provider,
        queries,
        ACCOUNT_PROOF_MAX_DEPTH,
        STORAGE_PROOF_MAX_DEPTH,
    )
    .unwrap();
}
