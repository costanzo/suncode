import { catalogResponse } from "../../../../../../lib/updates/http.ts";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

type Context = { params: Promise<{ channel: string; target: string }> };

export async function GET(request: Request, context: Context) {
  const { channel, target } = await context.params;
  return catalogResponse(request, channel, target);
}

export const HEAD = GET;
