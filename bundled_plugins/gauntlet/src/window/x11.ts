import { GeneratedCommand } from "@project-gauntlet/api/helpers";
import { application_pending_event } from "gauntlet:bridge/internal-all";
import { addOpenWindow, deleteOpenWindow, OpenWindowData } from "./shared";
import { linux_open_application } from "gauntlet:bridge/internal-linux";

export type X11WindowProtocol = "DeleteWindow" | "TakeFocus"
export type X11WindowType = "DropdownMenu" | "Dialog" | "Menu" | "Notification" | "Normal" | "PopupMenu" | "Splash" | "Toolbar" | "Tooltip" | "Utility"
export type X11WindowId = string

export type X11WindowData = {
    // x11 window id
    id: X11WindowId,
    // x11 parent window id
    parentId: X11WindowId,
    // do not show override_redirect windows in list of windows
    overrideRedirect: boolean,
    // window is visibly only if it and all of its parents are mapped
    mapped: boolean,
    // _NET_WM_NAME, or WM_NAME if that is not present
    title: string,
    // WM_CLASS
    class: string,
    // WM_CLASS
    instance: string,
    // WM_PROTOCOLS
    protocols: X11WindowProtocol[],
    // WM_HINTS
    windowGroup: X11WindowId | undefined,
    // todo icon
    // WM_TRANSIENT_FOR
    transientFor: string | undefined,
    // _NET_WM_WINDOW_TYPE
    windowTypes: X11WindowType[],
    // _KDE_NET_WM_DESKTOP_FILE or _GTK_APPLICATION_ID
    desktopFileName: string | undefined,
}

type X11ApplicationEvent = X11ApplicationEventInit
    | X11ApplicationEventCreateNotify
    | X11ApplicationEventDestroyNotify
    | X11ApplicationEventMapNotify
    | X11ApplicationEventUnmapNotify
    | X11ApplicationEventReparentNotify
    | X11ApplicationEventTitlePropertyNotify
    | X11ApplicationEventClassPropertyNotify
    | X11ApplicationEventHintsPropertyNotify
    | X11ApplicationEventProtocolsPropertyNotify
    | X11ApplicationEventTransientForPropertyNotify
    | X11ApplicationEventWindowTypePropertyNotify
    | X11ApplicationEventDesktopFileNamePropertyNotify;


type X11ApplicationEventInit = {
    type: "Init",
    id: X11WindowId,
    parent_id: X11WindowId,
    override_redirect: boolean,
    mapped: boolean,
};

type X11ApplicationEventCreateNotify = {
    type: "CreateNotify",
    id: X11WindowId,
    parent_id: X11WindowId,
    override_redirect: boolean,
};

type X11ApplicationEventDestroyNotify = {
    type: "DestroyNotify",
    id: X11WindowId,
}

type X11ApplicationEventMapNotify = {
    type: "MapNotify",
    id: X11WindowId,
};

type X11ApplicationEventUnmapNotify = {
    type: "UnmapNotify",
    id: X11WindowId,
};

type X11ApplicationEventReparentNotify = {
    type: "ReparentNotify",
    id: X11WindowId,
};

type X11ApplicationEventTitlePropertyNotify = {
    type: "TitlePropertyNotify",
    id: X11WindowId,
    title: string
};

type X11ApplicationEventClassPropertyNotify = {
    type: "ClassPropertyNotify",
    id: X11WindowId,
    class: string,
    instance: string
};

type X11ApplicationEventHintsPropertyNotify = {
    type: "HintsPropertyNotify",
    id: X11WindowId,
    window_group: X11WindowId | undefined,
};

type X11ApplicationEventProtocolsPropertyNotify = {
    type: "ProtocolsPropertyNotify",
    id: X11WindowId,
    protocols: X11WindowProtocol[],
};

type X11ApplicationEventTransientForPropertyNotify = {
    type: "TransientForPropertyNotify",
    id: X11WindowId,
    transient_for: X11WindowId | undefined,
};

type X11ApplicationEventWindowTypePropertyNotify = {
    type: "WindowTypePropertyNotify",
    id: X11WindowId,
    window_types: X11WindowType[]
};

type X11ApplicationEventDesktopFileNamePropertyNotify = {
    type: "DesktopFileNamePropertyNotify",
    id: X11WindowId,
    desktop_file_name: string
};

function openApplication(appId: string) {
    return () => {
        linux_open_application(appId)
    }
}

export function applicationEventLoopX11(
    openWindows: Record<string, OpenWindowData>,
    focusWindow: (windowId: string) => void,
    add: (id: string, data: GeneratedCommand) => void,
    get: (id: string) => GeneratedCommand | undefined,
    getAll: () => { [id: string]: GeneratedCommand },
) {
    const windows: Record<string, X11WindowData> = {};

    // noinspection ES6MissingAwait
    (async () => {
        // noinspection InfiniteLoopJS
        while (true) {
            const applicationEvent = await application_pending_event() as X11ApplicationEvent;
            switch (applicationEvent.type) {
                case "Init": {
                    windows[applicationEvent.id] = {
                        id: applicationEvent.id,
                        parentId: applicationEvent.parent_id,
                        overrideRedirect: applicationEvent.override_redirect,
                        mapped: applicationEvent.mapped,
                        class: "",
                        instance: "",
                        protocols: [],
                        title: "",
                        transientFor: undefined,
                        windowGroup: undefined,
                        windowTypes: [],
                        desktopFileName: undefined,
                    }
                    break;
                }
                case "CreateNotify": {
                    windows[applicationEvent.id] = {
                        id: applicationEvent.id,
                        parentId: applicationEvent.parent_id,
                        overrideRedirect: applicationEvent.override_redirect,
                        mapped: false,
                        class: "",
                        instance: "",
                        protocols: [],
                        title: "",
                        transientFor: undefined,
                        windowGroup: undefined,
                        windowTypes: [],
                        desktopFileName: undefined,
                    }
                    break;
                }
                case "DestroyNotify": {
                    delete windows[applicationEvent.id]

                    deleteOpenWindow(openWindows, applicationEvent.id, openApplication, focusWindow, get, add)

                    break;
                }
                case "MapNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.mapped = true;

                        validateAndAddOpenWindow(window, openWindows, windows, openApplication, focusWindow, add, getAll)
                    }

                    break;
                }
                case "UnmapNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.mapped = false;

                        deleteOpenWindow(openWindows, applicationEvent.id, openApplication, focusWindow, get, add)
                    }

                    break;
                }
                case "ReparentNotify": {
                    // for Dolphin FileManger map event doesn't seem to be fired, does reparent imply map?
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.mapped = true;

                        validateAndAddOpenWindow(window, openWindows, windows, openApplication, focusWindow, add, getAll)
                    }

                    break;
                }
                case "TitlePropertyNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.title = applicationEvent.title;

                        validateAndAddOpenWindow(window, openWindows, windows, openApplication, focusWindow, add, getAll)
                    }

                    break;
                }
                case "ClassPropertyNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.class = applicationEvent.class;
                        window.instance = applicationEvent.instance;

                        validateAndAddOpenWindow(window, openWindows, windows, openApplication, focusWindow, add, getAll)
                    }

                    break;
                }
                case "HintsPropertyNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.windowGroup = applicationEvent.window_group;

                        validateAndAddOpenWindow(window, openWindows, windows, openApplication, focusWindow, add, getAll)
                    }

                    break;
                }
                case "ProtocolsPropertyNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.protocols = applicationEvent.protocols;

                        validateAndAddOpenWindow(window, openWindows, windows, openApplication, focusWindow, add, getAll)
                    }

                    break;
                }
                case "TransientForPropertyNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.transientFor = applicationEvent.transient_for;

                        validateAndAddOpenWindow(window, openWindows, windows, openApplication, focusWindow, add, getAll)
                    }

                    break;
                }
                case "WindowTypePropertyNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.windowTypes = applicationEvent.window_types;

                        validateAndAddOpenWindow(window, openWindows, windows, openApplication, focusWindow, add, getAll)
                    }

                    break;
                }
                case "DesktopFileNamePropertyNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.desktopFileName = applicationEvent.desktop_file_name;

                        validateAndAddOpenWindow(window, openWindows, windows, openApplication, focusWindow, add, getAll)
                    }

                    break;
                }
            }
        }
    })()
}

function validateAndAddOpenWindow(
    window: X11WindowData,
    openWindows: Record<string, OpenWindowData>,
    windows: Record<string, X11WindowData>,
    openApplication: (appId: string) => (() => void),
    focusWindow: (windowId: string) => void,
    add: (id: string, data: GeneratedCommand) => void,
    getAll: () => { [id: string]: GeneratedCommand },
) {

    if (window.overrideRedirect) {
        return;
    }

    if (window.transientFor != undefined) {
        return;
    }

    if (!window.mapped) {
        return;
    }

    if (!window.windowTypes.includes("Normal")) {
        return;
    }

    let parentWindow = windows[window.parentId]
    while (parentWindow != undefined) {
        if (!parentWindow.mapped) {
            return;
        }

        parentWindow = windows[parentWindow.parentId]
    }

    function process(appId: string) {
        const generatedEntrypoint = generated[appId];

        if (generatedEntrypoint) {
            addOpenWindow(
                appId,
                generatedEntrypoint,
                window,
                openWindows,
                openApplication(appId),
                focusWindow,
                add,
            )
        }
    }

    const generated = getAll();

    let appId = window.desktopFileName;
    if (appId) {
        process(appId)
        return;
    }

    const startupWmClassToAppId = Object.fromEntries(
        Object.entries(generated)
            .map(([appId, generated]): [string | undefined, string] => [((generated as any)["__linux__"]).startupWmClass, appId])
            .filter((val): val is [string, string]  => {
                const [wmClass, _appId] = val
                return wmClass != undefined
            })
    );

    const appIdFromWmClassInstance = startupWmClassToAppId[window.instance];
    if (appIdFromWmClassInstance) {
        process(appIdFromWmClassInstance)
        return;
    }

    const appIdFromWmClass = startupWmClassToAppId[window.class];
    if (appIdFromWmClass) {
        process(appIdFromWmClass)
        return;
    }

    const wmClassInstanceAsAppId = window.instance;
    if (wmClassInstanceAsAppId) {
        process(wmClassInstanceAsAppId)
        return;
    }

    const wmClassAsAppId = window.class;
    if (wmClassAsAppId) {
        process(wmClassAsAppId)
        return;
    }

    // https://nicolasfella.de/posts/importance-of-desktop-file-mapping/
    // TODO do the rest of heuristics
    //   OR just tell users to use wayland?
    // https://github.com/KDE/plasma-workspace/blob/e2cf987971088640a149d871bcdfe63fa2aae855/libtaskmanager/xwindowtasksmodel.cpp#L519
    // https://github.com/GNOME/gnome-shell/blob/8fbaa5e55a8d65454c4d2a6f53ceb8bcaa687af5/src/shell-window-tracker.c#L390
}