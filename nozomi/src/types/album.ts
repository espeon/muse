import { ArtistPartial } from "./artistPartial"

export interface Album {
    id: number
    name: string
    art: string[]
    year: any
    created_at: number[]
    updated_at: any
    artist: Artist
    tracks: Track[]
  }
  
  export interface Artist {
    id: number
    name: string
    picture: string
  }
  
  export interface Track {
    id: number
    name: string
    album_artist: number
    artists: ArtistPartial[]
    plays: number
    duration: number
    liked: boolean
    last_play: any
    year: any
    number: number
    disc: number
    lossless: boolean
    created_at: number[]
    updated_at: any
    album: number
    album_name: string
    artist_name: string
  }
  