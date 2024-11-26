import type { FC } from "react";
import { runCommandGenerators, runGeneratedCommand, runGeneratedCommandAction } from "./command-generator";
import { reloadSearchIndex } from "./search-index";
import { clearRenderer, render } from "ext:gauntlet/renderer.js";
import {
    clear_inline_view,
    entrypoint_preferences_required,
    fetch_action_id_for_shortcut,
    op_inline_view_endpoint_id,
    op_log_debug,
    op_log_trace,
    plugin_preferences_required,
    show_plugin_error_view,
    show_preferences_required_view,
    op_plugin_get_pending_event
} from "ext:core/ops";

let latestRootUiWidget: UiWidget | undefined = undefined

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

function handleEvent(event: ViewEvent) {
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

                    op_log_trace("plugin_event_handler", `Calling handler with arguments ${Deno.inspect(eventArgs)}`)

                    property(...eventArgs);
                } else {
                    throw new Error(`Event handler has type ${typeof property}, but should be function`)
                }
            }
        }
    }
}

async function handleKeyboardEvent(event: NotReactsKeyboardEvent) {
    op_log_trace("plugin_event_handler", `Handling keyboard event: ${Deno.inspect(event)}`);
    switch (event.origin) {
        case "MainView": {
            runGeneratedCommandAction(event.entrypointId, event.key, event.modifierShift, event.modifierControl, event.modifierAlt, event.modifierMeta)
            break;
        }
        case "PluginView": {
            if (latestRootUiWidget) {
                const actionHandlers = findAllActionHandlers(latestRootUiWidget);

                const id = await fetch_action_id_for_shortcut(event.entrypointId, event.key, event.modifierShift, event.modifierControl, event.modifierAlt, event.modifierMeta);

                if (id) {
                    const actionHandler = actionHandlers.find(value => value.id === id);

                    if (actionHandler) {
                        actionHandler.onAction()
                    }
                }
            }
            break;
        }
    }
}

async function checkRequiredPreferences(entrypointId: string): Promise<boolean> {
    const pluginPreferencesRequired = plugin_preferences_required();
    const entrypointPreferencesRequired = entrypoint_preferences_required(entrypointId);

    return pluginPreferencesRequired || entrypointPreferencesRequired;
}

async function checkRequiredPreferencesAndAsk(entrypointId: string): Promise<boolean> {
    const pluginPreferencesRequired = await plugin_preferences_required();
    const entrypointPreferencesRequired = await entrypoint_preferences_required(entrypointId);

    const required = pluginPreferencesRequired || entrypointPreferencesRequired;
    if (required) {
        show_preferences_required_view(entrypointId, pluginPreferencesRequired, entrypointPreferencesRequired)
    }

    return required;
}

export async function runPluginLoop() {
    await runCommandGenerators();

    // runtime is stopped using tokio cancellation
    // noinspection InfiniteLoopJS
    while (true) {
        op_log_trace("plugin_loop", "Waiting for next plugin event...")
        const pluginEvent = await op_plugin_get_pending_event();
        op_log_trace("plugin_loop", `Received plugin event: ${Deno.inspect(pluginEvent)}`)
        switch (pluginEvent.type) {
            case "ViewEvent": {
                try {
                    handleEvent(pluginEvent)
                } catch (e) {
                    console.error("Error occurred when receiving view event to handle", e)
                }
                break;
            }
            case "KeyboardEvent": {
                try {
                    await handleKeyboardEvent(pluginEvent)
                } catch (e) {
                    console.error("Error occurred when receiving keyboard event to handle", e)
                }
                break;
            }
            case "OpenView": {
                try {
                    if (await checkRequiredPreferencesAndAsk(pluginEvent.entrypointId)) {
                        break;
                    }

                    const View: FC = (await import(`gauntlet:entrypoint?${pluginEvent.entrypointId}`)).default;
                    latestRootUiWidget = render(pluginEvent.entrypointId, "View", <View/>);
                } catch (e) {
                    console.error("Error occurred when rendering view", pluginEvent.entrypointId, e)
                    show_plugin_error_view(pluginEvent.entrypointId, "View")
                }
                break;
            }
            case "CloseView": {
                clearRenderer()
                break;
            }
            case "RunCommand": {
                try {
                    if (await checkRequiredPreferencesAndAsk(pluginEvent.entrypointId)) {
                        break;
                    }

                    const command: () => Promise<void> | void = (await import(`gauntlet:entrypoint?${pluginEvent.entrypointId}`)).default;
                    command()
                } catch (e) {
                    console.error("Error occurred when running a command", pluginEvent.entrypointId, e)
                }
                break;
            }
            case "RunGeneratedCommand": {
                try {
                    runGeneratedCommand(pluginEvent.entrypointId, pluginEvent.actionIndex)
                } catch (e) {
                    console.error("Error occurred when running a generated command", pluginEvent.entrypointId, e)
                }
                break;
            }
            case "OpenInlineView": {
                const endpointId = op_inline_view_endpoint_id();

                if (endpointId) {
                    if (await checkRequiredPreferences(endpointId)) {
                        break;
                    }

                    try {
                        const Handler: FC<{ text: string }> = (await import(`gauntlet:entrypoint?${endpointId}`)).default;

                        latestRootUiWidget = render(endpointId, "InlineView", <Handler text={pluginEvent.text}/>);

                        if (latestRootUiWidget.widgetChildren.length === 0) {
                            op_log_debug("plugin_loop", `Inline view rendered no children, clearing inline view...`)
                            clear_inline_view()
                        }
                    } catch (e) {
                        console.error("Error occurred when rendering inline view", e)
                    }
                }
                break;
            }
            case "ReloadSearchIndex": {
                runCommandGenerators()
                break;
            }
            case "RefreshSearchIndex": {
                // noinspection ES6MissingAwait
                reloadSearchIndex(false)
                break;
            }
        }
    }
}
