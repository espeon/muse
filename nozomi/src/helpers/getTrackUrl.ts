export default async function getTrackUrl(trackid: string): Promise<string> {
  let url = `/api/track/sign?id=${trackid}`;
  try {
    const response = await fetch(url, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
    });

    console.log("Response: ", response);

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data = await response.json();
    console.log("Track url: ", data);

    return data.url;
  } catch (error) {
    console.error("Error: ", error);
    throw error; // Re-throw the error so it can be handled by the caller
  }
}
