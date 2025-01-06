import { addOpenWindow, deleteOpenWindow, openLinuxApplication } from "./shared";
import { GeneratedCommand } from "@project-gauntlet/api/helpers";
import { linux_wayland_focus_window, application_wayland_pending_event } from "gauntlet:bridge/internal-linux";


export function focusWaylandWindow(windowId: string) {
    linux_wayland_focus_window(windowId)
}

export function applicationEventLoopWayland(
    focusWindow: (windowId: string) => void,
    add: (id: string, data: GeneratedCommand) => void,
    get: (id: string) => GeneratedCommand | undefined,
    getAll: () => { [id: string]: GeneratedCommand },
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

                    deleteOpenWindow(applicationEvent.window_id, openLinuxApplication, focusWindow, get, add)

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
    windowAppId: string,
    windowTitle: string,
    focusWindow: (windowId: string) => void,
    add: (id: string, data: GeneratedCommand) => void,
    get: (id: string) => GeneratedCommand | undefined,
    getAll: () => { [id: string]: GeneratedCommand },
) {
    let appId = windowAppId;
    let generatedEntrypoint = get(windowAppId);

    if (generatedEntrypoint == undefined) {
        const startupWmClassToAppId = Object.fromEntries(
            Object.entries(getAll())
                .map(([appId, generated]): [string | undefined, string] => [((generated as any)["__linux__"]).startupWmClass, appId])
                .filter((val): val is [string, string]  => {
                    const [wmClass, _appId] = val
                    return wmClass != undefined
                })
        );

        const appIdFromWmClass = startupWmClassToAppId[windowAppId];

        if (appIdFromWmClass) {
            appId = appIdFromWmClass;
            generatedEntrypoint = get(appId);
        }
    }

    if (generatedEntrypoint) {
        addOpenWindow(
            appId,
            generatedEntrypoint,
            windowId,
            windowTitle,
            openLinuxApplication(appId),
            focusWindow,
            add,
        )
    }
}