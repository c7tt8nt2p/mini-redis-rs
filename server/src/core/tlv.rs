const TLV_LENGTH_SIZE: usize = 8;

#[derive(Debug, Eq, PartialEq)]
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
            form_tlv(tlv_type, value)
        }
        TLVType::Int => {
            let tlv_type: [u8; 1] = [TLVType::Int as u8];
            form_tlv(tlv_type, value)
        }
    }
}

fn form_tlv(tlv_type: [u8; 1], tlv_value: Vec<u8>) -> Vec<u8> {
    let mut tlv = Vec::new();
    tlv.extend(tlv_type);
    tlv.extend(tlv_value.len().to_be_bytes());
    tlv.extend(tlv_value);
    tlv
}

/// Given a tlv byte array, converts it to the value
pub fn from_tlv(value: Vec<u8>) -> Vec<u8> {
    let Some(tlv_type) = TLVType::from_u8(value[0]) else { return Vec::new(); };
    match tlv_type {
        TLVType::String | TLVType::Int => {
            let tlv_length = &value[1..=TLV_LENGTH_SIZE];
            let value_length = usize::from_be_bytes(tlv_length.try_into().unwrap());
            value[TLV_LENGTH_SIZE + 1..=TLV_LENGTH_SIZE + value_length].to_vec()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::tlv::{form_tlv, from_tlv, TLVType, to_tlv};

    #[test]
    fn test_tlv_type_from_u8() {
        assert_eq!(Some(TLVType::String), TLVType::from_u8(1));
        assert_eq!(Some(TLVType::Int), TLVType::from_u8(2));
        assert_eq!(None, TLVType::from_u8(3));
    }

    #[test]
    fn test_to_tlv_string_type() {
        let message = "hello".as_bytes().to_vec();
        let tlv = to_tlv(message, TLVType::String);
        assert_eq!(
            vec![1, 0, 0, 0, 0, 0, 0, 0, 5, 104, 101, 108, 108, 111],
            tlv
        );
    }

    #[test]
    fn test_to_tlv_int_type() {
        let message = "123".as_bytes().to_vec();
        let tlv = to_tlv(message, TLVType::String);
        assert_eq!(vec![1, 0, 0, 0, 0, 0, 0, 0, 3, 49, 50, 51], tlv);
    }

    #[test]
    fn test_form_tlv() {
        let value = "tee al vee".as_bytes().to_vec();
        let tlv_type: [u8; 1] = [TLVType::String as u8];

        let tlv = form_tlv(tlv_type, value);
        assert_eq!(
            vec![1, 0, 0, 0, 0, 0, 0, 0, 10, 116, 101, 101, 32, 97, 108, 32, 118, 101, 101],
            tlv
        );
    }

    #[test]
    fn test_from_tlv() {
        let tlv = vec![
            1, 0, 0, 0, 0, 0, 0, 0, 10, 116, 101, 101, 32, 97, 108, 32, 118, 101, 101,
        ];
        let data = from_tlv(tlv);
        assert_eq!(vec![116, 101, 101, 32, 97, 108, 32, 118, 101, 101], data);
    }
}
