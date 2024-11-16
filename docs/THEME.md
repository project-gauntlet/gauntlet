# Gauntlet Theming

Gauntlet has extensive theming possibilities

There are 2 types of themes:
- Simple Theme
  - Global colors
  - Global border width and radius
- Complex Theme
  - Colors on per widget type bases
  - Paddings and spacing on per widget type bases
  - Borders colors, width and radius on per widget type bases

Unfortunately due to the internally invasive nature of themes, it is perpetually unstable feature.
Themes are versioned and only one version is supported by application at the same time.
Meaning if there were some changes made in the release and theme version was incremented,
theme will stop working until it is updated. 
This may change in the future

Current version:
- Simple Theme: `4`
- Complex Theme: `4`

Theming is only applied to main window and doesn't affect settings

### Creating a custom theme

Gauntlet provides 2 CLI commands to generate sample: `generate-sample-simple-theme` and `generate-sample-complex-theme`. Sample is just a default theme that has been saved to file.

Running the command will create sample file, print location of that sample file
and will print location to which theme file will need to be saved to be detected by application

Currently, theme change is only applied after application restart

Any errors in theme parsing will be shown in application logs

#### Linux
- `gauntlet generate-sample-simple-theme`
- `gauntlet generate-sample-complex-theme`

#### macOS
Note: the binary is not on the PATH
- `/Applications/Gauntlet.app/Contents/MacOS/Gauntlet generate-sample-simple-theme`
- `/Applications/Gauntlet.app/Contents/MacOS/Gauntlet generate-sample-complex-theme`
