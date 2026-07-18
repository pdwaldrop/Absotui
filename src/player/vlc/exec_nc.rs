use std::process::Command;
use std::process::Output;

pub async fn exec_nc(port: &str, address: String) -> Output {
    let output: Output = Command::new("kitty")
        .arg("nc")
        .arg(&address)
        .arg(port)
        .output()
        .expect("Failed to execute program");

    output
}

