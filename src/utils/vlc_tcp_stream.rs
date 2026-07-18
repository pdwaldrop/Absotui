use std::io::{self, Write, BufRead, BufReader};
use std::net::TcpStream;

pub fn vlc_tcp_stream(address: &str, port: &str, cmd: &str) -> io::Result<String> {
    let mut stream = TcpStream::connect(format!("{address}:{port}"))?;
    
    // Send command
    writeln!(stream, "{cmd}")?;

    // Read response
    let reader = BufReader::new(stream);
    // we need to select third line (the one where the repsonse is displayed)
    let response = reader
        .lines()
        .nth(2)
        .unwrap_or(Ok("N/A".to_string()))?
        .replace("> ", "") // Remove " > "
        .trim() // Trim any extra spaces
        .to_string();

    Ok(response)

}


