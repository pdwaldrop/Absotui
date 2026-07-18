use crate::db::crud::{get_is_vlc_launched_first_time, get_is_loop_break, update_is_loop_break, update_is_vlc_launched_first_time};
use log::info;
use crate::utils::pop_up_message::{pop_message, clear_message};
use std::io::stdout;

pub fn wait_prev_session_finished(username: String) {

    // pop message
    let message = "Syncing your last listening session. Please wait...";
    let mut stdout = stdout();

        // check if previous play is finished
        let is_vlc_first_launch = get_is_vlc_launched_first_time(&username);
        info!("[AppView::Home][is_vlc_first_launch]{is_vlc_first_launch}");

        if is_vlc_first_launch != "1" {
            let mut is_loop_break = get_is_loop_break(&username);
            info!("[AppView::Home][is_loop_break]{is_loop_break}");

            while is_loop_break != "1" {
                std::thread::sleep(std::time::Duration::from_secs(1));
                info!("[AppView::Home][loop][is_loop_break]");
                is_loop_break = get_is_loop_break(&username);
                let _ = pop_message(&mut stdout, 3, message);
            }

        }

        // update database
        let _ = update_is_loop_break("0", &username);
        let value = get_is_loop_break(username.as_str());
        info!("[AppView::Home][update_is_loop_break]{value}");
        let _ = update_is_vlc_launched_first_time("0", &username);
        let value = get_is_vlc_launched_first_time(username.as_str());
        info!("[AppView::Home][update_is_vlc_first_launch]{value}");

        // clear pop up message 
        let _ = clear_message(&mut stdout, 3);

}
