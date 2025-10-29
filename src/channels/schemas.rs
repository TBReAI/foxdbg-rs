use foxglove::Encode;

/// A custom Foxglove schema for a single 32-bit floating-point value.
#[derive(Encode)]
pub struct Float {
    pub value: f32
}

/// A custom Foxglove schema for a single 32-bit integer value.
#[derive(Encode)]
pub struct Integer {
    pub value: i32
}

/// A custom Foxglove schema for a single boolean value.
#[derive(Encode)]
pub struct Bool {
    pub value: bool
}
