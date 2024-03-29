
export default function Command() {
    const systemMemoryInfo = Deno.systemMemoryInfo();

    console.dir(systemMemoryInfo)
}
