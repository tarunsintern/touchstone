use halo2_base::{
    utils::{BigPrimeField, fe_to_biguint},
    QuantumCell::{self, Constant}, Context, AssignedValue,
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
pub struct WienerProcessChip<F: BigPrimeField> {
    pub fp_chip: FixedPointChip<F, 63>,
    seed: AssignedValue<F>,
    pub lookup_bits: usize,
}


impl<F: BigPrimeField> WienerProcessChip<F> {
    pub fn new(
        fp_chip: FixedPointChip<F, 63>,
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

    pub fn gen_wiener(
        &self,
        ctx: &mut Context<F>,
        n: usize
    ) -> AssignedValue<F> {
            let max_uint32 = F::from_str_vartime("495").unwrap();
            let max_uint32 = fe_to_biguint(ctx.load_witness(max_uint32).value());
            let mut poseidon_chip = PoseidonChip::<F, T, RATE>::new(ctx, R_F, R_P).unwrap();
            poseidon_chip.update(&[self.seed]);
            let hash = poseidon_chip.squeeze(ctx, self.fp_chip.gate()).unwrap();
            println!("n: {:?}, poseidon(x): {:?}", self.seed, hash.value());
            println!("max_uint32: {:?}", max_uint32);
            println!("anded: {:?}", self.fp_chip.range_gate().div_mod(ctx, hash, max_uint32, 254));
    
            hash
    } 

    pub fn mock_gen_wiener(
        &self,
        ctx: &mut Context<F>,
        n: usize
    ) -> Vec<AssignedValue<F>> {
        let samples = [0.99, 0.23, 0.9, 0.365, 0.5, 0.6, 0.7, 0.8,0.9, 0.1];
        let mut samples_f: Vec<AssignedValue<F>> = vec![];
        for i in 0..5 {
            // divide into two halves for marsaglia polar method
            let u1 = ctx.load_witness(self.fp_chip.quantization(samples[i]));
            let u2 = ctx.load_witness(self.fp_chip.quantization(samples[i+1]));

            let exp2 = ctx.load_constant(self.fp_chip.quantization(2.0));
            let u1_2 = self.fp_chip.qpow(ctx, u1, exp2 );
            let u2_2 = self.fp_chip.qpow(ctx, u2, exp2);

            // s = u1^2 + u2^2
            let s = self.fp_chip.qadd(ctx, u1_2, u2_2);
            println!("s: {:?}",  self.fp_chip.dequantization(*s.value()));

            // check if s >= 1
            let amp = self.fp_chip.qsub(ctx, s, Constant(F::from(1)));
            println!("amp: {:?}", self.fp_chip.dequantization(*amp.value()));
            let amp_sign =self.fp_chip.is_neg(ctx, amp);

            // multipler = √(-2 * log(s) / s)
            let log_s = self.fp_chip.qlog(ctx, s);
            let neg2 = ctx.load_constant(self.fp_chip.quantization(-2.0));
            let qnum = self.fp_chip.qmul(ctx, neg2, log_s);
            let term = self.fp_chip.qdiv(ctx, qnum, s);
            let multiplier = self.fp_chip.qsqrt(ctx, term);
            let multiplier = self.fp_chip.gate().select(ctx, Constant(F::from(0)), multiplier, amp_sign);

            // x1 = u1 * multiplier
            let x1 = self.fp_chip.qmul(ctx, u1, multiplier);
            samples_f.push(x1);
        }

        samples_f

    }
}
