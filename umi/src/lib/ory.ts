import { Configuration, FrontendApi } from "@ory/client";

export const frontend = new FrontendApi(
  new Configuration({
    basePath: process.env.NEXT_PUBLIC_KRATOS_PUBLIC_ENDPOINT,
    baseOptions: {
      withCredentials: true, // include browser cookies
    },
  })
);
