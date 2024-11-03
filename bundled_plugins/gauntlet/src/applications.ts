import { GeneratedCommand, GeneratorProps } from "@project-gauntlet/api/helpers";
import { walk } from "@std/fs/walk";
import { debounce } from "@std/async/debounce";

// @ts-expect-error
const denoCore: DenoCore = Deno[Deno.internal].core;
const InternalApi: InternalApi = denoCore.ops;

type DesktopApplicationData = {
    name: string
    icon: ArrayBuffer | undefined,
}

type DesktopPathAction = DesktopPathActionAdd | DesktopPathActionRemove

type DesktopPathActionAdd = {
    type: "add",
    id: string,
    data: DesktopApplicationData
}

type DesktopPathActionRemove = {
    type: "remove"
    id: string
}

interface InternalApi {
    linux_open_application(desktop_id: string): void
    linux_application_dirs(): string[]
    linux_app_from_path(path: string): undefined | DesktopPathAction
}

export default async function Applications({ add, remove }: GeneratorProps): Promise<() => void> {
    const paths = InternalApi.linux_application_dirs()
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
        for await (const dirEntry of walk(path)) {
            const app = InternalApi.linux_app_from_path(dirEntry.path);
            if (app) {
                switch (app.type) {
                    case "add": {
                        add(app.id, commandFromApplication(app.id, app.data))
                        break;
                    }
                }
            }
        }
    }

    const watcher = Deno.watchFs(paths);

    const handle = debounce(
        (event: Deno.FsEvent) => {
            switch (event.kind) {
                case "create":
                case "modify":
                case "remove": {
                    for (const path of event.paths) {
                        const app = InternalApi.linux_app_from_path(path);
                        if (app) {
                            switch (app.type) {
                                case "remove": {
                                    remove(app.id)
                                    break;
                                }
                                case "add": {
                                    add(app.id, commandFromApplication(app.id, app.data))
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        },
        200
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

function commandFromApplication(id: string, app: DesktopApplicationData): GeneratedCommand {
    return {
        name: app.name,
        fn: () => {
            InternalApi.linux_open_application(id)
        },
        icon: app.icon, // TODO lazy icons
    };
}