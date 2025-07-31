import type { FC } from "react";
import { runEntrypointGenerators, runGeneratedEntrypoint, runGeneratedEntrypointAction } from "./entrypoint-generator";
import { reloadSearchIndex } from "./search-index";
import {
    closeView,
    handleEvent,
    handlePluginViewKeyboardEvent, popMainView,
    renderInlineView,
    renderView,
} from "./render";
import {
    entrypoint_preferences_required,
    get_entrypoint_preferences,
    get_plugin_preferences,
    op_entrypoint_names,
    op_inline_view_entrypoint_id,
    op_log_trace,
    op_plugin_get_pending_event,
    plugin_preferences_required,
    show_plugin_error_view,
    show_preferences_required_view
} from "ext:core/ops";


async function handleKeyboardEvent(event: NotReactsKeyboardEvent) {
    op_log_trace("plugin_event_handler", `Handling keyboard event: ${Deno.inspect(event)}`);
    switch (event.origin) {
        case "MainView": {
            runGeneratedEntrypointAction(event.entrypointId, event.key, event.modifierShift, event.modifierControl, event.modifierAlt, event.modifierMeta)
            break;
        }
        case "PluginView": {
            handlePluginViewKeyboardEvent(event.entrypointId, event.key, event.modifierShift, event.modifierControl, event.modifierAlt, event.modifierMeta)
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
    await runEntrypointGenerators();

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
                const entrypointId = pluginEvent.entrypointId
                try {
                    if (await checkRequiredPreferencesAndAsk(entrypointId)) {
                        break;
                    }

                    const view: FC = (await import(`gauntlet:entrypoint?${entrypointId}`)).default;
                    renderView(entrypointId, getEntrypointName(entrypointId), view)
                } catch (e) {
                    console.error("Error occurred when rendering view", entrypointId, e)
                    show_plugin_error_view(entrypointId, "View")
                }
                break;
            }
            case "CloseView": {
                closeView()
                break;
            }
            case "PopView": {
                const entrypointId = pluginEvent.entrypointId
                try {
                    popMainView()
                } catch (e) {
                    console.error("Error occurred when popping view", entrypointId, e)
                    show_plugin_error_view(entrypointId, "View")
                }
                break;
            }
            case "RunCommand": {
                try {
                    if (await checkRequiredPreferencesAndAsk(pluginEvent.entrypointId)) {
                        break;
                    }

                    type CommandContext<P = object, E = object> = {
                        pluginPreferences: P,
                        entrypointPreferences: E,
                    };

                    const pluginPreferences = get_plugin_preferences();
                    const entrypointPreferences = get_entrypoint_preferences(pluginEvent.entrypointId);

                    const command: (context: CommandContext) => Promise<void> | void = (await import(`gauntlet:entrypoint?${pluginEvent.entrypointId}`)).default;
                    command({ pluginPreferences, entrypointPreferences })
                } catch (e) {
                    console.error("Error occurred when running a command", pluginEvent.entrypointId, e)
                }
                break;
            }
            case "RunGeneratedEntrypoint": {
                try {
                    runGeneratedEntrypoint(pluginEvent.entrypointId, pluginEvent.actionIndex)
                } catch (e) {
                    console.error("Error occurred when running a generated command", pluginEvent.entrypointId, e)
                }
                break;
            }
            case "OpenInlineView": {
                const entrypointId = op_inline_view_entrypoint_id();

                if (entrypointId) {
                    if (await checkRequiredPreferences(entrypointId)) {
                        break;
                    }

                    try {
                        type InlineView = { text: string };
                        const handler: FC<InlineView> = (await import(`gauntlet:entrypoint?${entrypointId}`)).default;

                        renderInlineView(entrypointId, getEntrypointName(entrypointId), handler, pluginEvent.text)
                    } catch (e) {
                        console.error("Error occurred when rendering inline view", e)
                    }
                }
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

function getEntrypointName(entrypointId: string): string {
    const entrypointNames = op_entrypoint_names();
    const entrypointName = entrypointNames[entrypointId];

    if (entrypointName) {
        return entrypointName
    }

    throw new Error(`Unable to get entrypoint name for entrypoint id: ${entrypointId}`)
}
