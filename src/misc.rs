use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i64 {
    a as i64 + b as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_test() {
        assert_eq!(add(2, 2), 4);
    }
}
