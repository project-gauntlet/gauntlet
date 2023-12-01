import { FC } from "react";

// @ts-expect-error does typescript support such symbol declarations?
const denoCore = Deno[Deno.internal].core;
const InternalApi = denoCore.ops;

const run_loop = async () => {
    while (true) {
        console.log("before op_plugin_get_pending_event")
        const pluginEvent = await denoCore.opAsync("op_plugin_get_pending_event");
        switch (pluginEvent.type) {
            case "ViewEvent": {
                console.log("ViewEvent")
                try {
                    InternalApi.op_react_call_event_listener(pluginEvent.widget, pluginEvent.eventName)
                } catch (e) {
                    console.error("Error occurred when receiving event to handle", e)
                }
                break;
            }
            case "ViewCreated": {
                console.log("ViewCreated")
                try {
                    const view: FC = (await import(`plugin:view?${pluginEvent.viewName}`)).default;
                    const { render } = await import("plugin:renderer");
                    render(pluginEvent.reconcilerMode, view)
                } catch (e) {
                    console.error("Error occurred when rendering view", pluginEvent.viewName, e)
                }
                break;
            }
            case "ViewDestroyed": {
                console.log("ViewDestroyed")
                break;
            }
            case "PluginCommand": {
                switch (pluginEvent.commandType) {
                    case "stop": {
                        console.log("PluginCommand stop")
                        return;
                    }
                }
            }
        }
    }
}

(async () => {
    await run_loop()
})();
