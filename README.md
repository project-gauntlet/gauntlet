# Gauntlet

[![Discord](https://discord.com/api/guilds/1205606511603359785/widget.png?style=shield)](https://discord.gg/gFTqYUkBrW)

<img align="right" width="100" height="100" src="assets/linux/icon_256.png">

Web-first cross-platform application launcher with React-based plugins

> [!WARNING]
> Launcher is being developed by single developer in their free time.
> Changes may be slow but steady
>
> There will probably be breaking changes which will be documented in [changelog](CHANGELOG.md).

![image](https://github.com/user-attachments/assets/81339462-9cc3-469e-8cdc-ca74918bceab)

## Demo

Slightly outdated demo

https://github.com/user-attachments/assets/19964ed6-9cd9-48d4-9835-6be04de14b66

## Features

- Plugin-first
  - Plugins are written in TypeScript
  - Extensive plugin API 
      - Create UI views
      - One-shot commands
      - Dynamically provide list of one-shot commands
      - Render quick "inline" content directly under main search bar based on value in it
      - Get content from and add to Clipboard
  - Plugins are distributed as separate branch in Git repository, meaning plugin distribution doesn't need any central
    server
  - Plugins IDs are just Git Repository URLs
  - [React](https://github.com/facebook/react)-based UI for plugins
    - Implemented using custom React Reconciler (no Electron)
  - [Deno JavaScript Runtime](https://github.com/denoland/deno)
    - Deno allows to sandbox JavaScript plugin code for better security
    - Plugins are required to explicitly specify what permissions they need to work
    - Node.js is used to run plugin tooling, but as a plugin developer you will always write code that runs on Deno
- Designed with cross-platform in mind from the beginning
- Commands and Views can be run/opened using custom global shortcuts
- Custom search alias can be assigned to Commands or Views
- Custom theme support
- Built-in functionality is provided by bundled plugins
  - Applications: shows applications installed on the system in search results
    - Plugin also tracks windows and which application they belong to, so opening already opened application will by default bring up previously created window
      - Not all systems are supported at the moment. See [feature support](https://gauntlet.sh/docs/feature-support)
  - Calculator: shows result of mathematical operations directly under main search bar
    - Includes converting currency using exchange rates
    - Powered by [Numbat](https://github.com/sharkdp/numbat)
- Frecency-based search result ordering
   - Frecency is a combination of frequency and recency
   - More often the item is used the higher in the result list it will be, but items used a lot in the past will be ranked lower than items used the same amount of times recently
   - Results are matched per word by substring

##### OS Support

##### Official
- <img src="https://cdn.jsdelivr.net/gh/simple-icons/simple-icons@develop/icons/linux.svg" width="18" height="18" /> Linux X11
- <img src="https://cdn.jsdelivr.net/gh/simple-icons/simple-icons@develop/icons/apple.svg" width="18" height="18" /> macOS M1

##### Best-effort
- <img src="https://cdn.jsdelivr.net/gh/simple-icons/simple-icons@develop/icons/linux.svg" width="18" height="18" /> Linux Wayland
  - Gnome Wayland is not yet supported, see [#40](https://github.com/project-gauntlet/gauntlet/issues/40)
- <img src="https://img.icons8.com/windows/32/windows-11.png" width="18" height="18" /> Windows
- <img src="https://cdn.jsdelivr.net/gh/simple-icons/simple-icons@develop/icons/apple.svg" width="18" height="18" /> macOS Intel

## Getting Started

### Install Gauntlet

See [Installation](https://gauntlet.sh/docs/installation)

### Global Shortcut

Main window can be opened using global shortcut or CLI command:
- Global Shortcut (can be changed in Settings)
  - Windows: <kbd>ALT</kbd> + <kbd>Space</kbd>
  - Linux X11: <kbd>Super</kbd> + <kbd>Space</kbd>
  - Linux Wayland
    - Global shortcut may not be supported, see [feature support](https://gauntlet.sh/docs/feature-support)
    - Please use CLI command instead, and invoke it using window manager specific approach
  - macOS: <kbd>CMD</kbd> + <kbd>Space</kbd>
- CLI command
  - `gauntlet open`

### Install plugin

Plugins are installed in Settings UI. Use Git repository url of the plugin to install it, e.g. `https://github.com/project-gauntlet/readme-demo-plugin.git`

![](docs/settings_ui.png)

### Create your own plugin

See [Getting started with plugin development](https://gauntlet.sh/docs/plugin-development/getting-started)

## Theming

See [Theming](https://gauntlet.sh/docs/theming)

## Building Gauntlet

You will need:
- NodeJS
- Rust
- Protobuf Compiler
- CMake (not used by the project itself, but is required by a dependency)
- On Linux: `libxkbcommon-dev` (note: name may differ depending on used distribution)

### Dev

To build dev run:
```bash
npm ci
npm run build
npm run build-dev-plugin
cargo build
```
In dev (without "release" feature) application will use directories ONLY inside project directory to store state or cache, to avoid messing up global installation

### Not-yet-packaged

To build not-yet-packaged release binary, run:
```bash
npm ci
npm run build
cargo build --release --features release
```

### Packaged
To build os-specific package, run one of the following:

macOS:
```bash
npm run build-macos-project --workspace @project-gauntlet/build
```

Windows:
```bash
npm run build-windows-project --workspace @project-gauntlet/build
```

Linux:
```bash
npm run build-linux-project --workspace @project-gauntlet/build
```

But the new version release needs to be done via GitHub Actions

## Contributing

If you'd like to help build Gauntlet you can do it in more ways than just contributing code:
- Reporting a bug or UI/UX problem
- Creating a plugin

For simple problems feel free to open an issue or PR and tackle it yourself. 
For more significant changes please contact creators on Discord (invite link on top of README) and discuss first.

All and any contributions are welcome.

