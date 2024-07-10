import { AlbumPartial, AlbumPartials } from "./albumPartial";

export interface Artist {
    id: number;
    name: string;
    picture: string;
    tags: string;
    bio: string;
    created_at: number[];
    updated_at: any;
    albums: AlbumPartial[];
}