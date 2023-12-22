import { FC } from "react";

// @ts-expect-error does typescript support such symbol declarations?
const denoCore = Deno[Deno.internal].core;
const InternalApi = denoCore.ops;

let latestRootUiWidget: RootUiWidget | undefined = undefined

function findWidgetWithId(widget: UiWidgetBase, widgetId: number): UiWidgetBase | undefined {
    // TODO not the most performant solution but works for now?

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
    InternalApi.op_log_trace("plugin_event_handler", `Beginning handleEvent with root widget: ${Deno.inspect(latestRootUiWidget)}`)

    if (latestRootUiWidget) {
        const widgetWithId = findWidgetWithId(latestRootUiWidget, event.widgetId);
        InternalApi.op_log_trace("plugin_event_handler", `Widget with id ${event.widgetId} is ${Deno.inspect(widgetWithId)}`)

        if (widgetWithId) {
            const property = widgetWithId.widgetProperties[event.eventName];

            InternalApi.op_log_trace("plugin_event_handler", `Event handler with name ${event.eventName} is ${Deno.inspect(property)}`)

            if (property) {
                if (typeof property === "function") {
                    property()
                } else {
                    throw new Error(`Event property has type ${typeof property}, but should be function`)
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
            case "ViewCreated": {
                try {
                    const view: FC = (await import(`gauntlet:view?${pluginEvent.viewName}`)).default;
                    const {render} = await import("gauntlet:renderer");
                    latestRootUiWidget = render(pluginEvent.reconcilerMode, view);
                } catch (e) {
                    console.error("Error occurred when rendering view", pluginEvent.viewName, e)
                }
                break;
            }
            case "ViewDestroyed": {
                break;
            }
            case "PluginCommand": {
                switch (pluginEvent.commandType) {
                    case "stop": {
                        return;
                    }
                }
            }
        }
    }
}

(async () => {
    await runLoop()
})();
