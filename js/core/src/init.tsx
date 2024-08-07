import { FC } from "react";
import { runCommandGenerators, runGeneratedCommand } from "./command-generator";
import { loadSearchIndex } from "./search-index";
import { clearRenderer } from "gauntlet:renderer";

// @ts-expect-error does typescript support such symbol declarations?
const denoCore: DenoCore = Deno[Deno.internal].core;
const InternalApi = denoCore.ops;

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
    InternalApi.op_log_trace("plugin_event_handler", `Handling view event: ${Deno.inspect(event)}`);
    InternalApi.op_log_trace("plugin_event_handler", `Root widget: ${Deno.inspect(latestRootUiWidget)}`);
    if (latestRootUiWidget) {
        const widgetWithId = findWidgetWithId(latestRootUiWidget, event.widgetId);
        InternalApi.op_log_trace("plugin_event_handler", `Found widget with id ${event.widgetId}: ${Deno.inspect(widgetWithId)}`)

        if (widgetWithId) {
            const property = widgetWithId.widgetProperties[event.eventName];

            InternalApi.op_log_trace("plugin_event_handler", `Found event handler with name ${event.eventName}: ${Deno.inspect(property)}`)

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

                    InternalApi.op_log_trace("plugin_event_handler", `Calling handler with arguments ${Deno.inspect(eventArgs)}`)

                    property(...eventArgs);
                } else {
                    throw new Error(`Event handler has type ${typeof property}, but should be function`)
                }
            }
        }
    }
}

async function handleKeyboardEvent(event: NotReactsKeyboardEvent) {
    InternalApi.op_log_trace("plugin_event_handler", `Handling keyboard event: ${Deno.inspect(event)}`);
    if (latestRootUiWidget) {
        const actionHandlers = findAllActionHandlers(latestRootUiWidget);

        const id = await InternalApi.fetch_action_id_for_shortcut(event.entrypointId, event.key, event.modifierShift, event.modifierControl, event.modifierAlt, event.modifierMeta);

        const actionHandler = actionHandlers.find(value => value.id === id);

        if (actionHandler) {
            actionHandler.onAction()
        }
    }
}

async function checkRequiredPreferences(entrypointId: string): Promise<boolean> {
    const pluginPreferencesRequired = InternalApi.plugin_preferences_required();
    const entrypointPreferencesRequired = InternalApi.entrypoint_preferences_required(entrypointId);

    return pluginPreferencesRequired || entrypointPreferencesRequired;
}

async function checkRequiredPreferencesAndAsk(entrypointId: string): Promise<boolean> {
    const pluginPreferencesRequired = await InternalApi.plugin_preferences_required();
    const entrypointPreferencesRequired = await InternalApi.entrypoint_preferences_required(entrypointId);

    const required = pluginPreferencesRequired || entrypointPreferencesRequired;
    if (required) {
        InternalApi.show_preferences_required_view(entrypointId, pluginPreferencesRequired, entrypointPreferencesRequired)
    }

    return required;
}

async function runLoop() {
    // runtime is stopped using tokio cancellation
    // noinspection InfiniteLoopJS
    while (true) {
        InternalApi.op_log_trace("plugin_loop", "Waiting for next plugin event...")
        const pluginEvent = await denoCore.opAsync("op_plugin_get_pending_event");
        InternalApi.op_log_trace("plugin_loop", `Received plugin event: ${Deno.inspect(pluginEvent)}`)
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
                    const { render } = await import("gauntlet:renderer");
                    latestRootUiWidget = render(pluginEvent.entrypointId, "View", <View/>);
                } catch (e) {
                    console.error("Error occurred when rendering view", pluginEvent.entrypointId, e)
                    InternalApi.show_plugin_error_view(pluginEvent.entrypointId, "View")
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

                    const command: () => void = (await import(`gauntlet:entrypoint?${pluginEvent.entrypointId}`)).default;
                    command()
                } catch (e) {
                    console.error("Error occurred when running a command", pluginEvent.entrypointId, e)
                }
                break;
            }
            case "RunGeneratedCommand": {
                try {
                    runGeneratedCommand(pluginEvent.entrypointId)
                } catch (e) {
                    console.error("Error occurred when running a generated command", pluginEvent.entrypointId, e)
                }
                break;
            }
            case "OpenInlineView": {
                const endpointId = InternalApi.op_inline_view_endpoint_id();

                if (endpointId) {
                    if (await checkRequiredPreferences(endpointId)) {
                        break;
                    }

                    try {
                        const Handler: FC<{ text: string }> = (await import(`gauntlet:entrypoint?${endpointId}`)).default;
                        const { render } = await import("gauntlet:renderer");

                        latestRootUiWidget = render(endpointId, "InlineView", <Handler text={pluginEvent.text}/>);

                        if (latestRootUiWidget.widgetChildren.length === 0) {
                            InternalApi.op_log_debug("plugin_loop", `Inline view rendered no children, clearing inline view...`)
                            InternalApi.clear_inline_view()
                        }
                    } catch (e) {
                        console.error("Error occurred when rendering inline view", e)
                    }
                }
                break;
            }
            case "ReloadSearchIndex": {
                runCommandGenerators()
                    .then(() => loadSearchIndex(false));
                break;
            }
        }
    }
}

runCommandGenerators()
    .then(() => loadSearchIndex(true));

(async () => {
    await runLoop()
})();
