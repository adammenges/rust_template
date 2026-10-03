fn main() {
    println!("cargo:rerun-if-changed=app.json");
    let data: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string("app.json").expect("read app.json"))
            .expect("parse app.json");
    let fields = [
        "package",
        "executable",
        "display_name",
        "identifier",
        "storage_name",
    ];
    let object = data
        .as_object()
        .expect("app.json must be an identity object");
    assert!(
        object.len() == fields.len()
            && fields
                .iter()
                .all(|key| object.get(*key).is_some_and(serde_json::Value::is_string)),
        "app.json must contain exactly the five string identity fields; run scripts/rename.py --check"
    );
    for key in ["package", "executable", "storage_name"] {
        assert!(
            valid_slug(data[key].as_str().unwrap()),
            "Invalid {key}; run scripts/rename.py --check"
        );
    }
    let identifier = data["identifier"].as_str().unwrap();
    assert!(
        identifier.len() <= 255
            && identifier.contains('.')
            && identifier.split('.').all(valid_slug),
        "Invalid application identifier"
    );
    let display = data["display_name"].as_str().unwrap();
    assert!(
        !display.is_empty()
            && display.len() <= 64
            && display.as_bytes()[0].is_ascii_alphanumeric()
            && display
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b" ._-".contains(&c)),
        "Invalid display name"
    );
    assert_eq!(
        data["package"].as_str().unwrap(),
        std::env::var("CARGO_PKG_NAME").unwrap(),
        "identity drift: run scripts/rename.py"
    );
    let mut source = String::new();
    for (key, constant) in [
        ("display_name", "DISPLAY_NAME"),
        ("identifier", "APP_ID"),
        ("storage_name", "STORAGE_NAME"),
        ("executable", "EXECUTABLE"),
    ] {
        source.push_str(&format!(
            "pub const {constant}: &str = {:?};\n",
            data[key].as_str().unwrap()
        ));
    }
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    std::fs::write(out.join("identity.rs"), source).expect("write identity constants");
}

fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 63
        && value.as_bytes()[0].is_ascii_lowercase()
        && value
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
