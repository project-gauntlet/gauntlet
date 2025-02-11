# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project doesn't adhere to Semantic Versioning, see [Versioning](./README.md#versioning)

For changes in `@project-gauntlet/tools` see [separate CHANGELOG.md](https://github.com/project-gauntlet/tools/blob/main/CHANGELOG.md)

## [Unreleased]

- When using active screen setting for window positioning, position calculated is now relative to size of the screen. Fixes unexpected position when using monitors of different size (contributed by @BennoCrafter)
- Add shortcut to open Settings UI (contributed by @BennoCrafter)
  - <kbd>Ctrl</kbd> + <kbd>,</kbd> on Windows and Linux
  - <kbd>Cmd</kbd> + <kbd>,</kbd> on macOS
- Global shortcut how hides the main window if it is already open (contributed by @BennoCrafter)
- It is now possible to run commands and open views using CLI command
  - Format: `gauntlet run <plugin-id> <entrypoint-id> <action-id>`
  - Plugin ID can be found in Settings UI
  - Entrypoint ID can be found in:
    - For entrypoint types `command` and `view` - in Plugin Manifest or in Settings UI TODO
    - For entrypoint type `entrypoint-generator` - in Settings UI TODO 
  - Action ID can also be found in Plugin Manifest
  - Action ID option also accepts special values
    - `:primary` - to run primary action of the entrypoint
    - `:secondary` - to run secondary action of the entrypoint
- Slightly improved --help documentation of CLI command

- In main window search result, moved plugin name next to entrypoint name
- In main window search result, displayed type of entrypoint in place of plugin name, use generator entrypoint name if generated

- Fix no plugins starting on Windows in release mode 
- Fix all global shortcut registrations failing if one shortcut registration failed
- Fix error when registering shortcut erroring whole settings window instead of adding an icon

## [14] - 2025-01-19

- Fixed mouse actions like scrolling or clicking not working on macOS

## [13] - 2025-01-19

### General
- Window Tracking
  - Gauntlet now tracks opened windows and assigns them to specific application entry in results
  - If application has window open, primary action now instead focuses the window
  - If there are multiple windows open, primary action opens view which contains list of windows that can be focused
  - If application has window open, it is still possible to open new application instance by using separate new action
  - It is experimental, and it is possible to disable window tracking by unchecking checkbox in Application entrypoint preferences in Settings UI  
  - Currently supported on
    - Linux X11
    - wlroots-based window managers
    - Hyprland
    - Cosmic
- Added "Show all opened windows" entrypoint to bundled plugin
- Application plugin is now implemented on Windows 
- macOS native-like dark and light mode themes are now available
- On macOS theme is now auto-selected based on system theme
- On macOS window can now be dragged to change its position
  - Window position is saved and will be used after restart
- Binary size has been reduced by around 40% (contributed by @davfsa)
- Added `main_window.close_on_unfocus` boolean option to config file to disable "close on unfocus" functionality of main window
  - Intended to be used when using "window focus follows mouse" functionality of OS, Desktop Environment or Window Manager 
- Added option in Settings UI to choose were main window appears when opening it
  - Current options
    - `Static`
      - Window always opens in the same location (on macOS location can be changed by dragging the window)
    - `Active Monitor`
      - Windows opens on monitor which has currently focused window
  - Currently supported only on macOS
- Improve config and theme config error parsing logs

### Theming
- Themes have been reworked
  - Removed complex themes
  - Removed theme versioning
  - Removed sample generation commands
  - Themes are now defined in TOML format
    - Theme file is located in config directory (varies based on OS) with name `theme.toml`
  - Format of theme file has been reworked, see bundled themes for examples
- 3 bundled themes are now available: [Bundled themes](./bundled_themes)
  - Legacy (previous default theme)
  - macOS Light
  - macOS Dark
- It is possible to change theme in Settings UI
  - By default, theme is auto-detected to use one of the bundled ones
  - Setting is locked if theme config file exists

### Plugins
- Entrypoint Generator improvements
  - **BREAKING CHANGE**: Renamed `"command-generator"` entrypoint type into `"entrypoint-generator"`, as well as all types related to it 
  - **BREAKING CHANGE**: Removed `GeneratedEntrypoint`'s `fn: () => void`
    - `actions: GeneratedEntrypointAction[]` field now is required to have at least one element
    - It is now possible to specify label displayed on bottom row panel for primary action
  - **BREAKING CHANGE**: Renamed `GeneratedEntrypointAction`'s `fn` field into `run`
  - It is now possible to have `GeneratedEntrypointAction` which opens view instead of running command by specifying `view` field with value of React `FC` type instead of `run`
  - Renamed `GeneratorProps` to `GeneratorContext`
  - Added `pluginPreferences` and `entrypointPreferences` properties to `GeneratorContext` to access preferences from Entrypoint Generator 
  - Added `get: (id: string) => GeneratedEntrypoint | undefined` function to `GeneratorContext` to get added entrypoint
  - Added `getAll: () => GeneratedEntrypoint[]` function to `GeneratorContext` to get all added entrypoints
  - Generated Entrypoints can now have accessories similar to `<List/>` component
- Removed `pluginPreferences` and `entrypointPreferences` helper functions
- Added `usePluginPreferences` and `useEntrypointPreferences` React Hooks 
- Command function now receives `CommandContext` as first argument
  - Object contains `pluginPreferences` and `entrypointPreferences` properties to access preferences from Command 
- Unified primary and secondary action execution in `<List.Item/>` and `<Grid.Item/>` 
  - **BREAKING CHANGE**: Removed `onClick` property on `<List.Item/>` and `<Grid.Item/>` components
  - **BREAKING CHANGE**: `<List.Item/>` and `<Grid.Item/>` now has how `id: string` required property 
  - If primary or secondary action is executed when `<List.Item/>` and `<Grid.Item/>` is focused, `onAction` handler first parameter will be value of `id` prop of focused item 
- Added `onItemFocusChange?: (itemId: string | undefined) => void` property on `<List.Item/>` and `<Grid.Item/>`. Function is called when focused item changes 
- **BREAKING CHANGE**: Renamed `Image` type to `ImageLike` to avoid conflict with `<Image/>` component
- When entrypoint is enabled/disabled or preference value is changed whole plugin runtime is restarted instead of just reloading the search index
- It is now possible to control whether the action closes main window by returning `{ close: true }` object from `onAction` property function
  - For `<Inline/>` view and commands (including generated commands) action always closes window without possibility to keep it open
- Improved rejected promise error log

### UI/UX Improvements
- On macOS main window now uses native window decorations
- Show name of generator entrypoint near plugin name of entrypoints generated by it in main view search results
- Improved styling of action panel popup
  - Tweaked padding between sections
  - Added shadow around it
- Tweak height of `<List.Detail.Metadata/>` to be slightly taller
- Values of fields in `<List.Detail.Metadata/>` are now positioned on the same row as labels 

### Fixes
- Fixed one thread having close to 100% CPU usage while main window is hidden
- Fixed icons in main search view sometimes not loading when window is opened or disappearing after scrolling
- Fixed commands in `permissions.exec.command` in Plugin Manifest not being resolved properly
- Fixed zombie processes being left over after plugin runtime is stopped
- Fixed `npm run dev` failing to reload in some cases
- Fixed `<Grid.EmptyView/>` not displaying the image
- Fixed image in `<List.EmptyView/>` being too big so labels are not shown
- Fixed action not being run if `<List/>` or `<Grid/>` view has focused `<SearchBar/>`
- Fixed `npm run dev` failing because of missing log files when run for the first time

## [12] - 2024-12-22

### General
- Each plugin Deno runtime now runs in separate OS process
- Nix package, dev shell and home-manager modules are now available (contributed by @schradert)
- Replaced wayland layer-shell implementation, fixing several long-standing issues
  - Fixed icons not being rendered properly
  - Fixed <kbd>Ctrl</kbd> + <kbd>A</kbd> shortcut not working in text inputs
  - Fixed <kbd>Backspace</kbd> only removing single character at a time
- Improved window handling on macOS
  - Main window is now non-activating, so it doesn't take away focus from front-most application 
  - Main window no longer shows up in macOS Dock panel
  - Fixed application not receiving keyboard input if front-most application is using Secure Event Input (e.g. Terminal)
- Main window is now closed automatically when plugin action is executed
- A lot of internal dependency updates

### Plugin API
- Internal JS functions are no longer accessible from plugins
- **BREAKING CHANGE**: Deno updated from `v1.37.0` to `v2.1.1`
- **BREAKING CHANGE**: Clipboard api now uses `ArrayBuffer` instead of `Blob`
- `@project-gauntlet/deno` is deprecated in favor of `@types/deno`

### UI/UX Improvements
- Hud window was moved lower (below main window) on non-wayland platforms

### Fixes
- Fixed possible freeze when spamming keys or buttons in plugin view
- Fixed separator in `<Inline/>` view not being horizontally centered

## [11] - 2024-11-16

### General
- Primary action label on bottom bar is now a clickable button
- It is now possible to unset global shortcut in settings
- Implemented keyboard navigation support for `<List/>` and `<Grid/>`
  - Note: `<Grid/>` scrolling while using keyboard navigation is still quite buggy and is work in progress

### Bundled plugin
#### `Applications`
- Applications commands are now automatically added or removed when application is installed or uninstalled respectively
- When loading the list of applications from OS, loading bar and "Indexing..." text in bottom panel is shown in main window

### Plugin API
- Added `<SearchBar/>` component in `<List/>` and `<Grid/>` which is text input field above content of the respective view
- `"command-generator"` entrypoints have been reworked
  - Now it is possible to update list of generated entrypoints (add or remove) after the main command generator function has finished running
  - **BREAKING CHANGE**: Command Generator entrypoint function now accepts an object with `add: (id: string, data: GeneratedCommand) => void` and `remove: (id: string) => void` functions
  - **BREAKING CHANGE**: Command Generator entrypoint function now should return nothing or a cleanup function e.g. close file watcher. Currently, it is called when disabling/enabling any of entrypoints in plugin, but it is not called when whole plugin is stopped
  - While generator function itself is running (given that the function is async) the loading bar and "Indexing..." text in bottom panel will be shown in main window 
- **BREAKING CHANGE**: Validation of React Component children is now a lot more strict with respect to what amount of children of specific type is allowed
- **BREAKING CHANGE**: `assetData` helper function renamed to `assetDataSync`
- Added async variant of `assetDataSync` helper function named `assetData` which returns `Promise<ArrayBuffer>`
- Added Plugin Environment API
  - `Environment.gauntletVersion` - `number`, current Gauntlet version
  - `Environment.isDevelopment` - `boolean`, `true` if plugin was added with `npm run dev` as opposed to Settings UI
  - `Environment.pluginCacheDir` - `string`, path to plugin cache directory, corresponds to `{common:plugin-cache}` variable in permissions
  - `Environment.pluginDataDir` - `string`, path to plugin data directory, corresponds to `{common:plugin-data}` variable in permissions

### Theming API
- Themes slightly reworked
  - "Color Theme" is renamed into "Simple Theme"
    - **BREAKING CHANGE**: Sample generation CLI command was changed to `gauntlet generate-sample-simple-theme`
    - **BREAKING CHANGE**: Theme file is renamed from `color_theme.json` to `simple-theme.json`
  - "Everything Theme" or just "Theme" is renamed into "Complex Theme"
    - **BREAKING CHANGE**: Sample generation CLI command was changed to `gauntlet generate-sample-complex-theme`
    - **BREAKING CHANGE**: Theme file is renamed from `theme.json` to `complex-theme.json`
  - It is now possible to customize color, width and radius of borders in Simple Theme 
- **BREAKING CHANGE**: Current Simple Theme version increased to `4`
- **BREAKING CHANGE**: Current Complex Theme version increased to `4`

### UI/UX Improvements
- Loading bar is now shown if opening plugin view takes more than 300 milliseconds
- Pressed button state now has distinct styling, providing more clear indication that button was pressed
- When registering global shortcut in settings fails, instead of showing error on whole settings screen now icon with on hover text is shown to the right of the setting field 
- If registering global shortcut on application startup fails, error is now also shown in settings 
- Fixed padding of grid and list section being too far down if it is first in the view
- Fixed incorrect supported schemas label in Settings UI, http(s), ssh and git are the only supported schemas for plugin IDs
- Better error for not supported plugin ID schemas
- `<Grid.Item/>` height is now dynamic and is based on `<Grid/>` or `<Grid.Section/>` `columns` property

### Fixes
- Fixed global shortcut not working on Windows
- Fixed emojis not working in a lot of places across the application
- Fixed hud window not disappearing on Wayland
- Fixed clipboard operations not working on KDE
  - Note: `Clipboard.clear()` or `Clipboard.writeText("")` is still not working on KDE due to upstream bug
- Fixed clipboard operations not working on Wayland
- Fix primary action of first search result being called if primary action of inline view is called using enter key
- Fix scrollable resetting when clicking action panel button in bottom panel

## [10] - 2024-10-13
### General
- Main view now has action bar and action panel 
  - Action bar displays current primary action depending on focused result item
  - <kbd>ALT</kbd> + <kbd>K</kbd> (<kbd>OPT</kbd> + <kbd>K</kbd> on macOS) is available to open action panel
  - Content of action panel can be defined by plugins
    - `"inline-view"` and `"command-generator"` entrypoint types can now specify custom actions on main view
    - Plugin can also provide shortcut that will be available depending on focused result item without opening the action panel 
- Primary and secondary actions
  - First action in action panel is now considered primary and can be run using <kbd>ENTER</kbd> without opening action panel
  - Second action in action panel is now considered secondary and can be run using <kbd>SHIFT</kbd> + <kbd>ENTER</kbd> without opening action panel
  - Works for all places that can define actions: `"inline-view"`, `"command-generator"` and `"view"` entrypoint types
- Action panel now supports keyboard navigation
- All bundled plugins are merged into one
- It is now possible to update plugin using "Check for updates" button in settings

### Bundled plugin
#### `Applications`
- Add Flatpak application support on Linux
- Fixed no applications being shown on macOS Sequoia (15)
- Fixed crash on macOS if macOS version only contains two segments, e.g. `15.0` vs `15.0.1`
- Fixed some applications not having icons on macOS

#### `Calculator`
- It is now possible to copy result of calculation using primary action and its shortcut
  - After copying, popup is shown to indicate that the result was copied
- Updated `numbat` dependency to [1.14.0](https://github.com/sharkdp/numbat/releases/tag/v1.14.0)
  - Notable change: "Add lowercase aliases for currency units"

### Plugin API
- Plugin permissions reworked
  - **BREAKING CHANGE**: Plugin manifest property `permissions.ffi` removed
    - FFI in Deno is an unstable feature
    - May be brought back in future
  - **BREAKING CHANGE**: Plugin manifest property `permissions.high_resolution_time` removed
    - This is done in preparation for Deno update, newer versions of which removed this permission
  - **BREAKING CHANGE**: Plugin manifest property `permissions.fs_read_access` renamed to `permissions.filesystem.read`
  - **BREAKING CHANGE**: Plugin manifest property `permissions.fs_write_access` renamed to `permissions.filesystem.write`
  - **BREAKING CHANGE**: Plugin manifest property `permissions.run_subprocess` has been split into 2 properties: `permissions.exec.command` and `permissions.exec.executable`
    - `command` is for commands on `PATH`, e.g. `"ls"`
    - `executable` is for absolute paths to binary, e.g. `"/usr/bin/ls"`
  - **BREAKING CHANGE**: Windows-style paths are not allowed in plugins that do not support Windows
  - **BREAKING CHANGE**: Unix-style paths are not allowed in plugins that do not support Linux or macOS
  - **BREAKING CHANGE**: Plugin manifest property `permissions.network` now can only contain domain and optionally port of URL
  - **BREAKING CHANGE**: Path permissions (`permissions.filesystem.read`, `permissions.filesystem.write` and `permissions.exec.executable`) now can only contain absolute paths
  - Path permissions (`permissions.filesystem.read`, `permissions.filesystem.write` and `permissions.exec.executable`) can now contain variables which will be replaced at plugin load time
    - Examples: `{linux:user-home}/.local/share`, `{common:plugin-cache}/my-plugin-cache`
    - Variables can only be used at the beginning of the path
    - List of currently available variables
      - `{macos:user-home}`
        - Resolves to `$HOME`, i.e. `/Users/<username>`
        - Only available if plugin supports macOS
      - `{linux:user-home}`
        - Resolves to `$HOME`, i.e. `/home/<username>`
        - Only available if plugin supports Linux
      - `{windows:user-home}`
        - Resolves to `{FOLDERID_Profile}`, i.e. `C:\Users\<username>`
        - Only available if plugin supports Windows
      - `{common:plugin-data}`
        - On Windows: `{FOLDERID_RoamingAppData}\Gauntlet\data\plugins\<plugin-uuid>`
        - On Linux: `$XDG_DATA_HOME/gauntlet/plugins/<plugin-uuid>`
        - On macOS: `$HOME/Library/Application Support/dev.project-gauntlet.gauntlet/plugins/<plugin-uuid>`
      - `{common:plugin-cache}`
        - On Windows:  `{FOLDERID_LocalAppData}\Gauntlet\cache\plugins\<plugin-uuid>`
        - On Linux:  `$XDG_CACHE_HOME/gauntlet/plugins/<plugin-uuid>`
        - On macOS:  `$HOME/Library/Application Support/dev.project-gauntlet.gauntlet/plugins/<plugin-uuid>`
- `<Grid.Item/>`'s `title` property is now optional
- `<Grid.Item/>` have a new `accessory` property, which provides an ability to specify text and/or icon under the grid cell
- `<List.Item/>` have a new `accessories` property, which provides an ability to specify one or multiple text and/or icon items on the right side of list item
- **BREAKING CHANGE**: `<Action>`'s `title` property renamed to `label`
- Added `entrypoint.icon` plugin manifest property that accepts path to image inside plugin's `assets` directory
- Added `showHud` function that will create a simple popup window with text provided to that function 

### Theming API
- **BREAKING CHANGE**: Current color theme version increased to `3`
- **BREAKING CHANGE**: Current everything theme version increased to `3`

### UI/UX Improvements
- Grid styling refined
- Inline view styling refined
- Plugin and entrypoint names of rendered inline view are now shown above that inline view
- Made color of text slightly more bright 
- Focused (by keyboard navigation) and hovered (by hovering with mouse) search items now have distinct styling
- Slightly increased size of icons in main search view
- Plugin ID is now shown in sidebar in settings when plugin is selected
- "Remove plugin" button has been moved to the bottom of the sidebar in settings
- In settings required preferences that do not have value provided or do not have default value are now highlighted  
- Names of keys of shortcuts were changed from all upper-case to first letter only upper-case 

### Fixes
- Fixed panic when trying to stop already stopped plugin
- Fixed crash on macOS if `openssl@v3` library is not installed
- Fixed inline view still being shown after main view was closed and reopened
- Fixed download info panel in settings sometimes going outside of window size and being cut off

## [9] - 2024-09-15

### Plugin API
- New React Hooks
  - `usePromise`
    - Helper to run promises in a context of React view
    - Returns `AsyncState` object which contains `isLoading`, `error` and `data` properties
  - `useStorage`
    - Helper to store data between entrypoint, plugin and application runs
    - Follows API similar to `useState` built-in React Hook
    - Uses `localStorage` internally
  - `useCache`
    - Helper to store data between entrypoint runs but will be reset when plugin or application is restarted 
    - Follows API similar to `useState` built-in React Hook
    - Uses `sessionStorage` internally
  - `useCachedPromise`
    - Helper to run promises with caching done automatically
    - Follows `stale-while-revalidate` caching strategy
    - Uses `usePromise` and `useCache` Hooks internally
  - `useFetch`
    - Helper to run `fetch()` with caching done automatically
    - Follows `stale-while-revalidate` caching strategy
    - Uses `useCachedPromise` Hook internally
- Add `isLoading` property on `<Detail/>`, `<Form/>`, `<Grid/>` and `<List/>`
  - If passed `true` the loading indicator will be shown above view content
- **BREAKING CHANGE**: To use `Clipboard` api, new permission `permissions.clipboard` is required to be specified in plugin manifest
  - `permissions.clipboard` manifest property accepts a list that can include one or multiple of `"read"`, `"write"` or `"clear"` values
- **BREAKING CHANGE**: To use plugin entrypoint of type `inline-view`, new permission `permissions.main_search_bar` is required to be specified in plugin manifest
  - `permissions.main_search_bar` manifest property accepts a list that can include `"read"` value
- **BREAKING CHANGE**: Plugin and Entrypoint Preference `name` properties in plugin manifest was split into 2 properties
  - `preferences.name` is split into `preferences.name` and `preferences.id` 
  - `entrypoint.preferences.name` is split into `entrypoint.preferences.name` and `entrypoint.preferences.id`
  - To preserve value set by user in settings please set the previous value of `name` to `id`
- **BREAKING CHANGE**: Replaced `onSelectionChange` and `id` properties on `<Grid/>` and `<List/>` with `onClick` on `<Grid.Item/>` and `<List.Item/>`

### UI/UX Improvements
- Added <kbd>ALT</kbd> + <kbd>K</kbd> (<kbd>OPT</kbd> + <kbd>K</kbd> on macOS) label to Action Panel button in bottom panel in plugin views
  - Refined styling to accommodate this change
  - **BREAKING CHANGE**: Current color theme version increased to `2` 
  - **BREAKING CHANGE**: Current everything theme version increased to `2` 

### `Applications` plugin
- Add macOS System settings items like Sound, Network, etc
  - Both pre- and post-Ventura macOS settings are supported
- Fixed macOS applications, that are nested more than one directory level deep in `Applications` directory, not being added 

### `Calculator` plugin
- Updated `numbat` dependency to [1.13.0](https://github.com/sharkdp/numbat/releases/tag/v1.13.0)
- Enabled currency exchange rate module

### Fixes
- Fix application crash when refreshing plugin via `npm run dev` from tools
- Fix plugin runtime shutting down when exception is thrown inside a promise handler

## [8] - 2024-09-07

### Plugin API
- Command Generator functions are no longer needlessly called after every entrypoint click in main window

### Fixes
- Fixed crash on Arch Linux if using AMD GPU with Vulkan not setup properly
  - Still if Vulkan is not setup properly it can result in low FPS when scrolling 
  - In some cases installing [vulkan-radeon](https://archlinux.org/packages/extra/x86_64/vulkan-radeon/) package resolves the FPS problem
  - Alternatively setting `WGPU_BACKEND=gl` environment variable may also resolve the FPS problem
- Reduced minimal required version of macOS to 11 (Big Sur)
- Fix panic when spamming enable/disable plugin or entrypoint checkbox in settings 

## [7] - 2024-08-30

### Big things
- Bundling improvements
  - `.dmg` file is now signed, notarized and stapled, removing the need for manual manipulations on `.dmg` file to be able to install Gauntlet
  - `.msi` installer for Windows is now available
  - `.tar.gz` archive for Linux now contains default `systemd` service, `.desktop` and `.png` icon file
- Added system tray icon on Windows and macOS with ability to open main or settings window, see version or quit Gauntlet
- Local plugins now always use development React bundle for better error messages at the expense of performance.

### General fixes
- Fix being unable to stop plugin in various situations including when there is unresolved pending promise
- Fix `generate-sample-theme` and `generate-sample-color-theme` CLI commands failing if config folder doesn't exist
- Fix console window popping up when starting Gauntlet on Windows

### `Applications` plugin
- On macOS and Linux, applications are now started detached from main process, fixing situations when other applications blocked Gauntlet from exiting 

### UI Improvements
- Action panel and download information panel in settings are now closed when clicking outside of panel on background

### API changes
- Type of `GeneratedCommand`'s `icon` field changed from `icon: ArrayBuffer | undefined` to `icon?: ArrayBuffer` 
- Default function returned from `command` and `command-generator` entrypoints can now be `async`

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