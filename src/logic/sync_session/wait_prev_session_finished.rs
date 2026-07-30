use crate::db::crud::{get_is_vlc_launched_first_time, get_is_loop_break, update_is_loop_break, update_is_vlc_launched_first_time, try_claim_playback_slot};
use log::info;
use crate::utils::pop_up_message::{pop_message, clear_message};
use std::io::stdout;

// Blocks until the previously-running playback task (if any) has finished its own
// close/sync and released the playback slot, so a new track can't start while the old
// one is still tearing down.
//
// Callers deliberately do NOT follow this with a defensive close of whatever's left in
// `listening_session`. Every normal exit path (Q, natural end-of-track, VLC-quit
// detection) closes its own session before returning, so the only way that row can be
// stale-and-never-closed is an abnormal exit (crash/kill) - and `main.rs` now closes
// that case once at startup instead. Running it here as well closed the session twice
// on every track switch (confirmed live 2026-07-28: once from the previous track's own
// quit-detection loop, once from the caller) - the concrete root cause behind bug_id
// dd9a649.
pub fn wait_prev_session_finished(username: String) {

    // pop message
    let message = "Syncing your last listening session. Please wait...";
    let mut stdout = stdout();

        // check if previous play is finished
        let is_vlc_first_launch = get_is_vlc_launched_first_time(&username);
        info!("[AppView::Home][is_vlc_first_launch]{is_vlc_first_launch}");
        let was_first_launch = is_vlc_first_launch == "1";

        if !was_first_launch {
            // Loops trying to atomically claim the slot (flip is_loop_break 1->0)
            // instead of a separate check-then-set - two "l" presses close enough
            // together could otherwise both observe is_loop_break=="1" before either
            // wrote "0", and both proceed to start a VLC session at once. Once this
            // call succeeds, is_loop_break is already "0" - no separate update needed.
            while !try_claim_playback_slot(&username) {
                std::thread::sleep(std::time::Duration::from_secs(1));
                info!("[AppView::Home][loop][is_loop_break]");
                let _ = pop_message(&mut stdout, 3, message);
            }
        } else {
            // First launch ever for this account: is_loop_break starts at "0" (see
            // auth_process.rs), so the claim above would never succeed here - nothing
            // to wait on yet, just land on the same "0" end state directly.
            let _ = update_is_loop_break("0", &username);
        }
        let value = get_is_loop_break(username.as_str());
        info!("[AppView::Home][update_is_loop_break]{value}");
        let _ = update_is_vlc_launched_first_time("0", &username);
        let value = get_is_vlc_launched_first_time(username.as_str());
        info!("[AppView::Home][update_is_vlc_first_launch]{value}");

        // clear pop up message
        let _ = clear_message(&mut stdout, 3);

}
