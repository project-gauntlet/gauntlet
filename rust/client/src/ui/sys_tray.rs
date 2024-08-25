use image::ImageFormat;

pub fn create_tray() -> tray_icon::TrayIcon {
    use tray_icon::TrayIconBuilder;
    use tray_icon::menu::{MenuEvent, Menu, MenuItem, PredefinedMenuItem, AboutMetadataBuilder};

    MenuEvent::set_event_handler(Some(|event: MenuEvent| {
        match event.id().as_ref() {
            "GAUNTLET_OPEN_MAIN_WINDOW" => {
                crate::open_window()
            }
            "GAUNTLET_OPEN_SETTING_WINDOW" => {
                crate::open_settings_window()
            }
            _ => {}
        }
    }));

    let (tray_icon, muda_icon) = {
        let bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/linux/icon_256.png"));

        let image = image::load_from_memory_with_format(bytes, ImageFormat::Png)
            .expect("Failed to open icon path")
            .into_rgba8();

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();

        let tray_icon = tray_icon::Icon::from_rgba(rgba.clone(), width, height)
            .expect("Failed to open icon");

        let muda_icon = tray_icon::menu::Icon::from_rgba(rgba, width, height)
            .expect("Failed to open icon");

        (tray_icon, muda_icon)
    };

    let about_metadata = AboutMetadataBuilder::new()
        .name(Some("Gauntlet"))
        .version(Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../VERSION"))))
        .authors(Some(vec!["Exidex".to_string()]))
        .credits(Some("Exidex".to_string()))
        .license(Some("MPL-2.0"))
        .website(Some("https://github.com/project-gauntlet/gauntlet"))
        .icon(Some(muda_icon))
        .build();

    let menu = Menu::with_items(
        &[
            &MenuItem::new("Gauntlet", false, None),
            &MenuItem::with_id("GAUNTLET_OPEN_MAIN_WINDOW", "Open", true, None),
            &MenuItem::with_id("GAUNTLET_OPEN_SETTING_WINDOW", "Open Settings", true, None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::about(Some("About..."), Some(about_metadata)),
            &PredefinedMenuItem::quit(Some("Quit Gauntlet")),
        ]
    ).expect("unable to create tray menu");

    TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_icon(tray_icon)
        .build()
        .expect("unable to create tray")
}