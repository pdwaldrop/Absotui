const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn changelog() -> String {
    let mut changelog = String::new();

let changelog_01 = "Changelog Toutui v0.1.0-beta (02/21/2025) \n\
         Fixed:\n\
         \n\
         First release.
         \n\
         Changed:\n\
         \n\
         First release.
         \n\
         Enjoy!\n
         ####\n".to_string();
let changelog_02 = "Changelog Toutui v0.1.1-beta (02/24/2025) \n\
         Fixed:\n\
         \n\
         - App crash (out of bounds) when API send empty values.
         - Close listening session not always working (bug_id: fixed_dd9a64)
         \n\
         Changed:\n\
         \n\
         No change.
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
let changelog_03 = "Changelog Toutui v0.1.2-beta (02/24/2025) \n\
         Fixed:\n\
         \n\
         - Partially fixed, becsause not optimal: bug_id: 9bacac Sync: If you open VLC to listen X, close VLC and quickly open VLC again to listen Y: X will still be sync — according to Y (normally, only Y has to be sync in this case).

         \n\
         Changed:\n\
         \n\
         No change.
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
let changelog_04 = "Changelog Toutui v0.1.3-beta (02/03/2025) \n\
         Fixed:\n\
         \n\
         - Fix bug_id: 3f729c Loading time not optimized for library with a lot of items (long start loading and refresh time)
         \n\
         Changed:\n\
         \n\
         - Script `hello_toutui` to make installation easier.
         \n\
         Contributors:\n\
         \n\
         - dougy147, dhonus
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
let changelog_05 = "Changelog Toutui v0.2.0-beta (07/03/2025) \n\
CAUTION: This version is not compatible with the previous one.  
You need to remove the database in ~/.config/toutui before proceeding. 
         Fixed:\n\
         \n\
         - From known_bugs.md, fixed:

    Find a robust solution for bug_id: 9bacac
    Fix bug_id: 86384e
    Fix bug_id: 6ac5d8
    Fix bug_id: 06e548
    Fix bug_id: e0b61c
    Fix bug_id: fc695f
    Fix bug_id: 40f48d
    Fix bug_id: bf10cd

         \n\
         Changed:\n\
         \n\
         - 
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
let changelog_06 = "Changelog Toutui v0.3.0-beta (24/03/2025) \n\
CAUTION: This version is not compatible with the previous one.  
To make it work properly, perform a fresh reinstall.
\n\
         Added:\n\
         - Integrated player. Keep calm and stay in your terminal! :)
         \n\
         Fixed:\n\
         \n\
         - Fixed: issue where pressing R twice was required to refresh the app.
         - Fixed: issue causing the cursor to disappear when the application is closed.
         - Fixed: issue if app is quitted for the first time and that listening session is empty.
         \n\
         Changed:\n\
         \n\
         - Faster loading time to play an item.
         - Improved synchronization accurary.
         - Removed warning during compilation time.
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID, dougy147
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
let changelog_07 = "Changelog Toutui v0.3.1-beta (25/03/2025) \n\
CAUTION: This version is not compatible with v0.2.0-beta and bellow.  
To make it work properly, perform a fresh reinstall.
\n\
         Fixed:\n\
         \n\
         - Fixed: incorrect merge
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
let changelog_08 = "Changelog Toutui v0.3.2-beta (26/03/2025) \n\
         Added:\n\
         \n\
         - macOS compatibility.
         \n\
         Fixed:\n\
         \n\
         - Issue with VLC buffer (if a chapter is manually changed or during jump/backward).
         - Display issue on small monitors.
         \n\
         Changed:\n\
         \n\
         - hello_toutui script improved
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID, dougy147
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
let changelog_09 = "Changelog Toutui v0.3.3-beta (02/04/2025) \n\
         \n\
         Changed:\n\
         \n\
         - Adding a login placeholder to specify the use of http:// or https:// for the server address.
         - Display error login message without time limit.
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
let changelog_10 = "Changelog Toutui v0.3.4-beta (23/04/2025) \n\
         \n\
         Fix:\n\
         \n\
         Handle empty podcast episode lists gracefully. Prevent panic and show 'No episodes' message. by @denispol in https://github.com/AlbanDAVID/Toutui/pull/22\n\
         Contributors:\n\
         \n\
         - AlbanDAVID, denispol
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
let changelog_11 = "Changelog Toutui v0.3.5-beta (27/04/2025) \n\
         \n\
         Added:\n\
         - Display number of total items for continue listening, library and library settings (for books and podcasts)
         - Clap crate and a function to display the version in the CLI (e.g. `toutui --version`)
         \n\
         Fixed:\n\
         \n\
         - [macos] vlc version not displayed in listening sessions (from ABS web browser)
         - Out of bounds in Library Settings
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
let changelog_12 = "Changelog Toutui v0.4.0-beta (10/05/2025) \n\
         \n\
         Warning:\n\
         - If you're already using the app, please follow the upgrade instructions here: => 
         https://github.com/AlbanDAVID/Toutui/wiki/Major-upgrade-instruction#v--035-beta-to-v040-beta

         Added:\n\
         - Simplified installation and updates by: 
            - Downloading the binary.
            - Compiling it from source (no local clone needed).

         -  New commands available:
            - toutui --update and toutui --uninstall cmd added.

         - Notify if an update is available directly in the app.

         - [Linux only] The app can now be launched via an app launcher.
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID, dougy147
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
let changelog_13 = "Changelog Toutui v0.4.1-beta (14/05/2025) \n\
         \n\
         Warning:\n\
         - If you're already using the app v0.3.5 or bellow, please follow the upgrade instructions here: => 
         https://github.com/AlbanDAVID/Toutui/wiki/Major-upgrade-instruction#v--035-beta-to-v040-beta

         Added:\n\
         - Archlinux users: the app is now available in the AUR (yay -S toutui)

         Changed:\n\
         - Minor changes in the installation process.

         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
let changelog_14 = "Changelog Toutui v0.4.2-beta (15/05/2025) \n\
         \n\
         Warning:\n\
         - If you're already using the app v0.3.5 or bellow, please follow the upgrade instructions here: =>
         https://github.com/AlbanDAVID/Toutui/wiki/Major-upgrade-instruction#v--035-beta-to-v040-beta

         Added:\n\
         - Verifying file integrity using SHA-256 before installation via curl script

         Changed:\n\
         - Clarification of update/uninstall instructions

         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
let changelog_15 = "Changelog Absotui v0.5.0-beta (18/07/2026) \n\
         \n\
         This is a fork of Toutui, renamed Absotui, continuing development independently.
         \n\
         Added:\n\
         - Progress bars and time/duration display for both books and podcast episodes
         - Podcast Home list reworked into a \"New & Unfinished\" view (merging Continue
           Listening and Newest Episodes, actively filtered by real finished status)
         - Now-playing marker, age labels (\"1Day\", \"2Weeks\"...), and a sort-by-age
           toggle (D) for the podcast list
         - Podcast Autoplay setting: automatically start the next episode when one finishes
         - Speed-adjusted vs raw content time toggle (T) for Elapsed/Left display
         - Switching libraries in Settings now applies immediately, no manual refresh needed

         Fixed:\n\
         - Podcast episodes were never actually detected as finished (progress lookup used
           the wrong API shape), so finished episodes never left the list
         - Crash (integer underflow) when jumping backward during podcast playback
         - Podcast player/list title inconsistency (now always \"Episode Title | Podcast Title\")
         - Progress showing over 100% at non-1x playback speed
         - ebookProgress deserialization failure for items with mixed audio/ebook progress
         \n\
         Changed:\n\
         - Modernized dependencies and Rust edition (2021 to 2024)
         - Renamed project from Toutui to Absotui throughout
         \n\
         Enjoy!\n
         ####\n".to_string();
let changelog_16 = "Changelog Absotui v0.5.1-beta (19/07/2026) \n\
         \n\
         Fixed:\n\
         - Install script's OS/distro detection only matched a handful of hardcoded
           distro names, so derivatives like CachyOS, Manjaro, Pop!_OS, and Linux Mint
           fell through to \"unknown\" and aborted the install. Now reads the
           standardized ID_LIKE field from /etc/os-release instead.
         - The checksums for config.example.toml, absotui.desktop, and the release
           binaries were still the original Toutui project's values, which would have
           failed verification for every install regardless of distro.
         - The hello_absotui.sh checksum baked into `absotui --update`/`--uninstall`
           was stale after the above script fixes, breaking update/uninstall for
           anyone who had already installed the binary.
         \n\
         Enjoy!\n
         ####\n".to_string();
let changelog_17 = "Changelog Absotui v0.5.2-beta (19/07/2026) \n\
         \n\
         Fixed:\n\
         - The install script's release checksums (for config.example.toml,
           absotui.desktop, and the binaries) went stale again immediately after the
           previous release, since CI only builds the real files after a release is
           cut. Checksums are no longer hardcoded in the script at all - it now
           fetches a SHA256SUMS.txt manifest that CI generates from the release's
           actual uploaded assets, so this class of bug can't recur.
         \n\
         Enjoy!\n
         ####\n".to_string();
let changelog_18 = "Changelog Absotui v0.5.3-beta (19/07/2026) \n\
         \n\
         Added:\n\
         - Podcast episodes now show cover art next to their description in Continue
           Listening, matching audiobooks - preferring the episode's own embedded
           artwork when its audio file has one, falling back to the podcast's cover
           otherwise.
         \n\
         Fixed:\n\
         - The podcast Home list's selection cursor could appear to drift to a
           different episode on its own every few seconds - the periodic background
           refresh (which keeps finished episodes from lingering in the list) reordered
           the list without preserving which episode was selected.
         \n\
         Enjoy!\n
         ####\n".to_string();
let changelog_19 = "Changelog Absotui v0.5.4-beta (19/07/2026) \n\
         \n\
         Added:\n\
         - A volume indicator in the player bar (VLC's own volume is only ever
           adjusted relatively, so absotui now tracks and displays it).
         - Settings > Per-Item Speed: books and podcast shows can each remember their
           own playback speed instead of sharing a single global speed.
         - A custom app icon, replacing the generic system one in your application
           launcher.
         \n\
         Fixed:\n\
         - Descriptions with HTML markup could lose text after a stray \"<\", double-decode
           entities, or fail to recognize self-closing <br/> tags.
         - Mouse wheel/trackpad scroll no longer hijacks the list selection.
         - Podcast Autoplay: the previous episode's VLC process wasn't being closed
           before starting the next one, breaking pause and progress sync for it; a
           race could start two playback sessions at once after a manual replay; the
           next episode is now picked from the live list instead of a stale snapshot;
           a blocking network call could freeze the whole UI right after a transition;
           and a finished session could keep getting shown as \"now playing\".
         - Playback speed no longer displays as an ugly float like 1.3000001 after
           repeated adjustments.
         \n\
         Enjoy!\n
         ####\n".to_string();
let changelog_20 = "Changelog Absotui v0.5.5-beta (19/07/2026) \n\
         \n\
         Fixed:\n\
         - The install/update/uninstall script's self-integrity check referenced
           variables an outer wrapper never actually set, which crashed the installer
           immediately for virtually everyone before any real install/update/uninstall
           logic could run - and, in the narrower case of running the script from
           inside a checked-out clone, could delete the script's own source file
           instead of just failing safely. The check now verifies the running script
           against the latest release's real published checksum, the same mechanism
           already used for every other downloaded file, and no longer deletes
           anything on a mismatch.
         \n\
         Enjoy!\n
         ####\n".to_string();
let changelog_21 = "Changelog Absotui v0.5.6-beta (19/07/2026) \n\
         \n\
         Fixed:\n\
         - `absotui --update`/`--uninstall` embedded a checksum for hello_absotui.sh
           directly in the compiled binary, which went stale the moment the script
           changed again (as it just did) and silently broke update/uninstall for
           anyone on an older binary - the exact class of bug this checksum handling
           was already supposed to have eliminated everywhere else. The binary no
           longer hardcodes anything here; hello_absotui.sh already verifies itself
           against the latest release's real checksum at run time, so there's nothing
           left for it to keep in sync. `--update`/`--uninstall` also now correctly
           report failure instead of always exiting 0 regardless of what happened.
         \n\
         Enjoy!\n
         ####\n".to_string();
let changelog_22 = "Changelog Absotui v0.5.7-beta (19/07/2026) \n\
         \n\
         Added:\n\
         - `F` in the podcast Home list marks the selected episode finished and
           removes it from New & Unfinished immediately, without waiting on it to
           actually be played through.
         \n\
         Fixed:\n\
         - Cleaned up inconsistent footer key-hint text across screens: \"Settings\"
           was capitalized while other screen names (like \"library\"/\"home\") weren't,
           \"top/bot\" vs \"top/bottom\" varied by screen for the same binding, some
           footers spelled out \"J(down) K(up) H(top)\" while others used arrows, and
           one Settings screen's footer had a leftover typo (\"Scroll :\" missing a
           word). All footers now share the same wording, built from one place so
           this can't drift between screens again as new ones get added.
         \n\
         Enjoy!\n
         ####\n".to_string();
let changelog_23 = format!(
    "Changelog Absotui v{VERSION} (19/07/2026) \n\
         \n\
         Fixed:\n\
         - `absotui --update` never refreshed absotui.desktop or the app icon, only a
           fresh install did - so the custom icon added in v0.5.4-beta went unnoticed
           by anyone who updated instead of reinstalling. Updating now refreshes both,
           same as installing does.
         \n\
         Enjoy!\n
         ####\n"
);


    changelog.push_str(&changelog_23);
    changelog.push_str(&changelog_22);
    changelog.push_str(&changelog_21);
    changelog.push_str(&changelog_20);
    changelog.push_str(&changelog_19);
    changelog.push_str(&changelog_18);
    changelog.push_str(&changelog_17);
    changelog.push_str(&changelog_16);
    changelog.push_str(&changelog_15);
    changelog.push_str(&changelog_14);
    changelog.push_str(&changelog_13); 
    changelog.push_str(&changelog_12); 
    changelog.push_str(&changelog_11); 
    changelog.push_str(&changelog_10); 
    changelog.push_str(&changelog_09); 
    changelog.push_str(&changelog_08); 
    changelog.push_str(&changelog_07); 
    changelog.push_str(&changelog_06); 
    changelog.push_str(&changelog_05); 
    changelog.push_str(&changelog_04); 
    changelog.push_str(&changelog_03); 
    changelog.push_str(&changelog_02); 
    changelog.push_str(&changelog_01); 


changelog
}
