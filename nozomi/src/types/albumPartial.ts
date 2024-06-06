import { Artist } from "./album"

export interface AlbumPartials {
    albums: AlbumPartial[]
    limit: number
    offset: number
  }
export interface AlbumPartial {
  id: number
  name: string
  art: string[]
  year?: number
  count: number
  artist: Artist
}