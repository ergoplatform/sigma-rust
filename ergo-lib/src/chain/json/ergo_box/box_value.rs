#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::chain::ergo_box::ErgoBox;

    #[test]
    fn parse_value_as_num() {
        let box_json = r#"{
          "boxId": "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e",
          "value": 67500000000,
          "ergoTree": "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
          "assets": [],
          "creationHeight": 284761,
          "additionalRegisters": {},
          "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
          "index": 1
        }"#;
        let b: ErgoBox = serde_json::from_str(box_json).unwrap();
        assert_eq!(b.value, 67500000000u64.try_into().unwrap());
    }

    #[test]
    fn parse_value_as_str() {
        let box_json = r#"{
          "boxId": "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e",
          "value": "67500000000",
          "ergoTree": "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
          "assets": [],
          "creationHeight": 284761,
          "additionalRegisters": {},
          "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
          "index": 1
        }"#;
        let b: ErgoBox = serde_json::from_str(box_json).unwrap();
        assert_eq!(b.value, 67500000000u64.try_into().unwrap());
    }
}
