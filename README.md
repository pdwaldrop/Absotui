## ℹ️ This is a fork
Absotui is a fork of [Toutui](https://github.com/AlbanDAVID/Toutui), a TUI Audiobookshelf client whose original author archived it in December 2025 ("I'm not able to properly maintain this project anymore... please don't wait for any new releases and issue fixing."). This fork exists to keep the project going and as a personal project to learn Rust, so expect things to move slowly and change shape as we go.

[![GitHub release](https://img.shields.io/github/v/release/pdwaldrop/Absotui?label=Latest%20Release&color=green&cacheSeconds=3600)](https://github.com/pdwaldrop/Absotui/releases/latest)
[![Release](https://github.com/pdwaldrop/Absotui/actions/workflows/release.yml/badge.svg)](https://github.com/pdwaldrop/Absotui/actions/workflows/release.yml)

# 🦜 Absotui: A TUI Audiobookshelf client for Linux and macOS

<p align="center">
    <em>The name plays on "ABS" (Audiobookshelf) + "TUI" (terminal user interface), read like "absolutely."<br>
    The original project, Toutui, took its name from the French phrase "tout ouïe" ("all ears").</em>
</p>

<p align="center">
    <img src="assets/demo_3.gif" alt="🎬 Demo">
</p>

<div align="center">
🎨 Explore and try various themes <a href="https://github.com/AlbanDAVID/Toutui-theme">here</a> (the original project's theme repo — themes there should still be compatible).
</div>

## ✨ Features  
 **Cross-platform:** <img src=".github/tux.png" align="top" width="24" alt="Tux (Linux)"/>  Linux and <img src=".github/apple.png" align="top" width="24" alt="Apple (macOS)"/> macOS    
 **Lightweight & Fast:** A minimalist terminal user interface (TUI) written in Rust 🦀  
 **Supports Books & Podcasts:** Enjoy both audiobooks and podcasts  
 **Sync Progress & Stats:** Keep your listening progress in sync  
 **Streaming Support:** Play directly without downloading  
 **Customizable Color Theme:** A config file will allow you to customize the color theme. Explore and try various themes [here](https://github.com/AlbanDAVID/Toutui-theme).

## 🛠️ Roadmap  
This fork just got started, so there's no fixed roadmap yet — for now the focus is on getting it running reliably and learning the codebase. See [known bugs](known_bugs.md) for the current state of things inherited from the original project.

## 🔮 Future features
Here are some features that could be added in future releases:
- Playlist/Collections view
- Ability to add new podcasts from the app
- Add stats
- Offline mode
  
## ⚠️ Caution: Beta Version  
This app is still in **heavy development and contains bugs**.  
❗Please check [here](known_bugs.md) for known bugs especially **MAJOR BUGS** before using the app, so you can use it with full awareness of any known issues.  
If you encounter any issues that are **not yet listed** in the Issues section or [known bugs](known_bugs.md), please **open a new issue** to report them.  

🔐 Although it's a beta version, you can use this app with **minimal risk** to your Audiobookshelf library.  
At worst, you may experience **sync issues**, but there is **no risk** of data loss, deletion, or irreversible changes (API is just used to retrieve books and sync them).

## 📝 Notes
### 🐛 **Issues**    
For any issues, check first the [issues](https://github.com/pdwaldrop/Absotui/issues) here. Otherwise, open a new one. (Also worth checking the [original project's wiki](https://github.com/AlbanDAVID/Toutui/wiki/) for general usage help, since most of the underlying app hasn't changed yet.)

### 🤝 **Contributing**  
Do not hesitate to contribute to this project by submitting your code, ideas, or feedback. Please make sure to read the [contributing guidelines](CONTRIBUTING.md) first.

### 🔁 Branching workflow 
This project follows this [branching workflow](https://gist.github.com/digitaljhelms/4287848). 

### 🎨 **UI**
Explore and share themes [here](https://github.com/AlbanDAVID/Toutui-theme).    
The **font** and **emojis** may vary depending on the terminal you are using.    
To ensure the best experience, it's recommended to use **Kitty** or **Alacritty** terminal.



## 🚨 Installation Instructions

>[!WARNING]
> - **This is a beta app, please read [this](#%EF%B8%8F-caution-beta-version).**
>  - For any issues, check first the [issues](https://github.com/pdwaldrop/Absotui/issues) here. Otherwise, open a new one.

>[!NOTE]
> There's no AUR package or prebuilt binary release for this fork yet — for now, building from source is the reliable path. The install script below is present but its binary-download option won't work until releases exist.

### ⚡ Easy installation (install script)

**Run the following in your terminal, then follow the on-screen instructions:**    

```bash
bash -c 'expected_sha256="616c538920c51fe21fab3f33009ecf70c462923613128790e1affea7322ca94f" export expected_sha256 tmpfile=$(mktemp) && curl -LsSf https://github.com/pdwaldrop/Absotui/raw/stable/hello_absotui.sh -o "$tmpfile" && bash "$tmpfile" install && rm -f "$tmpfile"'
```

#### **Update**

> [!IMPORTANT]  
> `absotui --update` is not working. You can do this instead: 
> ``` 
> bash -c 'expected_sha256="616c538920c51fe21fab3f33009ecf70c462923613128790e1affea7322ca94f" export expected_sha256 tmpfile=$(mktemp) && curl -LsSf https://github.com/pdwaldrop/Absotui/raw/stable/hello_absotui.sh -o "$tmpfile" && bash "$tmpfile" update && rm -f "$tmpfile"'
> ```

Quit the app and run the following in your terminal

```bash
bash -c 'expected_sha256="616c538920c51fe21fab3f33009ecf70c462923613128790e1affea7322ca94f" export expected_sha256 tmpfile=$(mktemp) && curl -LsSf https://github.com/pdwaldrop/Absotui/raw/stable/hello_absotui.sh -o "$tmpfile" && bash "$tmpfile" update && rm -f "$tmpfile"'
```

#### **Uninstall**

> [!IMPORTANT]  
> `absotui --uninstall` is not working. You can do this instead: 
> ``` 
> bash -c 'expected_sha256="616c538920c51fe21fab3f33009ecf70c462923613128790e1affea7322ca94f" export expected_sha256 tmpfile=$(mktemp) && curl -LsSf https://github.com/pdwaldrop/Absotui/raw/stable/hello_absotui.sh -o "$tmpfile" && bash "$tmpfile" uninstall && rm -f "$tmpfile"'
> ```

Quit the app and run the following in your terminal


```bash
bash -c 'expected_sha256="616c538920c51fe21fab3f33009ecf70c462923613128790e1affea7322ca94f" export expected_sha256 tmpfile=$(mktemp) && curl -LsSf https://github.com/pdwaldrop/Absotui/raw/stable/hello_absotui.sh -o "$tmpfile" && bash "$tmpfile" uninstall && rm -f "$tmpfile"'
```

#### **Notes**  

##### Files installed:
In `/usr/local/bin` (option 1, from install script) or `~/.cargo/bin` (option 2, from install script):
- `absotui` - The binary file.

In `~/.config/absotui` for Linux or `~/Library/Preferences` for macOS:    
**Note**: This is the default path if `XDG_CONFIG_HOME` is empty. 
- `.env` - Contains the secret key.
- `config.toml` - Configuration file.
- `absotui.log` - Log file.
- `db.sqlite3` - SQLite database file.

In `~/.local/share/applications` for Linux:
- `absotui.desktop` - Config file to launch Absotui from a launcher app.

### Install from source

>[!WARNING]
> This is a beta app, please read [this](#%EF%B8%8F-caution-beta-version).  

#### **Requirements**
- `Rust`
- `Netcat`
- `VLC`

Note: `main` might be unstable. Prefer `git clone --branch stable --single-branch https://github.com/pdwaldrop/Absotui` if you want to have the last stable release (once one exists).    
```bash
git clone https://github.com/pdwaldrop/Absotui
cd Absotui/
mkdir -p ~/.config/absotui
cp config.example.toml ~/.config/absotui/config.toml
```

Token encryption in the database (<u>**NOTE**</u>: replace `secret`)
```bash
echo ABSOTUI_SECRET_KEY=secret >> ~/.config/absotui/.env
```

```bash
cargo run --release
```

#### **Update**

When a new release is available, follow these steps:

```bash
git pull https://github.com/pdwaldrop/Absotui
cargo run --release
```

#### **Notes**  
##### Exec the binary:
```bash
cd target/release
./absotui
```

##### Files installed:
After installation, you will have the following files in `~/.config/absotui`
- `.env` - Contains the secret key.
- `config.toml` - Configuration file.
- `absotui.log` - Log file.
- `db.sqlite3` - SQLite database file.
