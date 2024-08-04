# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project doesn't adhere to Semantic Versioning, see [Versioning](./README.md#versioning)

For changes in `@project-gauntlet/tools` see [separate CHANGELOG.md](https://github.com/project-gauntlet/tools/blob/main/CHANGELOG.md)

## [Unreleased]

## [6] - 2024-08-04

### Big things
- Wayland support
  - Requires LayerShell protocol `zwlr_layer_shell_v1` to be supported by window manager
    - `zwlr_layer_shell_v1` is implemented by most window managers with the exception of Gnome 
  - Global Shortcut on Wayland is not yet implemented, please use `gauntlet open` CLI command to open main window 
- Style changes
  - Settings UI overhaul
  - Slightly tweaked colors
  - Changed text selection color to make text more visible
  - Other minor style changes
- Theming support
  - 2 types of themes
    - Color only
    - Everything: Color, paddings, borders, etc
  - Theming is only applied to main window
  - Versioned
    - Because of internally invasive nature of the themes, it is perpetually unstable feature.
Themes are versioned and only one version is supported at the same time by application, the theme will stop working as soon as version is incremented
      - This may change in the future
  - See [THEME.md](./docs/THEME.md) for details
- It is now possible to change global shortcut for opening main window in settings
- Plugin shortcuts are now matched based on physical keycode, which means shortcuts now work regardless of selected keyboard layout
- It is now possible to show/hide action panel using <kbd>ALT</kbd> + <kbd>K</kbd> on Windows and Linux or <kbd>OPT</kbd> + <kbd>K</kbd> on macOS
  - At the moment it is not possible to change this shortcut to different keys
- **BREAKING CHANGE**: Implemented actual validation of `supported_system` plugin manifest property.
  - If OS was not specified in `supported_system` plugin manifest property, plugin will stop loading on that OS unless plugin is cross-platform (it doesn't have `environment`, `ffi`, `fs_read_access`, `fs_write_access`, `run_subprocess` or `system` permission) 

### Plugin Development
- Local plugin stdout `console.log` and stderr `console.error` logs are now saved to file to be able to show them in tools dev server CLI
- Added `windows` and `macos` values to plugins manifest `supported_system` property
- **BREAKING CHANGE**: For local development plugins, `dist` suffix is now automatically added to plugin ID. Please update `@project-gauntlet/tools` to `0.6.0`
- **BREAKING CHANGE**: Image source properties how accept path to asset, url or icon instead of byte array
  - Url source at the moment is not cached, so the image is downloaded every time the view is opened

### `Applications` plugin
- On Linux `.desktop` files are now opened using `gtk-launch` instead of custom implementation, fixing several edge cases

### UI Improvements and Fixes
- Main window list is now scrolled automatically when using keyboard navigation to keep focused item visible
- Better feedback about in-progress/failed/successful plugin downloads with information about errors in settings

### General fixes
- Fix plugin toggle button to hide/show entrypoints in settings not doing anything
- Fix settings app being always on top of other windows on macOS

### Internal Improvements
- Removed the need to have `42321` TCP port
  - Note: port `42320` is still being used

## [5] - 2024-07-07

### Big things
- Style overhaul
- Window no longer changes the size in subviews. Default window size is a little bigger
- **BREAKING CHANGE**: Rename `management` cli option to `settings`
- Add error message for settings view when unable to connect to server

### Fixes
- Fix Gauntlet crash on startup on Linux
- Built-in plugins no longer re-enable themselves after restart

## [4] - 2024-06-20

### Fixes
- Fix Gauntlet crash on startup on macOS 
- Fix Gauntlet crash on startup if there is no enabled plugins

## [3] - 2024-06-16

### Big things
- macOS support
  - `.dmg` file is now available
  - Implement Applications Plugin on macOS
  - Implement auto-launch on startup
- Search improvements
  - Use Frecency-based sorting
  - Use substring matching instead of prefix matching
  - Multiple (space-separated) word queries now return intersection of results instead of union
- Startup was reworked again
  - **BREAKING CHANGE**: Server is now started with plain `gauntlet` command instead of `gauntlet server` CLI option
  - When executing the binary, if server is already running, GUI will be opened
  - Added `--minimized` flag to start server without opening the GUI
  - Frontend is now in the same process as server
- Main view search results are now refreshed if the plugin changed search index while the view was opened

### Work-in-progress Windows support
- Fix build on Windows
- Implemented auto-launch on Windows
- Use <kbd>ALT</kbd> + <kbd>SPACE</kbd> global shortcut on Windows to open main window

### APIs changes
- Implemented Clipboard API
  - `text/plain` and `image/png` are supported types of clipboards
- Remove `Detail` children component of `Grid` from TypeScript typings. It never worked

### UI Improvements and Fixes
- Slightly better styling of `EmptyView` component
- Remove vertical separator line in Details view when there is no Content

### General fixes
- Fix panic when global shortcut functionality is not available
- Fix React warning about required `key` property when `ActionPanel` component is used
- Fix useEffect destructors not being called when view is closed

### Internal Improvements
- Preparation for documentation web page
  - Screenshot generation tool to generate screenshots for showing how UI looks for specific TS code
- Preparation for theming support

## [2] - 2024-04-30

### Big changes

- **BREAKING CHANGE**: Instead of separate binary for "frontend", frontend is now started by server in separate process on initial application launch
  - Global shortcut is available to show the application window. Currently, hardcoded to <kbd>META</kbd> + <kbd>SPACE</kbd>
  - In case window manager or underlying library we use doesn't support global shortcuts, `gauntlet open` command is also now available 
  - Unfortunately had to remove Wayland support for now, because GUI library we use, on Wayland doesn't support hiding/showing windows as well as setting window to be top-level
- **BREAKING CHANGE**: Inter-process communication protocol has been changed from `DBus` to `gRPC`
  - To be able to support macOS and Windows easier in near future
  - Currently, 2 TCP ports are used and hardcoded to be `42320` and `42321`
- **BREAKING CHANGE**: Plugins and plugin entrypoints now require description to be specified in plugin manifest
  - `gauntlet.description` and `entrypoint.*.description` keys are now required
  - Currently, only shown in Settings
- Actions can now be executed also using shortcuts
  - Shortcuts for specific entrypoint need to be specified in plugin manifest
  - Plugins are allowed to use only limited set of keys for shortcuts
    - Only upper and lower-case letters, symbols and numbers
  - Shortcut can have either `"main"` or `"alternative"` kind
    - `"main"` shortcut requires following modifiers
      - Windows and Linux: <kbd>CTRL</kbd>
      - macOS: <kbd>CMD</kbd>
    - `"alternative"` shortcut requires following modifiers
      - Windows and Linux: <kbd>ALT</kbd>
      - macOS: <kbd>OPT</kbd>
    - Whether <kbd>SHIFT</kbd> is also required depends on character specified for shortcut, e.g `$` will require <kbd>SHIFT</kbd> to be pressed, while `4` will not  
- 3 new bundled plugins
  - `Applications`
    - Collects and shows list of applications from systems as `"command"` entrypoints
    - "Gauntlet Application Launcher can now launch applications" :) 
    - Currently, only works on Linux
      - Collects `.desktop` files based on [Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html)
  - `Calculator`
    - Shows result of mathematical operations directly in main view under the search bar
    - Powered by [Numbat](https://github.com/sharkdp/numbat)
  - `Settings`
    - Allows to open Gauntlet Settings from the Gauntlet itself 
- Production React bundle is now used instead of Development bundle
  - Better performance
  - Worse error reporting
- Main window now has keyboard navigation
- It is now possible to remove plugins in Settings
- Plugin Entrypoints can now have icons
- Linux binary is now published inside `.tar.gz` file 

### API changes
- **BREAKING CHANGE**: `"command"` entrypoint now expects function as default export instead of plain JS file
- **BREAKING CHANGE**: Removed `<Content.Link>` React Component
  - `<Content>` is now expected to only contain non-interactive elements because of usage in `<Inline>` and `<Grid.Item>`
- **BREAKING CHANGE**: `<MetadataIcon>` `icon` property type was changed to enum and renders to proper image of limited set of icons
- New `<List>` and `<Grid>` top-level React Components
- New `"inline-view"` entrypoint type
  - A view is rendered directly under main view search bar
  - Default export is expected to be a React Component that takes `text` string with current value of main search bar as single property
  - Child component is expected to be of type `<Inline>`
  - Used to implement built-in `Calculator` plugin but also available for any plugins to use
- New `"command-generator"` entrypoint type
  - Allows plugins to provide dynamically generated list of `"command"` entrypoints 
  - Used to implement built-in `Applications` plugin but also available for any plugins to use
- New `useNavigation` React Hook. Simple stack based navigation in plugins
- Plugins and plugin entrypoints can now have preferences
  - Need to be declared in plugin manifest
  - Values can be set either on first plugin/entrypoint usage (if preference is required) or in Settings
  - `pluginPreferences` and `entrypointPreferences` helper functions to retrieve values
- Plugins now have ability to provide assets in `assets` directory which later can be retrieved in JS using new `assetData` helper function
- Add `"title"` property to `Checkbox` React Component
  - It is shown to the right of the checkbox in forms
- `<Image>` is now properly implemented and has `source` prop of type `ImageSource` which is object that contains image binary data 

### UI/UX Improvements and Fixes
- Application window is now always show on top of all other windows
- Application now requests focus when the window is opened
- Application window is now re-centered when opening any plugin-rendered view 
  - Needed because plugin-rendered views make window bigger causing it to be not centered
- Show error in GUI when React rendering fails either because of plugin or Gauntlet itself
- Text now flows and wraps properly if multiple separate strings were passed as children parameter to `<Paragraph>` React Component 
- Entrypoint name is now shown at the bottom of every plugin-rendered view
- Clicking on `<Detail.Metadata.Link>` now properly opens links in your default browser 
- Plugin name in main view has been moved to the right side of the window

## [1] - 2024-02-03

### Added

- Initial release.