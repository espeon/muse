export type JLF = {
    lines: SyncedLines;
    richsync?: SyncedRich;
    // metadata WILL BE defined if you are creating a JLF object
    // it does NOT need to be defined if you are fetching a JLF object from db
    metadata?: SyncedMetadata;
    source: string;
    name?: string;
    message? : string;
}

export type SyncedMetadata = {
    MxmId?: string;
    ITunesId?: string;
    SpotifyId?: string;
    Artist: string;
    Title: string;
    Album: string;
    Copyright?: string;
}

export type SyncedLines = {
    lines: SyncedLine[];
    linesEnd: number;
}

/// Line of synchronised text in a song
export type SyncedLine = {
    /// ms since start of song
    time: number;
    text: string;
    translation?: string;
}

/// Synchronised rich text in a song
export type SyncedRich = {
    totalTime: number;
    sections: SyncedRichSection[];
    agents: SyncedRichAgent[];
}

export enum SyncedRichAgentType {
    Person = "person",
    Group = "group",
    Other = "other",
}

export type SyncedRichAgent = {
    type: string;
    id: string;
}

export type SyncedRichSection = {
    timeStart: number;
    timeEnd: number;
    lines: SyncedRichLine[];
}

export type SyncedRichLine = {
    timeStart: number;
    timeEnd: number;
    text: string;
    segments: SyncedRichLineSegment[];
    agent: string;
    bgVox: SyncedRichBackgroundLine | null;
}

export type SyncedRichBackgroundLine = {
    timeStart: number;
    timeEnd: number;
    text: string;
    segments: SyncedRichLineSegment[];
}

export type SyncedRichLineSegment = {
    text: string;
    timeStart: number;
    timeEnd: number;
}