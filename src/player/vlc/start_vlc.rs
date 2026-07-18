use std::process::Command;
use std::process::Output;
use crate::db::crud::get_speed_rate;

pub async fn start_vlc(
    current_time: &String, 
    port: &str, 
    address: String,
    content_url: &String, 
    token: Option<&String>, 
    title: String, 
    subtitle: String, 
    author: String, 
    server_address: String, 
    program: String, 
    username: String,
) -> Output {

    let speed_rate = get_speed_rate(username.as_str());

    let output: Output = Command::new(&program)
        .arg("-I") // for macos
        .arg("dummy") // for macos
        .arg(format!("--start-time={current_time}"))
        .arg("--extraintf")
        .arg("rc")
        .arg("--rc-host")
        .arg(format!("{address}:{port}"))
        .arg(format!("{}{}?token={}", server_address, content_url, token.unwrap()))
        .arg("--rate")
        .arg(speed_rate)
        .arg("--meta-description")
        .arg(author)
        .arg("--meta-title")
        .arg(subtitle)
        .arg("--meta-artist")
        .arg(title)
        .output()
        .expect("Failed to execute program");

    output
}

