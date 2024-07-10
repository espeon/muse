export interface ArtistPartials {
    artists: ArtistPartial[];
    limit: number;
    offset: number;
}

export interface ArtistPartial {
    id: number;
    name: string;
    picture: string;
    num_albums: number;
}