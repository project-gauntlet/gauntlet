import { GeneratedEntrypoint, GeneratorContext } from "@project-gauntlet/api/helpers";
import { walk, WalkOptions } from "@std/fs/walk";
import { debounce } from "@std/async/debounce";
import { current_os, wayland } from "gauntlet:bridge/internal-all";
import {
    linux_app_from_path,
    linux_application_dirs,
    linux_open_application,
    linux_wayland_focus_window,
    linux_x11_focus_window
} from "gauntlet:bridge/internal-linux";
import {
    macos_app_from_arbitrary_path,
    macos_app_from_path,
    macos_application_dirs,
    macos_focus_window,
    macos_get_localized_language,
    macos_major_version,
    macos_open_application,
    macos_open_setting_13_and_post,
    macos_open_setting_pre_13,
    macos_settings_13_and_post,
    macos_settings_pre_13,
    macos_system_applications
} from "gauntlet:bridge/internal-macos";
import { applicationAccessories, applicationActions } from "./window/shared";
import { applicationEventLoopX11 } from "./window/x11";
import { applicationEventLoopWayland } from "./window/wayland";
import {
    windows_app_from_path,
    windows_application_dirs,
    windows_open_application
} from "gauntlet:bridge/internal-windows";
import { applicationEventLoopMacos } from "./window/macos";

type EntrypointPreferences = { experimentalWindowTracking: boolean, bundleNameLang: "default" | "localized" };

export default async function Applications(context: GeneratorContext<object, EntrypointPreferences>): Promise<void | (() => void)> {
    const { add, remove, get, getAll, entrypointPreferences: { experimentalWindowTracking, bundleNameLang } } = context;

    switch (current_os()) {
        case "linux": {
            const cleanup = await genericGenerator<LinuxDesktopApplicationData>(
                linux_application_dirs(),
                path => linux_app_from_path(path),
                (id, data) => {
                    if (wayland()) {
                        return {
                            name: data.name,
                            actions: applicationActions(
                                id,
                                experimentalWindowTracking,
                                linux_open_application,
                                linux_wayland_focus_window,
                            ),
                            accessories: applicationAccessories(id, experimentalWindowTracking),
                            icon: data.icon, // TODO lazy icons
                            "__linux__": {
                                startupWmClass: data.startup_wm_class,
                                desktopFilePath: data.desktop_file_path
                            }
                        }
                    } else {
                        return {
                            name: data.name,
                            actions: applicationActions(
                                id,
                                experimentalWindowTracking,
                                linux_open_application,
                                linux_x11_focus_window,
                            ),
                            accessories: applicationAccessories(id, experimentalWindowTracking),
                            icon: data.icon, // TODO lazy icons
                            "__linux__": {
                                startupWmClass: data.startup_wm_class,
                                desktopFilePath: data.desktop_file_path
                            }
                        }
                    }
                },
                add,
                remove,
            );

            if (experimentalWindowTracking) {
                if (wayland()) {
                    try {
                        applicationEventLoopWayland(
                            linux_wayland_focus_window,
                            linux_open_application,
                            add,
                            get,
                            getAll
                        );
                    } catch (e) {
                        console.log("error when setting up wayland application event loop", e)
                    }
                } else {
                    try {
                        applicationEventLoopX11(
                            linux_x11_focus_window,
                            linux_open_application,
                            add,
                            get,
                            getAll
                        );
                    } catch (e) {
                        console.log("error when setting up x11 application event loop", e)
                    }
                }
            }

            return cleanup;
        }
        case "macos": {
            let lang: string | undefined;

            switch (bundleNameLang) {
                case "default": {
                    lang = undefined;
                    break;
                }
                case "localized": {
                    lang = macos_get_localized_language();
                    break;
                }
                default: {
                    throw new Error("Unknown bundle name type")
                }
            }

            const majorVersion = macos_major_version();

            if (majorVersion >= 13) {
                for (const setting of macos_settings_13_and_post(lang)) {
                    add(`settings:${setting.preferences_id}`, {
                        name: setting.name,
                        actions: [
                            {
                                label: "Open settings",
                                run: () => {
                                    macos_open_setting_13_and_post(setting.preferences_id)
                                },
                            }
                        ],
                        icon: setting.icon,
                    })
                }
            } else {
                for (const setting of macos_settings_pre_13()) {
                    add(`settings:${setting.path}`, {
                        name: setting.name,
                        actions: [
                            {
                                label: "Open settings",
                                run: () => {
                                    macos_open_setting_pre_13(setting.path)
                                },
                            }
                        ],
                        icon: setting.icon,
                    })
                }
            }

            for (const path of macos_system_applications()) {
                const app = await macos_app_from_path(path, lang)
                if (app) {
                    switch (app.type) {
                        case "add": {
                            let data = app.data;
                            add(data.path, {
                                name: data.name,
                                actions: [
                                    {
                                        label: "Open application",
                                        run: () => macos_open_application(data.path),
                                    }
                                ],
                                icon: data.icon,
                            })
                            break;
                        }
                    }
                } else {
                    console.error(`System application '${path}' was not loaded`)
                }
            }

            const cleanup = await genericGenerator<MacOSDesktopApplicationData>(
                macos_application_dirs(),
                path => macos_app_from_arbitrary_path(path, lang),
                (_id, data) => ({
                    name: data.name,
                    actions: applicationActions(
                        data.path,
                        experimentalWindowTracking,
                        macos_open_application,
                        macos_focus_window,
                    ),
                    accessories: applicationAccessories(data.path, experimentalWindowTracking),
                    icon: data.icon,
                }),
                add,
                remove,
                { exts: ["app"], maxDepth: 2, followSymlinks: true, }
            );

            if (experimentalWindowTracking) {
                try {
                    applicationEventLoopMacos(
                        macos_focus_window,
                        macos_open_application,
                        add,
                        get,
                    );
                } catch (e) {
                    console.log("error when setting up macos application event loop", e)
                }
            }

            return cleanup
        }
        case "windows": {
            return await genericGenerator<WindowsDesktopApplicationData>(
                windows_application_dirs(),
                path => windows_app_from_path(path),
                (_id, data) => ({
                    name: data.name,
                    actions: [
                        {
                            label: "Open application",
                            run: () => {
                                windows_open_application(data.path)
                            },
                        }
                    ],
                    icon: data.icon,
                }),
                add,
                remove,
                { exts: ["lnk", "exe"], maxDepth: 2 }
            );
        }
    }
}

async function genericGenerator<DATA>(
    directoriesToWatch: string[],
    appFromPath: (path: string) => Promise<undefined | DesktopPathAction<DATA>>,
    commandFromApp: (id: string, data: DATA) => GeneratedEntrypoint,
    add: (id: string, data: GeneratedEntrypoint) => void,
    remove: (id: string) => void,
    walkOpts?: WalkOptions
): Promise<() => void> {
    const paths = directoriesToWatch
        .filter(path => {
            try {
                Deno.lstatSync(path)
                return true
            } catch (err) {
                // most frequent error here is NotFound
                return false
            }
        });

    for (const path of paths) {
        for await (const dirEntry of walk(path, walkOpts)) {
            const app = await appFromPath(dirEntry.path);
            if (app) {
                switch (app.type) {
                    case "add": {
                        add(app.id, commandFromApp(app.id, app.data))
                        break;
                    }
                }
            }
        }
    }

    const watcher = Deno.watchFs(paths);

    const handle = debounce(
        async (event: Deno.FsEvent) => {
            switch (event.kind) {
                case "create":
                case "modify":
                case "remove": {
                    for (const path of event.paths) {
                        const app = await appFromPath(path);
                        if (app) {
                            switch (app.type) {
                                case "remove": {
                                    remove(app.id)
                                    break;
                                }
                                case "add": {
                                    add(app.id, commandFromApp(app.id, app.data))
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        },
        1000
    );

    // noinspection ES6MissingAwait
    (async () => {
        for await (const event of watcher) {
            handle(event)
        }
    })();

    return () => {
        watcher.close()
    }
}
