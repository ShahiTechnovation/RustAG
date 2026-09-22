import { RustagClient } from "@rustag/sdk";

const baseUrl = process.env.NEXT_PUBLIC_RUSTAG_API_URL ?? "http://localhost:9000";

/** Shared SDK client pointed at the running stagenet's REST API. */
export const client = new RustagClient({ baseUrl });
