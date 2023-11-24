use clap::Parser;
use axiom_eth::{keccak::KeccakChip, EthChip, Field};
use halo2_base::{AssignedValue, Context, gates::GateChip};
use halo2_base::utils::{BigPrimeField, fe_to_biguint};
use halo2_scaffold::circuits::wiener::WienerProcessChip;
use zk_fixed_point_chip::gadget::fixed_point::{FixedPointChip, FixedPointInstructions};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::run;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub x: String, // field element, but easier to deserialize as a string
}

fn compute_expected_shortfall<F: Field>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>
) where F: BigPrimeField {

    // fixed-point exp arithmetic
    let fixed_point_chip = FixedPointChip::<F, 63>::default(63);

    // weiner process
    

    let seed: F = F::from_str_vartime(&input.x).expect("deserialize field element should not fail");
    let seed = ctx.load_witness(seed);
    let wiener_chip = WienerProcessChip::<F>::new(fixed_point_chip, seed, 63);
    // let hash = wiener_chip.gen_wiener(ctx, 1);

    // let hash_value = fe_to_biguint(hash.value());
    // println!("hash: {:?}", hash_value);

    let hash2 = wiener_chip.mock_gen_wiener(ctx, 10);
    for h in hash2 {
        let hash_value = wiener_chip.fp_chip.dequantization(*h.value());
        println!("hash2: {:?}", hash_value);
    }
} 

fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(compute_expected_shortfall, args);
}