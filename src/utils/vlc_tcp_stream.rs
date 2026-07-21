use std::io::{self, Write, BufRead, BufReader};
use std::net::TcpStream;
use std::time::Duration;

pub fn vlc_tcp_stream(address: &str, port: &str, cmd: &str) -> io::Result<String> {
    let mut stream = TcpStream::connect(format!("{address}:{port}"))?;

    // Called every ~1s from the playback sync loop with no timeout previously set - if
    // VLC is alive but stops responding (stalled/deadlocked, rather than actually
    // closed), the blocking .lines().nth(2) read below would otherwise hang forever,
    // freezing that whole session's sync/progress loop with no recovery. 5s is well
    // above VLC's normal near-instant RC response time.
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;

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


