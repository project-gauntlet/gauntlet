# Gauntlet

<img align="right" width="100" height="100" src="docs/logo.png">

~~Web-first~~ (not yet) ~~cross-platform~~ (not yet) application launcher with React-based plugins.

> [!NOTE]
> This is an MVP, expect bugs, missing features, incomplete ux, etc.
>
> If you want to help, please do, if not, check up on the project every once in a while to see the progress.
> But at the moment it is not yet ready for daily usage.
>
> There will probably be breaking changes which will be documented in [changelog](CHANGELOG.md).

![](docs/readme_demo.png)

## Features

- Plugin-first
    - Plugins can create UI or one-shot commands
    - It is just a framework for plugins
    - No plugins are provided by default
    - Plugins are distributed as separate branch in git repository (similar to how GitHub Pages work)
    - Plugins are installed using Git Repository URL
- [React](https://github.com/facebook/react)-based UI for plugins
    - Implemented using custom React Reconciler
    - iced-rs is used for UI
    - It is even possible to have multiple frontend implementations
- [Deno JavaScript Runtime](https://github.com/denoland/deno)
    - Deno allows us to sandbox JavaScript code for better security
    - Plugins are required to explicitly specify what permissions they need to work
- Plugins are distributed as part of Git repository
  - Use URL of Git repository to install plugin
- Client-Server architecture
    - All plugins run on server and render UI to a separate client process
    - On Linux, DBus is used for inter-proces communication
- Designed with cross-platform in mind
    - Permissions
        - If plugin asked for access to filesystem, env variables, FFI or running commands, it is required to specify
          which operating systems it supports.
        - If plugin doesn't use filesystem, env variables, ffi or running commands and just uses network and/or UI, it
          is cross-platform
    - Keybinds (not yet implemented)
        - Keybind is "primary" or "secondary" + key
        - Meaning of "primary" and "secondary" depends on operating system that plugin is running on.
            - E.g. for Linux "primary" is <kbd>Ctrl</kbd>, "secondary" - <kbd>Alt</kbd>
            - But for macOS "primary" is <kbd>Cmd</kbd>, "secondary" - <kbd>Opt</kbd>

##### OS Support

###### Implemented

- <img src="https://cdn.jsdelivr.net/gh/simple-icons/simple-icons@develop/icons/linux.svg" width="18" height="18" /> Linux

###### Planned

- <img src="https://cdn.jsdelivr.net/gh/simple-icons/simple-icons@develop/icons/windows.svg" width="18" height="18" /> Windows
- <img src="https://cdn.jsdelivr.net/gh/simple-icons/simple-icons@develop/icons/apple.svg" width="18" height="18" /> macOS

##### UI

###### Implemented

- Detail
- Form
- Action Panel
- Separate settings view

###### Planned

- View router
- List
- Grid
- Toast popups
- Keyboard only navigation
- Theming
- Themable icons
- Vim motions

##### APIs

###### Planned

- Clipboard
- Preferences
- Local Storage
- OAuth PKCE flow support
- Search bar parsing

## Getting Started

### Create your own plugin

- Go to [plugin-template](https://github.com/project-gauntlet/plugin-template) and create your own GitHub repo from it.
- Run `npm run dev` to start dev server (requires running application server)
    - Dev server will automatically refresh the plugin on any file change
- Do the changes you need
    - You can configure plugin using [Plugin manifest](#plugin-manifest)
    - Documentation is, at the moment, basically non-existent but TypeScript declarations in `@project-gauntlet/api`
      and `@project-gauntlet/deno` should help
- Push changes to GitHub
- Run `publish` GitHub Actions workflow to [publish plugin to release branch]()
- Profit!

### Install plugin

Plugins are installed in Settings UI. Use Git repository name of the plugin to install it. 

![](docs/settings_ui.png)

### Install application

> [!NOTE]
> At the moment application is not published anywhere,
> so you have to download it from the [GitHub Releases](https://github.com/project-gauntlet/gauntlet/releases)

Be the first one to create a package. See [Application packaging for Linux](#application-packaging-for-Linux)

## Configuration

### Plugin manifest

```toml
[gauntlet]
name = 'Plugin Name'

[[entrypoint]]
id = 'ui-view' # id for entrypoint
name = 'UI view' # name of entrypoint
path = 'src/ui-view.tsx' # path to file, default export is expected to be function React Function Component
type = 'view'

[[entrypoint]]
id = 'command-a' 
name = 'Command A'
path = 'src/command-a.ts' # path to file, the whole file is a js script
type = 'command'

[permissions] # For allowed values see: https://docs.deno.com/runtime/manual/basics/permissions
environment = ["ENV_VAR_NAME"] # array of strings, if specified requires supported_system to be specified as well
high_resolution_time = false # boolean
network = ["github.com"] # array of strings
ffi = ["path/to/dynamic/lib"] # array of strings, if specified requires supported_system to be specified as well
fs_read_access = ["path/to/something"] # array of strings, if specified requires supported_system to be specified as well
fs_write_access = ["path/to/something"] # array of strings, if specified requires supported_system to be specified as well
run_subprocess = ["program"] # array of strings, if specified requires supported_system to be specified as well
system = ["apiName"] # array of strings, if specified requires supported_system to be specified as well

[[supported_system]]
os = 'linux' # currently only 'linux'

```

### Application config

Located at `$XDG_CONFIG_HOME/gauntlet/config.toml` for Linux. Not used at the moment.

## CLI

### Application

The Application has a simple command line interface

- `gauntlet server` - server part of launcher
- `gauntlet client` - gui part of launcher
- `gauntlet management` - settings, plugin installation, etc

### Dev Tools

[`@project-gauntlet/tools`](https://www.npmjs.com/package/@project-gauntlet/tools) contains separate CLI tool for plugin
development purposes. It has following commands:

- `gauntlet dev`
    - Starts development server which will automatically refreshed plugin on any file change.
- `gauntlet build`
    - Builds plugin
- `gauntlet publish`
    - Publishes plugin to separate git branch. Includes `build`
    - `publish` assumes some things about git repository, so it is recommended to publish plugin from GitHub Actions
      workflow

[Plugin template](https://github.com/project-gauntlet/plugin-template) has nice `npm run` wrappers for them.

## Architecture

The Application consists of three parts: server, frontend and settings.
Server is an application that is registered on session dbus with a well-known name and exposes 2 dbus interfaces:
one for frontend, other for settings application.
All plugins run on server.
Each plugin in its own sandboxed Deno Worker.
In plugin manifest it is possible to configure permissions which will allow plugin to have access to filesystem,
network, environment variables, ffi or subprocess execution.
Server saves plugins and state of plugins into SQLite database.

Frontend is GUI application that uses [iced-rs](https://github.com/iced-rs/iced) as a GUI framework.
It is also registered on session dbus with well-known name and exposes one dbus interface for server to call.
From the perspective of dbus, it can also be considered a "server" because it answers requests that come from server.

Plugins can create UI using [React](https://github.com/facebook/react).
Server implements custom React Reconciler (similar to React Native)
and renders to frontend running as a separate process.
Server listens on signals from frontend, so when user opens view defined by plugin, frontend sends an open-view signal.
Server then receives the signal, runs React render and React Reconciler
makes requests to the frontend containing information what actually should be rendered.
When a user interacts with the UI by clicking button or entering text into form,
frontend sends signals to server to see whether any re-renders are needed.

Settings is also a GUI application that communicates with server via DBus using a simple request-response approach.

<div hidden>
https://www.planttext.com/

```
@startuml

== Frontend: Start ==

Frontend -> Server: list of views and commands request
Server --> Frontend: list of views and commands response

== Frontend: Initial View Render ==

Frontend --> Server: open-view signal
Server -> Frontend: render components request
Frontend --> Server: render components response

== Frontend: Command Execution ==

Frontend --> Server: execute command signal

== Frontend: View Update On Event ==

Frontend --> Server: button click, key press in input component, etc
Server -> Frontend: render components request
Frontend --> Server: render components response

== Settings ==

Settings -> Server: request
Server --> Settings: response

@enduml
```

</div>

![](docs/architecture.png)

Plugins (or rather its compiled state) are distributed via Git repository in `release` branch (similar to GitHub Pages).
Which means there is no one central place for plugin distribution.
And to install plugin all you need is Git repository url.

Application defines set of React components to use for plugins.
Creating and validating components involves some boilerplate.
Component model was created for help manage is.
It is essentially a json file which defines what components exist, what properties and event handler they have.
This file is then used
to generate TypeScript typings for `@project-gauntlet/api` and Rust validation code for server and frontend.

## Application packaging for Linux

This section contains a list of things
that could be useful to someone who wants to package application for Linux distribution.
If something is missing, please [create an issue](https://github.com/project-gauntlet/gauntlet/issues).

Gauntlet executable consists of three applications:

- `$ path/to/gauntlet/executable server`
    - Needs to be started when user logs in
- `$ path/to/gauntlet/executable client`
    - Started on demand using <kbd>Super</kbd> key (or any other key/shortcut depending on your preference)
- `$ path/to/gauntlet/executable management`
    - Started on demand from the list of available applications (will vary depending on desktop environment or windows
      manager chosen)

Client and Settings applications expect Server to always be running.
Recommended way of ensuring that is running Server as Systemd service.
Communication is done via DBus session bus.

###### Directories used

- data dir - `$XDG_DATA_HOME/gauntlet` or `$HOME/.local/share/gauntlet`
    - contains application state `data.db`
- config dir - `$XDG_CONFIG_HOME/gauntlet` or `$HOME/.config/gauntlet`
    - contains application config `config.toml`
    - application will never do changes to config file

Application and Dev Tools use temporary directories:

- Rust: [tempfile crate](https://crates.io/crates/tempfile)
- JS: [NodeJS mkdtemp](https://nodejs.org/api/fs.html#fspromisesmkdtempprefix-options)

Client and Setting applications have GUI and therefore use all the usual graphics-related stuff from Wayland or X11.
For Wayland currently no special protocols are required, but it may change in the future.

## Versioning

### Application

Application uses simple incremental integers starting from `1`.
It doesn't follow the SemVer versioning.
Given application's reliance on plugins, once it is stable,
introducing breaking changes will be done carefully (if at all) and will be given a reasonable grace period to migrate.
SemVer is about a hard cutoff between major versions with breaking changes, which doesn't fit this kind of application.
Before application is declared stable, breaking changes could be done without a grace period.

### Tools

[`@project-gauntlet/tools`](https://www.npmjs.com/package/@project-gauntlet/tools) uses SemVer.

### Plugins

Plugins only have the latest published "version". 
