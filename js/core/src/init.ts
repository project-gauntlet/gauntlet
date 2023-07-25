// @ts-expect-error "Deno[Deno.internal]" is not a public interface
const denoCore = Deno.core;

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
        const uiEvent = await denoCore.opAsync("op_get_next_pending_ui_event");
        switch (uiEvent.type) {
            case "ViewEvent": {
                InternalApi.op_call_event_listener(uiEvent.widget, uiEvent.eventName)
                break;
            }
            case "ViewCreated": {
                const view = (await import("plugin:view")).default;
                const { render } = await import("plugin:renderer");

                render(view)
                break;
            }
            case "ViewDestroyed": {
                console.log("ViewDestroyed")
                break;
            }
        }
    }
})();
