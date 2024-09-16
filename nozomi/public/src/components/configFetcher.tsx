"use client";
import { useEffect } from "react";
import { ConfigEndpoints, useConfig } from "@/stores/configStore";

console.info(
  "%c hello 👋 from nashville %c",
  "background-color: rgb(0, 9, 26); color: white; font-size: 2em;",
  "color: unset",
);
console.info(
  "%c  help me make this better? https://github.com/espeon/muse)%c",
  "color: rgb(91, 201, 222)",
  "color: unset",
);

export const ConfigFetcher = () => {
  const { externalMakiBaseURL, umiBaseURL, setEndpoints } = useConfig();

  useEffect(() => {
    // update config just in case
    fetch("/api/config")
      .then((res) => res.json())
      .then((data: ConfigEndpoints) => {
        setEndpoints(data);
      });
  }, []);

  return null;
};
