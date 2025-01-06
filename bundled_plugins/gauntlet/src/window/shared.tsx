import { GeneratedCommand, GeneratedCommandAccessory, GeneratedCommandAction } from "@project-gauntlet/api/helpers";
import { List } from "@project-gauntlet/api/components";
import { linux_open_application } from "gauntlet:bridge/internal-linux";

export type OpenWindowData = {
    id: string,
    title: string,
    appId: string
}

export const openWindows: Record<string, OpenWindowData> = {};

export function applicationActions(
    id: string,
    openApplication: () => void,
    focusWindow: (windowId: string) => void,
): GeneratedCommandAction[] {
    const appWindows = Object.entries(openWindows)
        .filter(([_, windowData]) => windowData.appId == id)

    // TODO ability to close window

    if (appWindows.length == 0) {
        return [
            {
                label: "Open application",
                run: () => {
                    openApplication()
                },
            }
        ]
    } else if (appWindows.length == 1) {
        return [
            {
                label: "Focus window",
                run: () => {
                    let [appWindow] = appWindows;
                    let [windowId, _] = appWindow!!;
                    focusWindow(windowId)
                },
            }
        ]
    } else if (appWindows.length > 1) {
        return [
            {
                label: "Show windows",
                view: () => {
                    return (
                        <List>
                            {
                                appWindows
                                    .map(([_, windowData]) => (
                                        <List.Item
                                            key={windowData.id}
                                            title={windowData.title}
                                            onClick={() => {
                                                focusWindow(windowData.id)
                                            }}
                                        />
                                    ))
                            }
                        </List>
                    )
                }
            }
        ]
    } else {
        return []
    }
}

export function applicationAccessories(id: string): GeneratedCommandAccessory[] {
    const appWindows = Object.entries(openWindows)
        .filter(([_, windowData]) => windowData.appId == id)

    if (appWindows.length == 0) {
        return []
    } else if (appWindows.length == 1) {
        return [{ text: "1 window open" }]
    } else if (appWindows.length > 1) {
        return [{ text: `${appWindows.length} windows open` }]
    } else {
        return []
    }
}

export function addOpenWindow(
    appId: string,
    generatedEntrypoint: GeneratedCommand,
    windowId: string,
    windowTitle: string,
    openApplication: () => void,
    focusWindow: (windowId: string) => void,
    add: (id: string, data: GeneratedCommand) => void,
) {
    if (generatedEntrypoint) {
        openWindows[windowId] = {
            id: windowId,
            appId: appId,
            title: windowTitle
        }

        add(appId, {
            ...generatedEntrypoint,
            actions: applicationActions(appId, openApplication, focusWindow),
            accessories: applicationAccessories(appId)
        })
    }
}

export function deleteOpenWindow(
    windowId: string,
    openApplication: (appId: string) => (() => void),
    focusWindow: (windowId: string) => void,
    get: (id: string) => GeneratedCommand | undefined,
    add: (id: string, data: GeneratedCommand) => void,
) {
    const openWindow = openWindows[windowId];
    if (openWindow) {
        const generatedEntrypoint = get(openWindow.appId);

        delete openWindows[windowId];

        if (generatedEntrypoint) {
            add(openWindow.appId, {
                ...generatedEntrypoint,
                actions: applicationActions(openWindow.appId, openApplication(openWindow.appId), focusWindow),
                accessories: applicationAccessories(openWindow.appId)
            })
        }
    }
}

export function openLinuxApplication(appId: string) {
    return () => {
        linux_open_application(appId)
    }
}