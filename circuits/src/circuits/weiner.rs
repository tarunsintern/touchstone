use halo2_base::{
    utils::{BigPrimeField, fe_to_biguint},
    QuantumCell, Context, AssignedValue,
};
use zk_fixed_point_chip::gadget::fixed_point::{FixedPointChip, FixedPointInstructions};
use halo2_base::gates::{GateChip, RangeChip, GateInstructions, RangeInstructions};
use poseidon::PoseidonChip;

const T: usize = 3;
const RATE: usize = 2;
const R_F: usize = 8;
const R_P: usize = 57;

// TODO: make WeinerChip
// dequant
// marsaglia_polar for one point
// multiple points
// benchmarking
// slicing into 4 columns
// tests

#[derive(Clone, Debug)]
pub struct WeinerProcessChip<F: BigPrimeField> {
    pub fp_chip: FixedPointChip<F, 32>,
    seed: AssignedValue<F>,
    pub lookup_bits: usize,
}


impl<F: BigPrimeField> WeinerProcessChip<F> {
    pub fn new(
        fp_chip: FixedPointChip<F, 32>,
        seed: AssignedValue<F>,
        lookup_bits: usize,
    ) -> Self {
        let fp_chip = fp_chip;
        Self {
            fp_chip,
            seed,
            lookup_bits,
        }
    }

    pub fn gen_weiner(
        &self,
        ctx: &mut Context<F>,
        n: usize
    ) -> AssignedValue<F> {
            let max_uint32 = F::from_str_vartime("495").unwrap();
            let max_uint32_fe = fe_to_biguint(ctx.load_witness(max_uint32).value());
            let mut poseidon_chip = PoseidonChip::<F, T, RATE>::new(ctx, R_F, R_P).unwrap();
            poseidon_chip.update(&[self.seed]);
            let hash = poseidon_chip.squeeze(ctx, self.fp_chip.gate()).unwrap();
            println!("n: {:?}, poseidon(x): {:?}", self.seed, hash.value());
            let hash2 = self.fp_chip.qabs(ctx, hash);
            let m = self.fp_chip.max_value.clone();
            println!("anded: {:?}", self.fp_chip.range_gate().div_mod(ctx, hash2, max_uint32_fe, 254));
    

            hash
    } 
}
