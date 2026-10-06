use std::{env, fs};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let inventory: toml::Value = toml::from_str(&fs::read_to_string(&args[1]).unwrap()).unwrap();
    assert_eq!(inventory["version"].as_integer(), Some(1));
    let blur = inventory["fields"]
        .as_array()
        .unwrap()
        .iter()
        .find(|field| field["path"].as_str() == Some("window.blur"))
        .unwrap();
    assert!(
        blur["choices"]
            .as_array()
            .unwrap()
            .iter()
            .any(|choice| choice.as_bool() == Some(true))
    );
    let config: toml::Value = toml::from_str(&fs::read_to_string(&args[2]).unwrap()).unwrap();
    assert_eq!(config["window"]["blur"].as_bool(), Some(true));
}
