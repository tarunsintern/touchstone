use clap::Parser;
use axiom_eth::{keccak::KeccakChip, EthChip, Field};
use halo2_base::{AssignedValue, Context, gates::GateChip};
use halo2_base::utils::{ScalarField, BigPrimeField, fe_to_biguint};
use halo2_scaffold::circuits::weiner::WeinerProcessChip;
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
    let fixed_point_chip = FixedPointChip::<F, 32>::default(32);

    // weiner process
    

    let seed: F = F::from_str_vartime(&input.x).expect("deserialize field element should not fail");
    let seed = ctx.load_witness(seed);
    let weiner_chip = WeinerProcessChip::<F>::new(fixed_point_chip, seed, 32);
    let hash = weiner_chip.gen_weiner(ctx, 1);

    let hash_value = fe_to_biguint(hash.value());
    println!("hash: {:?}", hash_value);

    let flt = 124.12;
    let quant = weiner_chip.fp_chip.quantization(flt);

    let deq = weiner_chip.fp_chip.dequantization(quant);
    println!("deq: {:?}", deq);
} 

fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(compute_expected_shortfall, args);
}