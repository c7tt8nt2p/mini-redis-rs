const TLV_LENGTH_SIZE: usize = 8;
pub enum TLVType {
    String = 1,
    Int = 2,
}

impl TLVType {
    pub fn from_u8(value: u8) -> Option<TLVType> {
        match value {
            1 => Some(TLVType::String),
            2 => Some(TLVType::Int),
            _ => None,
        }
    }
}

/// Given a value as byte array, converts it to the tlv
pub fn to_tlv(value: Vec<u8>, tlv_type: TLVType) -> Vec<u8> {
    match tlv_type {
        TLVType::String => {
            let tlv_type: [u8; 1] = [TLVType::String as u8];
            let tlv_length: [u8; TLV_LENGTH_SIZE] = value.len().to_be_bytes();

            form_tlv(tlv_type, tlv_length, value)
        }
        TLVType::Int => {
            let tlv_type: [u8; 1] = [TLVType::Int as u8];
            let tlv_length: [u8; TLV_LENGTH_SIZE] = value.len().to_be_bytes();

            form_tlv(tlv_type, tlv_length, value)
        }
    }
}

fn form_tlv(tlv_type: [u8; 1], tlv_length: [u8; TLV_LENGTH_SIZE], tlv_value: Vec<u8>) -> Vec<u8> {
    let mut tlv = Vec::new();
    tlv.extend(tlv_type);
    tlv.extend(tlv_length);
    tlv.extend(tlv_value);
    tlv
}

/// Given a tlv byte array, converts it to the value
pub fn from_tlv(value: Vec<u8>) -> Vec<u8> {
    let Some(tlv_type) = TLVType::from_u8(value[0]) else { return Vec::new(); };
    match tlv_type {
        TLVType::String | TLVType::Int => {
            let tlv_length = &value[1..TLV_LENGTH_SIZE + 1];
            let value_length = usize::from_be_bytes(tlv_length.try_into().unwrap());
            value[TLV_LENGTH_SIZE + 1..TLV_LENGTH_SIZE + value_length + 1].to_vec()
        }
    }
}
