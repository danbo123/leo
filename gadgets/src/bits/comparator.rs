use snarkos_errors::gadgets::SynthesisError;
use snarkos_models::{
    curves::{Field, PrimeField},
    gadgets::{
        r1cs::ConstraintSystem,
        utilities::{
            boolean::Boolean,
            select::CondSelectGadget,
            uint::{UInt128, UInt16, UInt32, UInt64, UInt8},
        },
    },
};

pub trait EvaluateLtGadget<F: Field> {
    fn less_than<CS: ConstraintSystem<F>>(&self, cs: CS, other: &Self) -> Result<Boolean, SynthesisError>;
}

// implementing `EvaluateLtGadget` will implement `ComparatorGadget`
pub trait ComparatorGadget<F: Field>
where
    Self: EvaluateLtGadget<F>,
{
    fn greater_than<CS: ConstraintSystem<F>>(&self, cs: CS, other: &Self) -> Result<Boolean, SynthesisError> {
        other.less_than(cs, self)
    }

    fn less_than_or_equal<CS: ConstraintSystem<F>>(&self, cs: CS, other: &Self) -> Result<Boolean, SynthesisError> {
        let is_gt = self.greater_than(cs, other)?;
        Ok(is_gt.not())
    }

    fn greater_than_or_equal<CS: ConstraintSystem<F>>(&self, cs: CS, other: &Self) -> Result<Boolean, SynthesisError> {
        other.less_than_or_equal(cs, self)
    }
}

macro_rules! uint_cmp_impl {
    ($($gadget: ident),*) => ($(
        /*  Bitwise less than comparison of two unsigned integers */
        impl<F: Field + PrimeField> EvaluateLtGadget<F> for $gadget {
            fn less_than<CS: ConstraintSystem<F>>(&self, mut cs: CS, other: &Self) -> Result<Boolean, SynthesisError> {

                let mut result = Boolean::constant(true);
                let mut all_equal = Boolean::constant(true);

                // msb -> lsb
                for (i, (a, b)) in self
                    .bits
                    .iter()
                    .rev()
                    .zip(other.bits.iter().rev())
                    .enumerate()
                {
                    // a == 0 & b == 1
                    let less = Boolean::and(cs.ns(|| format!("not a and b [{}]", i)), &a.not(), b)?;

                    // a == b = !(a ^ b)
                    let not_equal = Boolean::xor(cs.ns(|| format!("a XOR b [{}]", i)), a, b)?;
                    let equal = not_equal.not();

                    // evaluate a <= b
                    let less_or_equal = Boolean::or(cs.ns(|| format!("less or equal [{}]", i)), &less, &equal)?;

                    // select the current result if it is the first bit difference
                    result = Boolean::conditionally_select(cs.ns(|| format!("select bit [{}]", i)), &all_equal, &less_or_equal, &result)?;

                    // keep track of equal bits
                    all_equal = Boolean::and(cs.ns(|| format!("accumulate equal [{}]", i)), &all_equal, &equal)?;
                }

                result = Boolean::and(cs.ns(|| format!("false if all equal")), &result, &all_equal.not())?;

                Ok(result)
            }
        }

        /* Bitwise comparison of two unsigned integers */
        impl<F: Field + PrimeField> ComparatorGadget<F> for $gadget {}
    )*)
}

uint_cmp_impl!(UInt8, UInt16, UInt32, UInt64, UInt128);