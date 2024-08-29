export async function GET() {
  const config = {
    // likely unneeded
    internalMakiBaseURL: process.env.INTERNAL_MAKI_BASE_URL,
    externalMakiBaseURL: process.env.EXTERNAL_MAKI_BASE_URL,
    umiBaseURL: process.env.UMI_BASE_URL,
  };
  return new Response(JSON.stringify(config));
}
