import { FC } from "react";

// @ts-expect-error does typescript support such symbol declarations?
const denoCore = Deno[Deno.internal].core;
const InternalApi = denoCore.ops;

const run_loop = async () => {
    while (true) {
        InternalApi.op_log_trace("plugin_loop", "Waiting for next plugin event...")
        const pluginEvent = await denoCore.opAsync("op_plugin_get_pending_event");
        InternalApi.op_log_trace("plugin_loop", `Received plugin event: ${Deno.inspect(pluginEvent)}`)
        switch (pluginEvent.type) {
            case "ViewEvent": {
                try {
                    InternalApi.op_react_call_event_listener(pluginEvent.widget, pluginEvent.eventName)
                } catch (e) {
                    console.error("Error occurred when receiving event to handle", e)
                }
                break;
            }
            case "ViewCreated": {
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
    await run_loop()
})();
