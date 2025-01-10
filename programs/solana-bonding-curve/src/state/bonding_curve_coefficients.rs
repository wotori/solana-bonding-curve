use anchor_lang::prelude::*;

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub struct BondingCurveCoefficients {
    pub coefficient_a: u64,
    pub coefficient_b: u64,
    pub coefficient_c: u64,
    // TODO:
}
