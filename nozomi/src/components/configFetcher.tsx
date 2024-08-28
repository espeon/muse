"use client";
import { useEffect } from "react";
import { ConfigEndpoints, useConfig } from "@/stores/configStore";

export const ConfigFetcher = () => {
  const { makiExternalBaseURL, umiBaseURL, setEndpoints } = useConfig();

  useEffect(() => {
    if (!makiExternalBaseURL || !umiBaseURL) {
      fetch("/api/config")
        .then((res) => res.json())
        .then((data: ConfigEndpoints) => {
          setEndpoints(data);
        });
    }
  }, []);

  return null;
};
