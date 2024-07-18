
interface Config {
  makiBaseURL: string
  umiBaseURL: string
}

export async function GET(){
    const config = {
      makiBaseURL: process.env.MAKI_BASE_URL,
      umiBaseURL: process.env.UMI_BASE_URL
    }
    return new Response(JSON.stringify(config))
  }