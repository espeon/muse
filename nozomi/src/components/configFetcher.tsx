"use client";
import { useEffect } from "react";
import { ConfigEndpoints, useConfig } from "@/stores/configStore";

export const ConfigFetcher = () => {
  const { makiExternalBaseURL, umiBaseURL, setEndpoints } = useConfig();

  useEffect(() => {
    // update config just in case
    fetch("/api/config")
      .then((res) => res.json())
      .then((data: ConfigEndpoints) => {
        console.log("Fetched config", data);
        setEndpoints(data);
      });
  }, []);

  return null;
};
