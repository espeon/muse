export async function getRefreshToken(refreshToken: string) {
    // Get the access token
    const data = await fetch(
      `${process.env.INTERNAL_MAKI_BASE_URL}/auth/refresh`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${refreshToken}`,
        },
      },
    )
      .then((res) => res.json())
      .catch((err) => {
        console.error("Error refreshing token:", err.message);
        return { success: false };
      });
  
    console.log("Refresh token response:", data);
    return data;
  }