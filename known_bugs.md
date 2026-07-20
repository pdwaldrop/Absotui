**MAJOR**

No major bug for the moment 🙏

**MINOR**

`bug_id: 4b3045`
**Authentification Bug (fix committed, unreleased):** Even if you fill in valid credentials, the database sync can be buggy, and authentication may fail. Normally, it works on the second try. Root cause confirmed 2026-07-20: the login loop waited a flat 1s guess before re-checking the database instead of actually waiting for the async auth attempt to finish. Fixed locally (commit `47d728b`), not yet released - needs a real logout/login to confirm.

`bug_id: 2eb9e3`
**Display (likely stale):** At the launch, the app is not displayed and no error message appears (especially if you change user, quit and restart the app). Solution: quit the terminal and try it again. Investigated 2026-07-20: could not reproduce via the documented repro (fresh start with no saved user, delete-user-then-restart, and repeated same-process failed logins all rendered correctly) in an isolated test config. Pre-dates the Absotui fork; several related sync/terminal bugs have been fixed since. Leaving filed in case it's terminal-emulator- or timing-specific in a way not reproduced here.

`bug_id: a49eza`
**cvlc error sync with ctrl vlc from a terminal:** If you use other command that `shutdown` to quit `cvlc` it may result of a sync issue.


**FIXED**  
`bug_id: 9bacac` 
**Sync**: If you open VLC to listen X, close VLC and quickly open VLC again to listen Y: X will still be sync — according to Y (normally, only Y has to be sync in this case).   
`bug_id: 86384e` 
**Sync**: Rarely and especially if you open VLC to listen X, close VLC and quickly open VLC again to listen Y: the progress of X is set to 0 seconds.  
`bug_id: 06e548` 
**Terminal broken**: The terminal is broken after the app is quit.  
`bug_id: 6ac5d8` 
**Data loss if app crash or disgracefully quit**: If app crash, the last session is not closed.  
`bug_id: bf10cd` 
**Launch a new media**: Have to close manually VLC to close and sync a session.  
`bug_id: 3f729c` 
**Loading time**: for now, not optimized for a library with a lot of items (long start loading and refresh time)  
`bug_id: dd9a649`
**Listening Session:** Sometimes, the session (that you can see in `yourserveraddress/audiobookshelf/config/sessions`) does not close correctly, especially if you open VLC, quit it quickly, and start another book.  
`bug_id: e0b61c`
**VLC:** `VLC` continue to run after the app is quit.  
`bug_id: fc695f`
**Listening session:** The session (that you can see in `yourserveraddress/audiobookshelf/config/sessions`) does not close when the app is quit.  
`bug_id: 40f48d`
**Cursor:** When you quit the app, terminal cursor disappear.  
`bug_id: fe4116`
**cvlc macOS:** `cvlc` option is not available for now in macOS.  
`bug_id: 2d358c53`
**Mark as finished:** When a title reach the end, mark as finished not always work. Fixed 2026-07-20: the actual bug was marking the *currently-playing* episode finished (the periodic progress sync would clobber it back to unfinished a few seconds later); the natural end-of-track path was verified still working correctly (confirmed live: `isFinished=true` held across 5+ refresh cycles after a real episode played to its end).  
`bug_id: 255b86` **(fix committed, not yet released)**
**Losing config after an update**: Ex: You change colors in config file and after an update, this configuration is lost and replaced by the config from main version. Root cause found 2026-07-20: `hello_absotui.sh`'s config-merge logic had a typo (`pseudo_escape_line` vs `pseudo_escaped_line`) that silently dropped any user config key not present in `config.example.toml`. Fixing just the typo would have exposed a second, worse bug underneath it (bracket-containing lines and section headers getting duplicated on every merge) - both are now fixed together, replacing the fragile `sed`-stripped `grep -E` prefix matching with bash-native literal string comparison. Verified via an isolated test harness against sample configs and a read-only dry run against the real config files - no release cut yet, so this hasn't reached `hello_absotui.sh` on `stable` (what the installer actually pulls from) until one does.
