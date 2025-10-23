/// This file defines custom message schemas to be used by channels
use foxglove::Encode;

#[derive(Encode)]
pub struct Float {
    pub value: f32
}

#[derive(Encode)]
pub struct Integer {
    pub value: u32
}

#[derive(Encode)]
pub struct Bool {
    pub value: bool
}
