/**
 * JLF (JSON Lyrics Format) — the response shape from umi's `/lyrics` endpoint.
 *
 * Ported from the iOS (`muse/muse/Lyrics/JLF.swift`) and Android
 * (`muse-android/.../lyrics/Jlf.kt`) models. umi returns all timestamps as
 * **seconds** (floating point or string). We convert to **milliseconds**
 * internally so the lyrics view can compare against `player.currentTime * 1000`.
 */

// --------------------------------------------------------------- raw JSON shape

/** Raw JLF as it arrives from umi — times are in seconds. */
export interface RawJlf {
  lines: RawSyncedLines;
  richsync?: RawSyncedRich | null;
  metadata?: RawSyncedMetadata | null;
  source: string;
  name?: string | null;
  message?: string | null;
}

export interface RawSyncedLines {
  lines: RawSyncedLine[];
  /** seconds — number or string (umi is inconsistent). */
  linesEnd: number | string;
}

export interface RawSyncedLine {
  /** seconds */
  time: number | string;
  text?: string;
  translation?: string | null;
}

export interface RawSyncedMetadata {
  MxmId?: string;
  ITunesId?: string;
  SpotifyId?: string;
  Artist: string;
  Title: string;
  Album: string;
  Copyright?: string;
}

export interface RawSyncedRich {
  /** seconds */
  totalTime: number | string;
  sections: RawSyncedRichSection[];
  agents?: RawSyncedRichAgent[];
}

export interface RawSyncedRichAgent {
  type: string;
  id: string;
}

export interface RawSyncedRichSection {
  /** seconds */
  timeStart: number | string;
  timeEnd: number | string;
  lines: RawSyncedRichLine[];
}

export interface RawSyncedRichLine {
  /** seconds */
  timeStart: number | string;
  timeEnd: number | string;
  text: string;
  segments: RawSyncedRichLineSegment[];
  agent: string;
  bgVox?: RawSyncedRichBackgroundLine | null;
}

export interface RawSyncedRichBackgroundLine {
  /** seconds */
  timeStart: number | string;
  timeEnd: number | string;
  text: string;
  segments: RawSyncedRichLineSegment[];
}

export interface RawSyncedRichLineSegment {
  text: string;
  /** seconds */
  timeStart: number | string;
  timeEnd: number | string;
}

// --------------------------------------------------------------- parsed (ms)

/** Parsed JLF — all times converted to integer milliseconds. */
export interface Jlf {
  lines: SyncedLines;
  richsync: SyncedRich | null;
  metadata: SyncedMetadata | null;
  source: string;
  name: string | null;
  message: string | null;
}

export interface SyncedLines {
  lines: SyncedLine[];
  linesEnd: number;
}

export interface SyncedLine {
  /** ms since song start */
  time: number;
  text: string;
  translation: string | null;
  /** stable id for React keys */
  id: number;
}

export interface SyncedMetadata {
  mxmId: string | null;
  iTunesId: string | null;
  spotifyId: string | null;
  artist: string;
  title: string;
  album: string;
  copyright: string | null;
}

export interface SyncedRich {
  totalTime: number;
  sections: SyncedRichSection[];
  agents: SyncedRichAgent[];
}

export interface SyncedRichAgent {
  type: string;
  id: string;
}

export interface SyncedRichSection {
  timeStart: number;
  timeEnd: number;
  lines: SyncedRichLine[];
}

export interface SyncedRichLine {
  timeStart: number;
  timeEnd: number;
  text: string;
  segments: SyncedRichLineSegment[];
  agent: string;
  bgVox: SyncedRichBackgroundLine | null;
}

export interface SyncedRichBackgroundLine {
  timeStart: number;
  timeEnd: number;
  text: string;
  segments: SyncedRichLineSegment[];
}

export interface SyncedRichLineSegment {
  text: string;
  timeStart: number;
  timeEnd: number;
}

// --------------------------------------------------------------- conversion

function toMs(v: number | string | undefined | null): number {
  if (v == null) return 0;
  const n = typeof v === "string" ? parseFloat(v) : v;
  return Math.round((n || 0) * 1000);
}

export function parseJlf(raw: RawJlf): Jlf {
  return {
    lines: {
      lines: (raw.lines?.lines ?? []).map((l, i) => ({
        time: toMs(l.time),
        text: l.text ?? "",
        translation: l.translation ?? null,
        id: i,
      })),
      linesEnd: toMs(raw.lines?.linesEnd),
    },
    richsync: raw.richsync
      ? {
          totalTime: toMs(raw.richsync.totalTime),
          sections: (raw.richsync.sections ?? []).map((s) => ({
            timeStart: toMs(s.timeStart),
            timeEnd: toMs(s.timeEnd),
            lines: (s.lines ?? []).map((l) => ({
              timeStart: toMs(l.timeStart),
              timeEnd: toMs(l.timeEnd),
              text: l.text ?? "",
              segments: (l.segments ?? []).map((seg) => ({
                text: seg.text,
                timeStart: toMs(seg.timeStart),
                timeEnd: toMs(seg.timeEnd),
              })),
              agent: l.agent ?? "",
              bgVox: l.bgVox
                ? {
                    timeStart: toMs(l.bgVox.timeStart),
                    timeEnd: toMs(l.bgVox.timeEnd),
                    text: l.bgVox.text ?? "",
                    segments: (l.bgVox.segments ?? []).map((seg) => ({
                      text: seg.text,
                      timeStart: toMs(seg.timeStart),
                      timeEnd: toMs(seg.timeEnd),
                    })),
                  }
                : null,
            })),
          })),
          agents: raw.richsync.agents ?? [],
        }
      : null,
    metadata: raw.metadata
      ? {
          mxmId: raw.metadata.MxmId ?? null,
          iTunesId: raw.metadata.ITunesId ?? null,
          spotifyId: raw.metadata.SpotifyId ?? null,
          artist: raw.metadata.Artist,
          title: raw.metadata.Title,
          album: raw.metadata.Album,
          copyright: raw.metadata.Copyright ?? null,
        }
      : null,
    source: raw.source,
    name: raw.name ?? null,
    message: raw.message ?? null,
  };
}
