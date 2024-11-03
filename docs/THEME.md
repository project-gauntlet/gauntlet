# Gauntlet Theming

Gauntlet has extensive theming possibilities

There are 2 types of themes:
- Color only
- Everything: Color, paddings, borders, etc

Unfortunately due to the internally invasive nature of themes, it is perpetually unstable feature.
Themes are versioned and only one version is supported by application at the same time.
Meaning if there were some changes made in the release and theme version was incremented,
theme will stop working until it is updated. 
This may change in the future

Current theme version:
- Color: `3`
- Everything: `3`

Theming is only applied to main window and doesn't affect settings

### Creating a custom theme

Gauntlet provides 2 CLI commands to generate sample: `generate-sample-color-theme` and `generate-sample-color-theme`. Sample is just a default theme that has been saved to file.

Running the command will create sample file, print location of that sample file
and will print location to which theme file will need to be saved to be detected by application

Currently, theme change is only applied after application restart

Any errors in theme parsing will be shown in application logs

#### Linux
- `gauntlet generate-sample-color-theme`
- `gauntlet generate-sample-theme`

#### macOS
Note: the binary is not on the PATH
- `/Applications/Gauntlet.app/Contents/MacOS/Gauntlet generate-sample-color-theme`
- `/Applications/Gauntlet.app/Contents/MacOS/Gauntlet generate-sample-theme`
