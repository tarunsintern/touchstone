use std::str::FromStr;

use axiom_eth::{
    keccak::KeccakChip,
    providers::get_block_storage_input,
    Field, storage::EthStorageChip,
};
use ethers_core::types::{H160, H256};
use ethers_providers::{Http, Provider};
use halo2_base::{AssignedValue, Context};

pub struct CircuitInput {
    pub provider_url: String,
    pub state_roots: Vec<String>,
    pub block_numbers: Vec<u32>,
    pub contract_address: String,
    pub slot: u32,
    pub account_proof_max_depth: usize,
    pub storage_proof_max_depth: usize,
}

fn compute_storage_proofs<F: Field>(
    ctx: &mut Context<F>,
    eth_chip: &dyn EthStorageChip<F>,
    keccak: &mut KeccakChip<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) {
    let provider = Provider::<Http>::try_from(input.provider_url.as_str()).unwrap();
    let slot = H256::from_low_u64_be(input.slot as u64);
    let contract_addr = H160::from_str(&input.contract_address).unwrap();

    // let mut callbacks = Vec::new();

    for block_number in &input.block_numbers {
        let proof = get_block_storage_input(
            &provider,
            *block_number,
            contract_addr,
            vec![slot],
            input.account_proof_max_depth,
            input.storage_proof_max_depth,
        ).storage.storage_pfs[0].clone().2.assign(ctx);

        let slot_bytes = ctx.assign_witnesses(slot.to_fixed_bytes().map(|b| F::from(b as u64)));
        for byte in &slot_bytes {
            make_public.push(*byte);
        }

        let storage_trace_witness = eth_chip.parse_storage_proof_phase0(ctx, keccak, slot_bytes, proof.clone());
        #[allow(clippy::let_and_return)]
        let callback =
        |ctx_gate: &mut Context<F>, ctx_rlc: &mut Context<F>, eth_chip: &dyn EthStorageChip<F>| {
            eth_chip.parse_storage_proof_phase1((ctx_gate, ctx_rlc), storage_trace_witness);
        };
    }
}

fn main() {
    env_logger::init();

    // run(get_storage_slot, args);
}
