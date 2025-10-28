# Tauri + React + Typescript

This template should help get you started developing with Tauri, React and Typescript in Vite.



## Setup

### Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

### Git
Clone the respository, then enter with `cd clipboard-manager`. 


### 🦀 Rust toolchain
```
curl https://sh.rustup.rs -sSf | sh
```

then verify rust and cargo installations with `rustc --version` and `cargo --version`.

### Node.js and npm
Verify node and npm installed with `node -v` and `npm -v`, if not install for your OS at [nodejs.org](https://nodejs.org/).




### Tauri Pre-reqs
| Platform | Commands |
| ------- | -------- |
| macOS | `brew install cmake libiconv` |
| Windows | Install Microsoft Visual Studio Build Tools (with Desktop Development with C++) |
| Linux | `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev libgtk-3-dev`

Then we need to check that dependencies are correct:
```
    cd src-tauri
    cargo check
```

### 🏃‍♂️ Running the script
Navigate to root directory, /clipboard-manager and run the following:
```
npm install --save-dev @tauri-apps/cli
npm install @tauri-apps/api
```

Then you should be able to run the app using:
```
npm run tauri dev
```

**If this doesn't work, then copy/paste your errors into GPT and give it context, then pray for the best 🙏**
And please update README if you find it's missing something!


## 🛠️ Updates Needed:
- Complete UI Overhaul
- Allow copying images as well
- Paraphrase long texts with ... to shorten
