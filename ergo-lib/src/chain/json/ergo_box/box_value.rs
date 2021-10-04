#[cfg(test)]
mod tests {
    use crate::chain::ergo_box::BoxValue;
    use std::convert::TryInto;

    #[test]
    fn parse_value_as_str() {
        let json = "\"67500000000\"";
        let v: BoxValue = serde_json::from_str(json).unwrap();
        assert_eq!(v, 67500000000u64.try_into().unwrap());
    }

    #[test]
    fn parse_value_as_num() {
        let json = "67500000000";
        let v: BoxValue = serde_json::from_str(json).unwrap();
        assert_eq!(v, 67500000000u64.try_into().unwrap());
    }

    #[ignore = "moved to wasm"]
    #[test]
    fn encode_value_as_str() {
        let json = "\"67500000000\"";
        let v: BoxValue = serde_json::from_str(json).unwrap();
        assert_eq!(v, 67500000000u64.try_into().unwrap());
        let to_json = serde_json::to_string_pretty(&v).unwrap();
        assert_eq!(to_json, json);
    }
}
