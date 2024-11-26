import { runPluginLoop } from "gauntlet:core";

globalThis.addEventListener("unhandledrejection", (event) => {
    event.preventDefault()
    console.error("Rejected promise", event);
});

(async () => {
    await runPluginLoop()
})();
