import { GeneratedCommand, GeneratorProps } from "@project-gauntlet/api/helpers";
import { walk, WalkOptions } from "@std/fs/walk";
import { debounce } from "@std/async/debounce";

// @ts-expect-error
const denoCore: DenoCore = Deno[Deno.internal].core;
const InternalApi: InternalApi = denoCore.ops;

type LinuxDesktopApplicationData = {
    name: string
    icon: ArrayBuffer | undefined,
}

type MacOSDesktopApplicationData = {
    name: string
    path: string,
    icon: ArrayBuffer | undefined,
}

type DesktopPathAction<DATA> = DesktopPathActionAdd<DATA> | DesktopPathActionRemove

type DesktopPathActionAdd<DATA> = {
    type: "add",
    id: string,
    data: DATA
}

type DesktopPathActionRemove = {
    type: "remove"
    id: string
}

type MacOSDesktopSettingsPre13Data = {
    name: string
    path: string,
    icon: ArrayBuffer | undefined,
}

type MacOSDesktopSettings13AndPostData = {
    name: string
    preferences_id: string
    icon: ArrayBuffer | undefined,
}

interface InternalApi {
    current_os(): string

    linux_open_application(desktop_id: string): void
    linux_application_dirs(): string[]
    linux_app_from_path(path: string): Promise<undefined | DesktopPathAction<LinuxDesktopApplicationData>>

    macos_major_version(): number

    macos_settings_pre_13(): MacOSDesktopSettingsPre13Data[]
    macos_settings_13_and_post(): MacOSDesktopSettings13AndPostData[]
    macos_open_setting_13_and_post(preferences_id: String): void
    macos_open_setting_pre_13(setting_path: String): void

    macos_system_applications(): string[]
    macos_application_dirs(): string[]
    macos_app_from_path(path: string): Promise<undefined | DesktopPathAction<MacOSDesktopApplicationData>>
    macos_app_from_arbitrary_path(path: string): Promise<undefined | DesktopPathAction<MacOSDesktopApplicationData>>
    macos_open_application(app_path: String): void
}

export default async function Applications({ add, remove }: GeneratorProps): Promise<void | (() => void)> {
    switch (InternalApi.current_os()) {
        case "linux": {
            return await genericGenerator(
                InternalApi.linux_application_dirs(),
                path => InternalApi.linux_app_from_path(path),
                (id, data) => ({
                    name: data.name,
                    fn: () => {
                        InternalApi.linux_open_application(id)
                    },
                    icon: data.icon, // TODO lazy icons
                }),
                add,
                remove
            );
        }
        case "macos": {
            const majorVersion = InternalApi.macos_major_version();

            if (majorVersion >= 13) {
                for (const setting of InternalApi.macos_settings_13_and_post()) {
                    add(`settings:${setting.preferences_id}`, {
                        name: setting.name,
                        fn: () => {
                            InternalApi.macos_open_setting_13_and_post(setting.preferences_id)
                        },
                        icon: setting.icon,
                    })
                }
            } else {
                for (const setting of InternalApi.macos_settings_pre_13()) {
                    add(`settings:${setting.path}`, {
                        name: setting.name,
                        fn: () => {
                            InternalApi.macos_open_setting_pre_13(setting.path)
                        },
                        icon: setting.icon,
                    })
                }
            }

            for (const path of InternalApi.macos_system_applications()) {
                const app = await InternalApi.macos_app_from_path(path)
                if (app) {
                    switch (app.type) {
                        case "add": {
                            let data = app.data;
                            add(data.path, {
                                name: data.name,
                                fn: () => {
                                    InternalApi.macos_open_application(data.path)
                                },
                                icon: data.icon,
                            })
                            break;
                        }
                    }
                } else {
                    console.error(`System application '${path}' was not loaded`)
                }
            }

            return await genericGenerator(
                InternalApi.macos_application_dirs(),
                path => InternalApi.macos_app_from_arbitrary_path(path),
                (_id, data) => ({
                    name: data.name,
                    fn: () => {
                        InternalApi.macos_open_application(data.path)
                    },
                    icon: data.icon,
                }),
                add,
                remove,
                { exts: ["app"], maxDepth: 2 }
            );
        }
    }
}

async function genericGenerator<DATA>(
    directoriesToWatch: string[],
    appFromPath: (path: string) => Promise<undefined | DesktopPathAction<DATA>>,
    commandFromApp: (id: string, data: DATA) => GeneratedCommand,
    add: (id: string, data: GeneratedCommand) => void,
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
    })()

    return () => {
        watcher.close()
    }
}
