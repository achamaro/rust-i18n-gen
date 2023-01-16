use std::fs::write;

use rust_i18n_gen::{generate, load_resources};

fn main() {
    let code = generate(
        "ja",
        &mut load_resources(&vec!["../example/lang".to_string()]),
    );

    write("src/lib.rs", code).unwrap();
}
