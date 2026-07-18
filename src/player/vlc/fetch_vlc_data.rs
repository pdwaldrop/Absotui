use std::io::{self};
use vlc_rc::Client;
use tokio::net::TcpStream;
use std::process::Command;
use std::str;
use regex::Regex;
use log::{info, warn, error};

/// This function : 
/// allow to connect and remotely ctrl VLC (with vlc-rc crate) on the port that was provided (`Client::connect(format!("{}`:{}", port))) 
/// if connection is successul, fecth data thanks to remotly control
/// this fn is in the loop and run while vlc is running (bu checking if the port is still open)
pub async fn fetch_vlc_data(port: String, address: String) -> Result<Option<u32>, io::Error> {

    loop {
        // Check if VLC is running, if not, break the loop
        if !is_vlc_running(port.clone(), address.clone()).await {
            break Ok(None); // Exit loop if VLC is not running anymore
        }

        // Connect to VLC and fetch data
        let mut player = match Client::connect(format!("{address}:{port}")) {
            Ok(player) => player,
            Err(e) => {
                error!("[fetch_vlc_data] {e}");
           //     if let Err(file_error) = log_error_to_file(&e.to_string()) {
           //         eprintln!("Failed to log to vlc: {}", file_error);
           //         error!("Failed to log to vlc: {}", file_error);
           //     }
                continue;
            }
        }; 
        // Fetch VLC current time (if connection is successful)
        let seconds = match player.get_time() {
            Ok(Some(value)) => Some(value),
            Ok(None) => None,
            Err(e) => {
                eprintln!("Failed to fetch time from VLC: {e}");
                error!("Failed to fetch time from VLC: {e}");
                None
            }
        };

        // Print and return the fetched seconds
        if let Some(sec) = seconds {
            if sec > 0 {
                return Ok(Some(sec)); // Return seconds once fetched
            }
            info!("[is_vlc_running][check_seconds]: {sec:?}");
        }

        // Sleep to fetch data every second and avoid CPU overload
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

}

// fetch if vlc is playing or stopped (return true if vlc is paused)
pub async fn fetch_vlc_is_playing(port: String, address: String) -> Result<bool, String> {
    // Tentative de connexion à VLC
    let mut player = match Client::connect(format!("{address}:{port}")) {
        Ok(player) => player,
        Err(e) => {
            warn!("[fetch_vlc_is_playing] Failed to connect to VLC at port {port}: {e}");
            return Err(format!("Failed to connect to VLC at port {port}: {e}"));
        }};

    // Tentative de récupération du statut "is_playing"
    let is_playing = match player.is_playing() {
        Ok(true) => {
            //println!("The track is currently playing.");
            true
        }
        Ok(false) => {
            // vlc is still open but we have reached the end of the audio playback
            // allow to be check is the track is finished. But different from the case where VLC is
            // stopped by the user.
            //println!("The track is currently stopped.");
            false
        }
        Err(e) => {
            // vlc is closed ba the the user, as VLC is not open anymore. Indeed, match Client::connect(format!("{}:{}", &port))
            // will send an error because VLC is not open anymore. Allow to differenciate from an
            // reach the end of audio just above. Here, the VLC vlc is closed be the user so we
            // want to make sur to differienciate from a normal reached of the audio playback
            error!("Failed to check the play status of VLC: {e}");
            return Err(format!("Failed to check the play status of VLC: {e}"))
        }
    };

    Ok(is_playing)
}



// check if VLC is running by checking if the port used by the app to open VLC is open
pub async fn is_vlc_running(port: String, address: String) -> bool {
    if TcpStream::connect(format!("{address}:{port}")).await.is_ok() {
        //println!("VLC is still running (port {} is open).", port);
        true
    } else {
        info!("[is_vlc_running] VLC is not running (port {port} is closed).");
        //println!("VLC is not running (port {} is closed).", port);
        false
    }
}

// get vlc version
pub async fn get_vlc_version() -> Result<String, io::Error> {

    let command: &str = if cfg!(target_os = "macos") {
        "/Applications/VLC.app/Contents/MacOS/VLC"
    } else {
        "vlc"
    };

    let output = Command::new(command)
        .arg("--version")
        .output()?;

    let version_output = str::from_utf8(&output.stdout)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?; 

    let re = Regex::new(r"VLC (?:media player |version )?([\d.]+)").unwrap();

    if let Some(captures) = re.captures(version_output)
        && let Some(version) = captures.get(1) {
            return Ok(version.as_str().to_string());
        }

    Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Could not extract VLC version",
    ))
}

//fn log_error_to_file(error_message: &str) -> io::Result<()> {
//    let mut file = OpenOptions::new()
//        .create(true)
//        .append(true)
//        .open("vlc_errors.txt")?;
//    writeln!(file, "{}", error_message)?;
//    Ok(())
//}
//#[allow(dead_code)]
//fn write_to_file(file_path: &str, content: &str) -> io::Result<()> {
//    let mut file = OpenOptions::new()
//        .create(true)
//        .append(true)
//        .open(file_path)?;
//    writeln!(file, "{}", content)?;
//    Ok(())
//}
