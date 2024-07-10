import { Track } from "@/stores/queueStore";
import { Album, Track as AlbumTrack } from "@/types/album";

export function albumTrackToTrack(album: Album, t: AlbumTrack): Track {
    return {
      title: t.name,
      artist: album.artist.name,
      album: album.name,
      artwork:
        album.art.length > 0
          ? `${process.env.NEXT_PUBLIC_MAKI_BASE_URL}/art/${album.art[0]}`
          : "https://i.imgur.com/moGByde.jpeg",
      stream: `${process.env.NEXT_PUBLIC_MAKI_BASE_URL}/track/${t.id}/stream`,
    }
  }