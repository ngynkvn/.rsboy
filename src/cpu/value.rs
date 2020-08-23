pub enum Value {
    U16(u16),
    U8(u8),
}

impl Into<u8> for Value {
    fn into(self) -> u8 {
        if let Value::U8(value) = self {
            value
        } else {
            panic!("Tried to convert U16 into U8.")
        }
    }
}

impl Into<u16> for Value {
    fn into(self) -> u16 {
        if let Value::U16(value) = self {
            value
        } else {
            panic!("Tried to convert U16 into U8.")
        }
    }
}