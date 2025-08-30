import { addOpenWindow, deleteOpenWindow, openWindows } from "./shared";
import { GeneratedEntrypoint } from "@project-gauntlet/api/helpers";
import { application_macos_pending_event } from "gauntlet:bridge/internal-macos";

export function applicationEventLoopMacos(
    focusWindow: (windowId: string) => void,
    openApplication: (appId: string) => void,
    add: (id: string, data: GeneratedEntrypoint) => void,
    get: (id: string) => GeneratedEntrypoint | undefined,
) {
    // noinspection ES6MissingAwait
    (async () => {
        // noinspection InfiniteLoopJS
        while (true) {
            const applicationEvent = await application_macos_pending_event();
            switch (applicationEvent.type) {
                case "WindowOpened": {
                    addOpenWindow(
                        applicationEvent.bundle_path,
                        applicationEvent.window_id,
                        applicationEvent.title,
                        openApplication,
                        focusWindow,
                        get,
                        add,
                    )

                    break;
                }
                case "WindowClosed": {
                    deleteOpenWindow(
                        applicationEvent.window_id,
                        openApplication,
                        focusWindow,
                        get,
                        add
                    )

                    break;
                }
                case "WindowTitleChanged": {
                    const window = openWindows()[applicationEvent.window_id];
                    if (window) {
                        addOpenWindow(
                            window.appId,
                            applicationEvent.window_id,
                            applicationEvent.title,
                            openApplication,
                            focusWindow,
                            get,
                            add,
                        )
                    }
                    break;
                }
            }
        }
    })()
}

