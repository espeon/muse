import { AlbumPartial } from "./albumPartial";

export type Home = HomeRow[];

export type HomeRowType = "album" | "artist" | "track";

export interface HomeRow {
  name: string;
  albums: AlbumPartial[];
  row_type: HomeRowType;
}
