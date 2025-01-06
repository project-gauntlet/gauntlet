import { GeneratedCommand } from "@project-gauntlet/api/helpers";
import { addOpenWindow, deleteOpenWindow, openLinuxApplication } from "./shared";
import { application_x11_pending_event, linux_x11_focus_window } from "gauntlet:bridge/internal-linux";

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


export function focusX11Window(windowId: string) {
    linux_x11_focus_window(windowId)
}

export function applicationEventLoopX11(
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
            const applicationEvent = await application_x11_pending_event();
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

                    deleteOpenWindow(applicationEvent.id, openLinuxApplication, focusWindow, get, add)

                    break;
                }
                case "MapNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.mapped = true;

                        validateAndAddOpenWindow(window, windows, openLinuxApplication, focusWindow, add, getAll)
                    }

                    break;
                }
                case "UnmapNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.mapped = false;

                        deleteOpenWindow(applicationEvent.id, openLinuxApplication, focusWindow, get, add)
                    }

                    break;
                }
                case "ReparentNotify": {
                    // for Dolphin FileManger map event doesn't seem to be fired, does reparent imply map?
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.mapped = true;

                        validateAndAddOpenWindow(window, windows, openLinuxApplication, focusWindow, add, getAll)
                    }

                    break;
                }
                case "TitlePropertyNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.title = applicationEvent.title;

                        validateAndAddOpenWindow(window, windows, openLinuxApplication, focusWindow, add, getAll)
                    }

                    break;
                }
                case "ClassPropertyNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.class = applicationEvent.class;
                        window.instance = applicationEvent.instance;

                        validateAndAddOpenWindow(window, windows, openLinuxApplication, focusWindow, add, getAll)
                    }

                    break;
                }
                case "HintsPropertyNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.windowGroup = applicationEvent.window_group;

                        validateAndAddOpenWindow(window, windows, openLinuxApplication, focusWindow, add, getAll)
                    }

                    break;
                }
                case "ProtocolsPropertyNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.protocols = applicationEvent.protocols;

                        validateAndAddOpenWindow(window, windows, openLinuxApplication, focusWindow, add, getAll)
                    }

                    break;
                }
                case "TransientForPropertyNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.transientFor = applicationEvent.transient_for;

                        validateAndAddOpenWindow(window, windows, openLinuxApplication, focusWindow, add, getAll)
                    }

                    break;
                }
                case "WindowTypePropertyNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.windowTypes = applicationEvent.window_types;

                        validateAndAddOpenWindow(window, windows, openLinuxApplication, focusWindow, add, getAll)
                    }

                    break;
                }
                case "DesktopFileNamePropertyNotify": {
                    const window = windows[applicationEvent.id];
                    if (window) {
                        window.desktopFileName = applicationEvent.desktop_file_name;

                        validateAndAddOpenWindow(window, windows, openLinuxApplication, focusWindow, add, getAll)
                    }

                    break;
                }
            }
        }
    })()
}

function validateAndAddOpenWindow(
    window: X11WindowData,
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
                window.id,
                window.title,
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