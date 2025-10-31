use foxglove::Encode;

#[derive(Encode)]
pub struct Float {
    pub value: f32
}

#[derive(Encode)]
pub struct Integer {
    pub value: i32
}

#[derive(Encode)]
pub struct Bool {
    pub value: bool
}
