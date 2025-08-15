import {
    GeneratedEntrypoint,
    GeneratedEntrypointAccessory,
    GeneratedEntrypointAction,
} from "@project-gauntlet/api/helpers";
import { linux_open_application } from "gauntlet:bridge/internal-linux";
import { Action, ActionPanel, List } from "@project-gauntlet/api/components";

export function ListOfWindows({ windows, focusWindow }: {
    windows: Record<string, OpenWindowData>,
    focusWindow: (windowId: string) => void
}) {
    const knownWindows = readWindowOrder();

    const sortedWindows = Object.keys(windows) // sort windows based on array stored on storage
        .sort((a, b) => knownWindows.indexOf(a) - knownWindows.indexOf(b));

    return (
        <List
            actions={
                <ActionPanel>
                    <Action
                        label="Focus window"
                        onAction={id => {
                            if (id) {
                                focusAndSort(id, focusWindow)
                                return { close: true }
                            }
                        }}
                    />
                </ActionPanel>
            }
        >
            {
                sortedWindows.map(window => <List.Item key={window} id={window} title={windows[window]!!.title}/>)
            }
        </List>
    )
}

export type OpenWindowData = {
    id: string,
    title: string,
    appId: string
}


export function openWindows(): Record<string, OpenWindowData> {
    if ((globalThis as any).__openWindows == undefined) {
        (globalThis as any).__openWindows = {}
    }
    return (globalThis as any).__openWindows
}

export function applicationActions(
    id: string,
    experimentalWindowTracking: boolean,
    openApplication: () => void,
    focusWindow: (windowId: string) => void,
): GeneratedEntrypointAction[] {
    if (!experimentalWindowTracking) {
        return [
            {
                label: "Open application",
                run: () => {
                    openApplication()
                },
            }
        ]
    }

    const appWindows = Object.fromEntries(
        Object.entries(openWindows())
            .filter(([_, windowData]) => windowData.appId == id)
    )

    // TODO ability to close window

    const windowCount = Object.keys(appWindows).length;

    if (windowCount == 0) {
        return [
            {
                label: "Open application",
                run: () => {
                    openApplication()
                },
            }
        ]
    } else if (windowCount == 1) {
        return [
            {
                label: "Focus window",
                run: () => {
                    let [windowId] = Object.keys(appWindows);
                    focusAndSort(windowId!!, focusWindow)
                },
            },
            {
                label: "Open new instance",
                run: () => {
                    openApplication()
                },
            }
        ]
    } else if (windowCount > 1) {
        return [
            {
                label: "Show windows",
                view: () => {
                    return (
                        <ListOfWindows
                            windows={appWindows}
                            focusWindow={windowId => focusWindow(windowId)}
                        />
                    )
                }
            },
            {
                label: "Open new instance",
                run: () => {
                    openApplication()
                },
            }
        ]
    } else {
        return []
    }
}

export function applicationAccessories(id: string, experimentalWindowTracking: boolean): GeneratedEntrypointAccessory[] {
    if (!experimentalWindowTracking) {
        return []
    }

    const appWindows = Object.entries(openWindows())
        .filter(([_, windowData]) => windowData.appId == id)

    if (appWindows.length == 0) {
        return []
    } else if (appWindows.length == 1) {
        return [{ text: "1 window" }]
    } else if (appWindows.length > 1) {
        return [{ text: `${appWindows.length} windows` }]
    } else {
        return []
    }
}

export function addOpenWindow(
    appId: string,
    generatedEntrypoint: GeneratedEntrypoint,
    windowId: string,
    windowTitle: string,
    openApplication: () => void,
    focusWindow: (windowId: string) => void,
    add: (id: string, data: GeneratedEntrypoint) => void,
) {
    if (generatedEntrypoint) {
        openWindows()[windowId] = {
            id: windowId,
            appId: appId,
            title: windowTitle
        }

        const knownWindows = readWindowOrder();
        knownWindows.push(windowId);
        writeWindowOrder(knownWindows)

        add(appId, {
            ...generatedEntrypoint,
            actions: applicationActions(appId, true, openApplication, focusWindow),
            accessories: applicationAccessories(appId, true)
        })
    }
}

export function deleteOpenWindow(
    windowId: string,
    openApplication: (appId: string) => (() => void),
    focusWindow: (windowId: string) => void,
    get: (id: string) => GeneratedEntrypoint | undefined,
    add: (id: string, data: GeneratedEntrypoint) => void,
) {
    const openWindow = openWindows()[windowId];
    if (openWindow) {
        const generatedEntrypoint = get(openWindow.appId);

        delete openWindows()[windowId];

        const knownWindows = readWindowOrder();
        const newKnownWindows = knownWindows.filter(id => id != windowId)
        writeWindowOrder(newKnownWindows)

        if (generatedEntrypoint) {
            add(openWindow.appId, {
                ...generatedEntrypoint,
                actions: applicationActions(openWindow.appId, true, openApplication(openWindow.appId), focusWindow),
                accessories: applicationAccessories(openWindow.appId, true)
            })
        }
    }
}

export function openLinuxApplication(appId: string) {
    return () => {
        linux_open_application(appId)
    }
}

function focusAndSort(windowId: string, focus: (windowId: string) => void): void {
    focus(windowId)

    // TODO would probably be better if this is based on os focus events
    const knownWindows = readWindowOrder();
    const newKnownWindows = knownWindows.filter(id => id != windowId)
    newKnownWindows.unshift(windowId);
    writeWindowOrder(newKnownWindows)
}

const WINDOW_LIST_STORE_KEY = "opened-window-list";

function readWindowOrder(): string[] {
    const item = sessionStorage.getItem(WINDOW_LIST_STORE_KEY);
    if (item == null || item === "") {
        return []
    } else {
        return item.split(",")
    }
}

function writeWindowOrder(windowIds: string[]): void {
    return sessionStorage.setItem(WINDOW_LIST_STORE_KEY, windowIds.join(","));
}
