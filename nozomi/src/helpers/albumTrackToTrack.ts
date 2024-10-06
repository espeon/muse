import { Track } from "@/stores/queueStore";
import { Album, Track as AlbumTrack } from "@/types/album";
import getTrackUrl from "./getTrackUrl";

export function albumTrackToTrack(album: Album, t: AlbumTrack): Track {
  return {
    title: t.name,
    artist: album.artist.name,
    album: album.name,
    artwork:
      album.art.length > 0 ? album.art[0] : "https://i.imgur.com/moGByde.jpeg",
    stream: t.id.toString(),
  };
}
