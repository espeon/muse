"use client";
import NavControls from "@/components/navControls";
import { Track as AlbumTrack } from "@/types/album";
import Link from "next/link";
import { useState, useEffect, ChangeEvent } from "react";

export default function SearchPage() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<AlbumTrack[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!query) return;

    setLoading(true);
    fetch(`/api/search?query=${encodeURIComponent(query)}`)
      .then((response) => response.json())
      .then((data) => setResults(data))
      .finally(() => setLoading(false));
  }, [query]);

  const handleSearch = (event: ChangeEvent<HTMLInputElement>) => {
    event.preventDefault();
    setQuery(event.target.value);
  };

  return (
    <div>
      <input
        type="text"
        name="query"
        placeholder="Search..."
        content={query}
        onChange={(e) => setQuery(e.target.value)}
      />

      {loading && <p>Loading...</p>}
      {!loading && results.length > 0 && (
        <ul>
          {results.map((track) => (
            <li key={track.id}>
              <div>
                {track.album} by {track.album_artist}
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
