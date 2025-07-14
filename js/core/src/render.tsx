import {
    fetch_action_id_for_shortcut,
    op_log_trace,
    hide_window
} from "ext:core/ops";
import { clearRenderer, rerender, render } from "ext:gauntlet/renderer.js";
import type { FC } from "react";

let latestRootUiWidget: UiWidget | undefined = undefined
let latestRootUiRenderLocation: RenderLocation | undefined = undefined

export function renderView(entrypointId: string, entrypointName: string, View: FC) {
    latestRootUiRenderLocation = "View";
    latestRootUiWidget = render(entrypointId, entrypointName, "View", <View/>);
}

export function renderInlineView(entrypointId: string, entrypointName: string, Handler: FC<{ text: string }>, text: string) {
    switch (latestRootUiRenderLocation) {
        case "InlineView": {
            rerender(<Handler text={text}/>);
            break
        }
        default: {
            latestRootUiRenderLocation = "InlineView";
            latestRootUiWidget = render(entrypointId, entrypointName, "InlineView", <Handler text={text}/>);
            break
        }
    }
}

export function closeView() {
    latestRootUiRenderLocation = undefined;
    clearRenderer()
}

export async function handlePluginViewKeyboardEvent(entrypointId: string, key: string, modifierShift: boolean, modifierControl: boolean, modifierAlt: boolean, modifierMeta: boolean) {
    if (latestRootUiWidget) {
        const actionHandlers = findAllActionHandlers(latestRootUiWidget);

        const id = await fetch_action_id_for_shortcut(entrypointId, key, modifierShift, modifierControl, modifierAlt, modifierMeta);

        if (id) {
            const actionHandler = actionHandlers.find(value => value.id === id);

            if (actionHandler) {
                actionHandler.onAction()
            }
        }
    }
}

function findAllActionHandlers(widget: UiWidget): { id: string, onAction: () => void }[] {
    if (widget.widgetType === "gauntlet:action") {
        const id = widget.widgetProperties["id"];
        const onAction = widget.widgetProperties["onAction"];
        if (!!id && !!onAction) {
            return [{ id, onAction }]
        } else {
            return []
        }
    }

    let result: { id: string, onAction: () => void }[] = []
    for (let widgetChild of widget.widgetChildren) {
        const actionHandler = findAllActionHandlers(widgetChild);

        result.push(...actionHandler)
    }

    return result;
}

export function handleEvent(event: ViewEvent) {
    op_log_trace("plugin_event_handler", `Handling view event: ${Deno.inspect(event)}`);
    op_log_trace("plugin_event_handler", `Root widget: ${Deno.inspect(latestRootUiWidget)}`);
    if (latestRootUiWidget) {
        const widgetWithId = findWidgetWithId(latestRootUiWidget, event.widgetId);
        op_log_trace("plugin_event_handler", `Found widget with id ${event.widgetId}: ${Deno.inspect(widgetWithId)}`)

        if (widgetWithId) {
            const property = widgetWithId.widgetProperties[event.eventName];

            op_log_trace("plugin_event_handler", `Found event handler with name ${event.eventName}: ${Deno.inspect(property)}`)

            if (property) {
                if (typeof property === "function") {

                    const eventArgs = event.eventArguments
                        .map(arg => {
                            switch (arg.type) {
                                case "Undefined": {
                                    return undefined
                                }
                                case "String": {
                                    return arg.value
                                }
                                case "Number": {
                                    return arg.value
                                }
                                case "Bool": {
                                    return arg.value
                                }
                            }
                        });

                    op_log_trace("plugin_event_handler", `Calling handler with arguments ${Deno.inspect(eventArgs)}`);

                    (async () => {
                        const result = await property(...eventArgs);

                        // special case for action results
                        if (event.eventName == "onAction") {
                            if (result?.close === true) {
                                hide_window()
                            }
                        }
                    })();
                } else {
                    throw new Error(`Event handler has type ${typeof property}, but should be function`)
                }
            }
        }
    }
}

function findWidgetWithId(widget: UiWidget, widgetId: number): UiWidget | undefined {
    if (widget.widgetId === widgetId) {
        return widget
    }

    for (let widgetChild of widget.widgetChildren) {
        const widgetWithId = findWidgetWithId(widgetChild, widgetId);
        if (widgetWithId) {
            return widgetWithId
        }
    }

    return undefined;
}
