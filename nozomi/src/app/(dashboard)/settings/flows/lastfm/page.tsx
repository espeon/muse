"use client";

import React, { useState, useEffect } from "react";

const LastFmConnect = () => {
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState("");
  const [token, setToken] = useState("");

  const handleConnect = async () => {
    setIsConnecting(true);
    setError("");

    try {
      const tokenResponse = await fetch("/api/connect/lastfm");
      if (!tokenResponse.ok) throw new Error("Failed to get Last.fm token");
      const { token, url } = await tokenResponse.json();

      setToken(token);

      // Open the url in a new tab
      window.open(url, "_blank");
    } catch (err) {
      console.error("Error connecting to Last.fm:", err);
      setError("Failed to connect to Last.fm. Please try again.");
    } finally {
      setIsConnecting(false);
    }
  };

  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search);
    const token = urlParams.get("token");
    if (token) {
      completeAuthentication(token);
    }
  }, []);

  const completeAuthentication = async (token: string) => {
    try {
      const sessionResponse = await fetch("/api/connect/lastfm", {
        method: "POST",
        headers: {
          "Content-Type": "application/json", // Replace with actual auth token
        },
        body: JSON.stringify({ token }),
      });

      if (!sessionResponse.ok) throw new Error("Failed to get Last.fm session");
      const sessionData = await sessionResponse.json();

      console.log("Last.fm connected successfully:", sessionData);
      // Here you can update your UI to show that the connection was successful
      // Or navigate to another page
    } catch (err) {
      console.error("Error completing Last.fm authentication:", err);
      setError("Failed to complete Last.fm authentication. Please try again.");
    }
  };

  return (
    <div className="bg-white shadow-md rounded-lg p-6">
      <h2 className="text-2xl font-semibold mb-4 text-gray-700">
        Connect to Last.fm
      </h2>
      {error && <p className="text-red-500 mb-4">{error}</p>}
      {token ? (
        <button
          onClick={() => completeAuthentication(token)}
          className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 transition-colors"
        >
          Verify connection
        </button>
      ) : (
        <>
          <p className="text-gray-500 mb-2">
            You will be redirected to Last.fm to authorize the connection. Come
            back here once you're done.
          </p>
          <button
            onClick={handleConnect}
            disabled={isConnecting}
            className="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700 transition-colors disabled:bg-gray-400"
          >
            {isConnecting ? "Connecting..." : "Connect to Last.fm"}
          </button>
        </>
      )}
    </div>
  );
};

export default LastFmConnect;
