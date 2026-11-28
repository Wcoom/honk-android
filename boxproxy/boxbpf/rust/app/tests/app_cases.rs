#[test]
fn recorded_cases() {
    trycmd::TestCases::new()
        .register_bin("boxbpf", std::path::Path::new(env!("CARGO_BIN_EXE_boxbpf")))
        .case("../app-cases/*.toml");
}
