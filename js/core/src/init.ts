import { runPluginLoop } from "gauntlet:core";

globalThis.addEventListener("unhandledrejection", (event) => {
    event.preventDefault()
    console.error("Rejected promise, reason:", event.reason);
});

(async () => {
    await runPluginLoop()
})();
