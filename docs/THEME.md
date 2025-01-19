# Gauntlet Theming

Currently, in Gauntlet with themes it is possible to change (list is likely be extended with future updates):
- Colors of text and background
- Window border color, width and radius
- Border radius of components in content

Theming is only affects main window and doesn't affect settings

Theme config file is in TOML format

Theme config file locations:
- Windows:  `C:\Users\Username\AppData\Roaming\Gauntlet\config\theme.toml`
- Linux: `$XDG_CONFIG_HOME/gauntlet/theme.toml`
- macOS: `$HOME/Library/Application Support/dev.project-gauntlet.gauntlet/theme.toml`

Currently, theme change is only applied after application restart

Any errors in theme parsing will be shown in application logs

See bundled themes for examples [here](./../bundled_themes)
