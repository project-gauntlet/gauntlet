// @ts-expect-error
const denoCore: DenoCore = Deno[Deno.internal].core;
const InternalApi: InternalApi = denoCore.ops;

interface InternalApi {
    open_settings(): void
}

export default function Default(): void {
    InternalApi.open_settings()
}
