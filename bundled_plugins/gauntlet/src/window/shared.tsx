import {
    GeneratedEntrypoint,
    GeneratedEntrypointAccessory,
    GeneratedEntrypointAction,
    showHud
} from "@project-gauntlet/api/helpers";
import { linux_open_application } from "gauntlet:bridge/internal-linux";
import React from "react";
import { Action, ActionPanel, List } from "@project-gauntlet/api/components";

export function ListOfWindows({ windows, focus }: { windows: OpenWindowData[], focus: (windowId: string) => void }) {
    return (
        <List
            actions={
                <ActionPanel>
                    <Action
                        label="Focus window"
                        onAction={(id: string | undefined) => {
                            if (id) {
                                focus(id)
                                console.log("focus: " + id)
                            } else {
                                showHud("No window selected")
                                console.log("No window selected")
                            }
                        }}
                    />
                </ActionPanel>
            }
        >
            {
                windows.map(window => <List.Item key={window.id} id={window.id} title={window.title}/>)
            }
        </List>
    )
}

export type OpenWindowData = {
    id: string,
    title: string,
    appId: string
}

export const openWindows: Record<string, OpenWindowData> = {};

export function applicationActions(
    id: string,
    windowTracking: boolean,
    openApplication: () => void,
    focusWindow: (windowId: string) => void,
): GeneratedEntrypointAction[] {
    if (!windowTracking) {
        return [
            {
                label: "Open application",
                run: () => {
                    openApplication()
                },
            }
        ]
    }

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
                    const appWindowsArr = appWindows
                        .map(([_, window]) => window);

                    return <ListOfWindows windows={appWindowsArr} focus={windowId => focusWindow(windowId)}/>
                }
            }
        ]
    } else {
        return []
    }
}

export function applicationAccessories(id: string, windowTracking: boolean): GeneratedEntrypointAccessory[] {
    if (!windowTracking) {
        return []
    }

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
    generatedEntrypoint: GeneratedEntrypoint,
    windowId: string,
    windowTitle: string,
    openApplication: () => void,
    focusWindow: (windowId: string) => void,
    add: (id: string, data: GeneratedEntrypoint) => void,
) {
    if (generatedEntrypoint) {
        openWindows[windowId] = {
            id: windowId,
            appId: appId,
            title: windowTitle
        }

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
    const openWindow = openWindows[windowId];
    if (openWindow) {
        const generatedEntrypoint = get(openWindow.appId);

        delete openWindows[windowId];

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