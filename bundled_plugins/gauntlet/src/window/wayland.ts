import { addOpenWindow, deleteOpenWindow } from "./shared";
import { GeneratedEntrypoint } from "@project-gauntlet/api/helpers";
import { application_wayland_pending_event } from "gauntlet:bridge/internal-linux";

export function applicationEventLoopWayland(
    focusWindow: (windowId: string) => void,
    openApplication: (appId: string) => void,
    add: (id: string, data: GeneratedEntrypoint) => void,
    get: (id: string) => GeneratedEntrypoint | undefined,
    getAll: () => { [id: string]: GeneratedEntrypoint },
) {
    const knownWindows: Record<string, { title: string | undefined, appId: string | undefined }> = { };

    // noinspection ES6MissingAwait
    (async () => {
        // noinspection InfiniteLoopJS
        while (true) {
            const applicationEvent = await application_wayland_pending_event();
            switch (applicationEvent.type) {
                case "WindowOpened": {
                    knownWindows[applicationEvent.window_id] = {
                        appId: undefined,
                        title: undefined
                    }

                    break;
                }
                case "WindowClosed": {
                    delete knownWindows[applicationEvent.window_id]

                    deleteOpenWindow(applicationEvent.window_id, openApplication, focusWindow, get, add)

                    break;
                }
                case "WindowTitleChanged": {
                    const windowId = applicationEvent.window_id;
                    const knownWindow = knownWindows[windowId];

                    if (knownWindow) {
                        knownWindow.title = applicationEvent.title;

                        const windowTitle = knownWindow.title;
                        const windowAppId = knownWindow.appId;

                        if (typeof windowAppId == "string") {
                            addOpenWindowWayland(
                                windowId,
                                windowAppId,
                                windowTitle,
                                focusWindow,
                                openApplication,
                                add,
                                get,
                                getAll
                            )
                        }
                    }

                    break;
                }
                case "WindowAppIdChanged": {
                    const windowId = applicationEvent.window_id;
                    const knownWindow = knownWindows[windowId];

                    if (knownWindow) {
                        knownWindow.appId = applicationEvent.app_id;

                        const windowTitle = knownWindow.title;
                        const windowAppId = knownWindow.appId;

                        if (typeof windowTitle == "string") {
                            addOpenWindowWayland(
                                windowId,
                                windowAppId,
                                windowTitle,
                                focusWindow,
                                openApplication,
                                add,
                                get,
                                getAll
                            )
                        }
                    }
                    break;
                }
            }
        }
    })()
}

function addOpenWindowWayland(
    windowId: string,
    appId: string,
    windowTitle: string,
    focusWindow: (windowId: string) => void,
    openApplication: (appId: string) => void,
    add: (id: string, data: GeneratedEntrypoint) => void,
    get: (id: string) => GeneratedEntrypoint | undefined,
    getAll: () => { [id: string]: GeneratedEntrypoint },
) {
    const newGet = (id: string): GeneratedEntrypoint | undefined => {
        let generatedEntrypoint = get(id);

        if (generatedEntrypoint == undefined) {
            const startupWmClassToAppId = Object.fromEntries(
                Object.entries(getAll())
                    .map(([appId, generated]): [string | undefined, string] => [((generated as any)["__linux__"]).startupWmClass, appId])
                    .filter((val): val is [string, string]  => {
                        const [wmClass, _appId] = val
                        return wmClass != undefined
                    })
            );

            const appIdFromWmClass = startupWmClassToAppId[id];

            if (appIdFromWmClass) {
                appId = appIdFromWmClass;
                generatedEntrypoint = get(appId);
            }
        }

        return generatedEntrypoint;
    }

    addOpenWindow(
        appId,
        windowId,
        windowTitle,
        openApplication,
        focusWindow,
        newGet,
        add,
    )
}
