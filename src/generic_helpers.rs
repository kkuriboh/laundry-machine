pub enum Assert<const CHECK: bool> {}

pub trait IsTrue {}
pub trait IsFalse {}

impl IsTrue for Assert<true> {}
impl IsFalse for Assert<false> {}
