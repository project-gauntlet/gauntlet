import { FC } from "react";
import { generatedCommandSearchIndex, runCommandGenerators, runGeneratedCommand } from "./command-generator";
import { loadSearchIndex } from "./search-index";

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


async function runLoop() {
    while (true) {
        InternalApi.op_log_trace("plugin_loop", "Waiting for next plugin event...")
        const pluginEvent = await denoCore.opAsync("op_plugin_get_pending_event");
        InternalApi.op_log_trace("plugin_loop", `Received plugin event: ${Deno.inspect(pluginEvent)}`)
        switch (pluginEvent.type) {
            case "ViewEvent": {
                try {
                    handleEvent(pluginEvent)
                } catch (e) {
                    console.error("Error occurred when receiving event to handle", e)
                }
                break;
            }
            case "OpenView": {
                try {
                    const View: FC = (await import(`gauntlet:entrypoint?${pluginEvent.entrypointId}`)).default;
                    const { render } = await import("gauntlet:renderer");
                    latestRootUiWidget = render(pluginEvent.frontend, pluginEvent.entrypointId, "View", <View/>);
                } catch (e) {
                    console.error("Error occurred when rendering view", pluginEvent.entrypointId, e)
                }
                break;
            }
            case "RunCommand": {
                try {
                    await import(`gauntlet:entrypoint?${pluginEvent.entrypointId}`)
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
                const endpoint_id = InternalApi.op_inline_view_endpoint_id();

                if (endpoint_id) {
                    try {
                        const Handler: FC<{ text: string }> = (await import(`gauntlet:entrypoint?${endpoint_id}`)).default;
                        const { render } = await import("gauntlet:renderer");

                        latestRootUiWidget = render("default", endpoint_id, "InlineView", <Handler text={pluginEvent.text}/>);

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
            case "PluginCommand": {
                switch (pluginEvent.commandType) {
                    case "stop": {
                        return;
                    }
                }
                break;
            }
            case "ReloadSearchIndex": {
                await loadSearchIndex();
                break;
            }
        }
    }
}

await runCommandGenerators();

await loadSearchIndex();

(async () => {
    await runLoop()
})();
