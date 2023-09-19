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
    op_call_event_listener(instance: InstanceSync, eventName: string): void;
}

(async () => {
    // noinspection InfiniteLoopJS
    while (true) {
        console.log("before op_get_next_pending_ui_event")
        const uiEvent = await denoCore.opAsync("op_get_next_pending_ui_event");
        switch (uiEvent.type) {
            case "ViewEvent": {
                console.log("ViewCreated")
                InternalApi.op_call_event_listener(uiEvent.widget, uiEvent.eventName)
                break;
            }
            case "ViewCreated": {
                console.log("ViewCreated")
                const view = (await import(`plugin:view?${uiEvent.viewName}`)).default;
                const { render } = await import("plugin:renderer");

                render(uiEvent.reconcilerMode, view)
                break;
            }
            case "ViewDestroyed": {
                console.log("ViewDestroyed")
                break;
            }
        }
    }
})();
