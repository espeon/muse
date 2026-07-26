/**
 * LLM-based lyrics translation service.
 * Ported from the iOS LLMTranslationService — uses any OpenAI-compatible API
 * (OpenRouter, OpenAI, etc.) to translate lyric lines in batch.
 *
 * Configuration is stored in localStorage (set via the Settings page).
 */

const API_KEY_KEY = "muse-web.llm.apiKey";
const BASE_URL_KEY = "muse-web.llm.baseURL";
const MODEL_KEY = "muse-web.llm.model";
const TARGET_LANG_KEY = "muse-web.llm.targetLang";

export function getLLMConfig() {
  return {
    apiKey: localStorage.getItem(API_KEY_KEY) ?? "",
    baseURL:
      localStorage.getItem(BASE_URL_KEY) ?? "https://openrouter.ai/api/v1",
    model: localStorage.getItem(MODEL_KEY) ?? "openai/gpt-4o-mini",
    targetLang: localStorage.getItem(TARGET_LANG_KEY) ?? "English",
  };
}

export function isLLMConfigured(): boolean {
  return !!localStorage.getItem(API_KEY_KEY);
}

// --------------------------------------------------------------- cache

interface CacheEntry {
  sourceHash: string;
  translations: string[];
}

function sourceHash(texts: string[]): string {
  // Simple hash — not cryptographic, just for cache invalidation
  let h = 0;
  for (const text of texts) {
    for (let i = 0; i < text.length; i++) {
      h = ((h << 5) - h + text.charCodeAt(i)) | 0;
    }
    h = ((h << 5) - h + 0) | 0; // separator
  }
  return String(h);
}

function cacheKey(
  trackId: string,
  texts: string[],
  target: string,
  model: string,
): string {
  return `muse-web.translation.${trackId}.${target}.${model}.${sourceHash(texts)}`;
}

function getCached(
  trackId: string,
  texts: string[],
  target: string,
  model: string,
): string[] | null {
  try {
    const raw = localStorage.getItem(
      cacheKey(trackId, texts, target, model),
    );
    if (!raw) return null;
    const entry: CacheEntry = JSON.parse(raw);
    if (entry.sourceHash !== sourceHash(texts)) return null;
    return entry.translations;
  } catch {
    return null;
  }
}

function setCached(
  translations: string[],
  trackId: string,
  texts: string[],
  target: string,
  model: string,
) {
  try {
    const entry: CacheEntry = {
      sourceHash: sourceHash(texts),
      translations,
    };
    localStorage.setItem(
      cacheKey(trackId, texts, target, model),
      JSON.stringify(entry),
    );
  } catch {
    // localStorage full or unavailable — non-fatal
  }
}

// --------------------------------------------------------------- API

interface ChatMessage {
  role: string;
  content: string;
}

interface ChatRequest {
  model: string;
  messages: ChatMessage[];
  temperature: number;
}

interface ChatResponse {
  choices?: { message: { content?: string } }[];
}

/**
 * Translate an array of lyric lines into the target language.
 * Empty lines are preserved as empty strings in the output.
 * Results are cached per track + source hash.
 */
export async function translateLyrics(
  texts: string[],
  trackId: string,
): Promise<string[]> {
  const config = getLLMConfig();
  if (!config.apiKey) {
    throw new Error("LLM translation is not configured. Add an API key in Settings.");
  }

  // Check cache
  const cached = getCached(
    trackId,
    texts,
    config.targetLang,
    config.model,
  );
  if (cached) return cached;

  // Separate empty and non-empty lines
  const nonEmptyIndices: number[] = [];
  const nonEmptyTexts: string[] = [];
  for (let i = 0; i < texts.length; i++) {
    if (texts[i]) {
      nonEmptyIndices.push(i);
      nonEmptyTexts.push(texts[i]);
    }
  }

  if (nonEmptyTexts.length === 0) {
    return texts;
  }

  const systemPrompt = `You are a professional lyric translator. You will receive a JSON array of lyric lines, in their original sequential order. Translate each line into ${config.targetLang}, taking into account surrounding lines for context. Try to make it flow well in the target language, even if that means deviating from a literal translation. Preserve any rhymes or poetic devices where possible, but prioritize flow, naturalness and emotional impact in the target language.

Your response must be ONLY a JSON array of strings, with exactly ${nonEmptyTexts.length} elements, in the same order as the input. No markdown, no explanation, just the array.

These are song lyrics presented in the order they are sung. Preserve that order exactly; do not rearrange, merge, or skip lines.

Example input: ["hello world", "goodbye"]
Example output: ["hola mundo", "adiós"]`;

  const body: ChatRequest = {
    model: config.model,
    messages: [
      { role: "system", content: systemPrompt },
      { role: "user", content: JSON.stringify(nonEmptyTexts) },
    ],
    temperature: 0.3,
  };

  const url =
    config.baseURL.replace(/\/+$/, "") + "/chat/completions";

  const res = await fetch(url, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${config.apiKey}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });

  if (!res.ok) {
    const errBody = await res.text();
    throw new Error(`Translation API error (${res.status}): ${errBody}`);
  }

  const data: ChatResponse = await res.json();
  const content = data.choices?.[0]?.message?.content;
  if (!content) {
    throw new Error("Translation API returned empty response");
  }

  const translated = parseResponse(content);

  if (translated.length !== nonEmptyTexts.length) {
    throw new Error(
      `Translation line count mismatch: expected ${nonEmptyTexts.length}, got ${translated.length}`,
    );
  }

  // Reassemble into full array (empty lines stay empty)
  const results = new Array(texts.length).fill("");
  for (let i = 0; i < nonEmptyIndices.length; i++) {
    results[nonEmptyIndices[i]] = translated[i];
  }

  setCached(results, trackId, texts, config.targetLang, config.model);
  return results;
}

function parseResponse(content: string): string[] {
  let cleaned = content.trim();

  // Strip markdown code fences
  if (cleaned.startsWith("```")) {
    const firstNewline = cleaned.indexOf("\n");
    if (firstNewline !== -1) {
      cleaned = cleaned.slice(firstNewline + 1);
    }
    const closingIdx = cleaned.lastIndexOf("\n```");
    if (closingIdx !== -1) {
      cleaned = cleaned.slice(0, closingIdx);
    } else if (cleaned.endsWith("```")) {
      cleaned = cleaned.slice(0, -3);
    }
    cleaned = cleaned.trim();
  }

  // Try direct JSON parse
  try {
    return JSON.parse(cleaned);
  } catch {
    // Fall through
  }

  // Try extracting JSON array from surrounding text
  const firstBracket = cleaned.indexOf("[");
  const lastBracket = cleaned.lastIndexOf("]");
  if (firstBracket !== -1 && lastBracket !== -1) {
    const jsonStr = cleaned.slice(firstBracket, lastBracket + 1);
    try {
      return JSON.parse(jsonStr);
    } catch {
      // Fall through
    }
  }

  throw new Error(`Invalid JSON from LLM: ${content.slice(0, 200)}`);
}
