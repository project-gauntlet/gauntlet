import "plugin:renderer"

// @ts-expect-error "Deno[Deno.internal]" is not a public interface
const denoCore = Deno[Deno.internal].core;

const InternalApi: InternalApi = denoCore.ops;

type InstanceSync = UiWidget

declare interface UiWidget {
}

// TODO move all InternalApi declares to one place
// TODO import other submodules with types but without bundling
declare interface InternalApi {
    op_react_call_event_listener(instance: InstanceSync, eventName: string): void;
}

const run_loop = async () => {
    while (true) {
        console.log("before op_plugin_get_pending_event")
        const pluginEvent = await denoCore.opAsync("op_plugin_get_pending_event");
        switch (pluginEvent.type) {
            case "ViewEvent": {
                console.log("ViewEvent")
                InternalApi.op_react_call_event_listener(pluginEvent.widget, pluginEvent.eventName)
                break;
            }
            case "ViewCreated": {
                console.log("ViewCreated")
                const view = (await import(`plugin:view?${pluginEvent.viewName}`)).default;
                const { render } = await import("plugin:renderer");

                render(pluginEvent.reconcilerMode, view)
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
