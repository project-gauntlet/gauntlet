use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use std::env;
use std::io::Read;
use std::path::{Path, PathBuf};

use directories::BaseDirs;
use freedesktop_entry_parser::parse_entry;
use freedesktop_icons::lookup;
use image::ImageFormat;
use image::imageops::FilterType;
use regex::Regex;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct DesktopEntry {
    pub name: String,
    pub icon: Option<Vec<u8>>,
    pub command: Vec<String>,
}

#[cfg(target_os = "linux")]
fn find_application_dirs() -> Option<Vec<PathBuf>> {
    let data_home = match env::var_os("XDG_DATA_HOME") {
        Some(val) => {
            PathBuf::from(val)
        },
        None => {
            let base_dirs = BaseDirs::new()?;
            let home = base_dirs.home_dir();
            home.join(".local/share")
        }
    };
    let extra_data_dirs = match env::var_os("XDG_DATA_DIRS") {
        Some(val) => {
            env::split_paths(&val).map(PathBuf::from).collect()
        },
        None => {
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share")
            ]
        }
    };

    let mut res = Vec::new();
    res.push(data_home.join("applications"));
    for dir in extra_data_dirs {
        res.push(dir.join("applications"));
    }
    Some(res)
}

#[cfg(target_os = "linux")]
pub fn get_apps() -> Vec<DesktopEntry> {
    let app_dirs = find_application_dirs()
        .unwrap_or_default()
        .into_iter()
        .filter(|dir| dir.exists())
        .collect::<Vec<_>>();

    let mut result: HashMap<String, DesktopEntry> = HashMap::new();

    for app_dir in app_dirs {
        let found_desktop_entries = WalkDir::new(app_dir.clone())
            .into_iter()
            .filter_map(|dir_entry| dir_entry.ok())
            .filter(|dir_entry| dir_entry.file_type().is_file())
            .filter_map(|path| {
                let path = path.path();

                tracing::debug!("path: {:?}", path);

                match path.extension() {
                    None => None,
                    Some(extension) => {
                        match extension.to_str() {
                            Some("desktop") => {

                                let desktop_id = path.strip_prefix(&app_dir)
                                    .ok()?
                                    .to_str()?
                                    .to_owned();

                                let entry = create_app_entry(path.to_path_buf())?;

                                Some((desktop_id, entry))
                            },
                            _ => None,
                        }
                    }
                }
            })
            .collect::<HashMap<_, _>>();

        for (path, desktop_entry) in found_desktop_entries {
            if let Vacant(entry) = result.entry(path) {
                entry.insert(desktop_entry);
            }
        }
    }

    result.into_values().collect()
}

#[cfg(target_os = "linux")]
fn create_app_entry(path: PathBuf) -> Option<DesktopEntry> {
    let entry = parse_entry(&path)
        .inspect_err(|err| tracing::warn!("error parsing .desktop file at path {:?}: {:?}", &path, err))
        .ok()?;

    let entry = entry.section("Desktop Entry");

    let name = entry.attr("Name")?;
    let exec = entry.attr("Exec")?;
    let icon = entry.attr("Icon").map(|s| s.to_string());
    let no_display = entry.attr("NoDisplay").map(|val| val == "true").unwrap_or(false);
    let hidden = entry.attr("Hidden").map(|val| val == "true").unwrap_or(false);
    // TODO NotShowIn, OnlyShowIn https://wiki.archlinux.org/title/desktop_entries
    // TODO DBusActivatable
    if no_display || hidden {
        return None
    }

    let exec = unescape_freedesktop_type_string(&exec);
    let command = parse_freedesktop_exec(&exec)
        .inspect_err(|err| tracing::warn!("error parsing Exec {:?}: {:?}", &path, err))
        .ok()?;

    let icon = icon
        .map(|icon| {
            let icon_path = PathBuf::from(&icon);
            if icon_path.is_absolute() {
                Some(icon_path)
            } else {
                lookup(&icon)
                    .with_size(48)
                    .find()
            }
        })
        .flatten()
        .inspect(|path| tracing::debug!("icon path: {:?}", path))
        .map(|path| {
            match path.extension() {
                None => Err(anyhow::anyhow!("unknown format")),
                Some(extension) => {
                    match extension.to_str() {
                        Some("png") => {
                            let data = std::fs::read(path)?;

                            resize_icon(data)
                        },
                        Some("svg") => {
                            let data = std::fs::read(path)?;

                            let tree = resvg::usvg::Tree::from_data(&data, &resvg::usvg::Options::default())?;

                            let pixmap_size = tree.size().to_int_size();
                            let mut pixmap = resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();

                            resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

                            let data = pixmap.encode_png()?;

                            let data = resize_icon(data)?;

                            Ok(data)
                        },
                        Some("xpm") => Err(anyhow::anyhow!("xpm format")),
                        _ => Err(anyhow::anyhow!("unsupported by spec format {:?}", extension)),
                    }
                }
            }
        })
        .map(|res| {
            res
                .inspect_err(|err| tracing::warn!("error processing icon {:?}: {:?}", &path, err))
                .ok()
        })
        .flatten();

    Some(DesktopEntry {
        name: name.to_string(),
        icon,
        command,
    })
}

fn resize_icon(data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let data = image::load_from_memory_with_format(&data, ImageFormat::Png)?;
    let data = image::imageops::resize(&data, 48, 48, FilterType::Lanczos3);

    let mut buffer = std::io::Cursor::new(vec![]);

    data.write_to(&mut buffer, ImageFormat::Png)?;

    Ok(buffer.into_inner())
}

fn parse_freedesktop_exec(value: &str) -> anyhow::Result<Vec<String>> {
    // TODO handle %i, %c, %k properly

    let mut command: Vec<String> = vec![];
    let mut current = String::new();
    let mut in_quote: bool = false;
    let mut escape_next: bool = false;
    let mut field_code_next: bool = false;
    for char in value.chars() {
        match char {
            '\\' => {
                if field_code_next {
                    return Err(anyhow::anyhow!("unknown field code \"{char}\""));
                }

                match (in_quote, escape_next) {
                    (true, true) => {
                        current.push('\\');
                        escape_next = false;
                    }
                    (true, false) => {
                        escape_next = true;
                    }
                    (false, _) => {
                        return Err(anyhow::anyhow!("special character backslash (\\) outside of quotes"));
                    }
                }
            }
            ' ' => {
                if escape_next {
                    return Err(anyhow::anyhow!("unknown character escape space \" \""));
                }
                if field_code_next {
                    return Err(anyhow::anyhow!("unknown field code \"{char}\""));
                }

                if in_quote {
                    current.push(' ');
                } else {
                    // argument separator

                    // https://specifications.freedesktop.org/desktop-entry-spec/latest/ar01s07.html
                    // spec says "a space" but there is probably no need to restrict separator to a single space
                    if !current.is_empty() {
                        command.push(current);
                        current = String::new();
                    }
                }
            },
            '"' => {
                if field_code_next {
                    return Err(anyhow::anyhow!("unknown field code \"{char}\""));
                }

                match (in_quote, escape_next) {
                    (true, true) => {
                        current.push('"');

                    }
                    (true, false) => {
                        command.push(current);
                        current = String::new();
                        in_quote = !in_quote;
                    }
                    (false, true) => {
                        unreachable!("impossible for escape to be outside of quotes");
                    }
                    (false, false) => {
                        in_quote = !in_quote;
                    }
                }

                escape_next = false;
            },
            '`' | '$' => {
                if field_code_next {
                    return Err(anyhow::anyhow!("unknown field code \"{char}\""));
                }

                match (in_quote, escape_next) {
                    (true, true) => {
                        current.push(char);
                    }
                    (true, false) => {
                        return Err(anyhow::anyhow!("backtick (`) and dollar sign ($) must always be escaped"));
                    }
                    (false, _) => {
                        unreachable!("impossible for escape to be outside of quotes");
                    }
                }
                escape_next = false;
            }
            '%' => {
                if field_code_next {
                    current.push('%');
                    field_code_next = false;
                } else if in_quote {
                    current.push('%'); // spec says it is undefined what happens here
                    field_code_next = false;
                } else {
                    field_code_next = true;
                }
            }
            char @ _ => {
                if field_code_next {
                    match char {
                        'f' | 'F' | 'u' | 'U' | 'd' | 'D' | 'n' | 'N' | 'v' | 'm' => {
                            // ignore
                        }
                        'i' | 'c' | 'k' => {
                            // TODO support
                        }
                        _ => {
                            return Err(anyhow::anyhow!("unknown field code \"{char}\""));
                        }
                    }
                } else if escape_next {
                    if in_quote {
                        return Err(anyhow::anyhow!("unknown character escape \"{char}\""));
                    } else {
                        unreachable!("impossible for escape to be outside of quotes");
                    }
                } else {
                    current.push(char);
                }
                escape_next = false;
                field_code_next = false;
            },
        }
    }

    if !current.is_empty() {
        command.push(current);
    }

    Ok(command)
}

fn unescape_freedesktop_type_string(value: &str) -> String {
    // https://specifications.freedesktop.org/desktop-entry-spec/latest/ar01s04.html
    // unescape \s, \n, \t, \r, and \\

    let mut out = String::new();
    let mut escape_next = false;
    for char in value.chars() {
        match char {
            '\\' => {
                if escape_next {
                    out.push('\\');
                    escape_next = false;
                } else {
                    escape_next = true;
                }
            }
            's' => {
                if escape_next {
                    out.push(' ');
                } else {
                    out.push('s');
                }
                escape_next = false;
            }
            'n' => {
                if escape_next {
                    out.push('\n');
                } else {
                    out.push('n');
                }
                escape_next = false;
            }
            't' => {
                if escape_next {
                    out.push('\t');
                } else {
                    out.push('t');
                }
                escape_next = false;
            }
            'r' => {
                if escape_next {
                    out.push('\r');
                } else {
                    out.push('r');
                }
                escape_next = false;
            }
            other @ _ => {
                if escape_next {
                    out.push('\\');
                    out.push(other);
                } else {
                    out.push(other);
                }
                escape_next = false;
            }
        }
    }

    if escape_next {
        out.push('\\');
    }

    out
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn not_escaped_special_chars() {
        assert_eq!(unescape_freedesktop_type_string(r#"\"#), "\\"); // this one should be invalid
        assert_eq!(unescape_freedesktop_type_string(r#"r"#), "r");
        assert_eq!(unescape_freedesktop_type_string(r#"s"#), "s");
        assert_eq!(unescape_freedesktop_type_string(r#"n"#), "n");
        assert_eq!(unescape_freedesktop_type_string(r#"t"#), "t");
        assert_eq!(unescape_freedesktop_type_string(r#" "#), " ");
    }

    #[test]
    fn escaped_special_chars() {
        assert_eq!(unescape_freedesktop_type_string(r#"\\"#), "\\");
        assert_eq!(unescape_freedesktop_type_string(r#"\r"#), "\r");
        assert_eq!(unescape_freedesktop_type_string(r#"\r"#), "\r");
        assert_eq!(unescape_freedesktop_type_string(r#"\s"#), " ");
        assert_eq!(unescape_freedesktop_type_string(r#"\n"#), "\n");
        assert_eq!(unescape_freedesktop_type_string(r#"\t"#), "\t");
    }

    #[test]
    fn not_escaped_non_special() {
        assert_eq!(unescape_freedesktop_type_string(r#"\i"#), "\\i"); // this one should be invalid
        assert_eq!(unescape_freedesktop_type_string(r#"\ "#), "\\ "); // this one should be invalid
    }


    fn assert_ok(actual: anyhow::Result<Vec<String>>, expected_ok: &[&str]) {
        let expected = expected_ok.into_iter().map(|s| s.to_owned()).collect::<Vec<_>>();
        match actual {
            Ok(val) => assert_eq!(val, expected),
            Err(err) => panic!("expecting ok, found: {:?}", err)
        }
    }
    fn assert_err(actual: anyhow::Result<Vec<String>>, expected_err: &str) {
        match actual {
            Ok(val) => panic!("expecting error, found: {:?}", val),
            Err(err) => assert_eq!(format!("{}", err.root_cause()), expected_err)
        }
    }

    #[test]
    fn unquote() {
        assert_ok(parse_freedesktop_exec(r#"test"#), &["test"]);
        assert_ok(parse_freedesktop_exec(r#""test""#), &["test"]);
        assert_ok(parse_freedesktop_exec(r#""test" t"#), &["test", "t"]);
        assert_ok(parse_freedesktop_exec(r#""test" "t""#), &["test", "t"]);
        assert_ok(parse_freedesktop_exec(r#""test" "t""#), &["test", "t"]);
        assert_ok(parse_freedesktop_exec(r#"test "t""#), &["test", "t"]);
        assert_ok(parse_freedesktop_exec(r#""te\"st""#), &["te\"st"]);
        assert_ok(parse_freedesktop_exec(r#""te\`st""#), &["te`st"]);
        assert_ok(parse_freedesktop_exec(r#""te\$t""#), &["te$t"]);
        assert_ok(parse_freedesktop_exec(r#""te\\t""#), &["te\\t"]);
        assert_ok(parse_freedesktop_exec(r#" "test""#), &["test"]); // not sure about this one
        assert_ok(parse_freedesktop_exec(r#""test"  "t"#), &["test", "t"]); // and this one
    }

    #[test]
    fn unquote_special_field_codes() {
        assert_ok(parse_freedesktop_exec(r#"test %%"#), &["test", "%"]);
        assert_ok(parse_freedesktop_exec(r#"test %f"#), &["test"]);
        assert_ok(parse_freedesktop_exec(r#"test %F"#), &["test"]);
        assert_ok(parse_freedesktop_exec(r#"test %u"#), &["test"]);
        assert_ok(parse_freedesktop_exec(r#"test %U"#), &["test"]);
        assert_ok(parse_freedesktop_exec(r#"test %d"#), &["test"]);
        assert_ok(parse_freedesktop_exec(r#"test %D"#), &["test"]);
        assert_ok(parse_freedesktop_exec(r#"test %n"#), &["test"]);
        assert_ok(parse_freedesktop_exec(r#"test %N"#), &["test"]);
        assert_ok(parse_freedesktop_exec(r#"test %i"#), &["test"]); // %i is not supported yet
        assert_ok(parse_freedesktop_exec(r#"test %c"#), &["test"]); // %c is not supported yet
        assert_ok(parse_freedesktop_exec(r#"test %k"#), &["test"]); // %k is not supported yet
        assert_ok(parse_freedesktop_exec(r#"test %v"#), &["test"]);
        assert_ok(parse_freedesktop_exec(r#"test %m"#), &["test"]);

        assert_ok(parse_freedesktop_exec(r#"test %f %i --test"#), &["test", "--test"]);

        assert_err(parse_freedesktop_exec(r#"test %q"#), "unknown field code \"q\"");
    }

    #[test]
    fn double_escape() {
        assert_ok(parse_freedesktop_exec(&unescape_freedesktop_type_string(r#""test\\\\test""#)), &["test\\test"]);
        assert_ok(parse_freedesktop_exec(&unescape_freedesktop_type_string(r#""test\\$test""#)), &["test$test"]);
    }
}
