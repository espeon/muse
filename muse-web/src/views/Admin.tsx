import { useCallback, useEffect, useState } from "react";
import { Link } from "@tanstack/react-router";
import { ArrowLeft } from "lucide-react";
import {
  applyAlbumMatch,
  fetchAlbumMatches,
  fetchEnrichmentStatus,
  fetchIdentityReviews,
  resolveIdentityReview,
  retryFailedEnrichment,
  type AlbumMatch,
  type EnrichmentStatus,
  type IdentityReview,
} from "@/lib/api";
import { Button } from "@/components/ui/button";

export function Admin() {
  const [status, setStatus] = useState<EnrichmentStatus | null>(null);
  const [reviews, setReviews] = useState<IdentityReview[]>([]);
  const [albumId, setAlbumId] = useState("");
  const [matches, setMatches] = useState<AlbumMatch[]>([]);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [nextStatus, nextReviews] = await Promise.all([
        fetchEnrichmentStatus(),
        fetchIdentityReviews(),
      ]);
      setStatus(nextStatus);
      setReviews(nextReviews);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const loadMatches = async () => {
    const id = Number(albumId);
    if (!Number.isInteger(id) || id <= 0) {
      setError("enter a positive album id");
      return;
    }
    try {
      setMatches(await fetchAlbumMatches(id));
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <div className="mx-auto max-w-3xl px-4 py-8">
      <div className="mb-6 flex items-center gap-3">
        <Link
          to="/settings"
          className="inline-flex h-9 w-9 items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-accent"
        >
          <ArrowLeft size={20} />
        </Link>
        <h1 className="text-2xl font-bold">Metadata review</h1>
      </div>
      {error && <p className="mb-4 text-sm text-destructive">{error}</p>}

      <section className="mb-8">
        <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-muted-foreground">
          Enrichment queue
        </h2>
        <div className="space-y-2 rounded-xl border border-border bg-card p-4 text-sm">
          {status ? (
            <>
              <p>
                pending {status.pending} · running {status.running} · done {status.done} · failed{" "}
                {status.failed}
              </p>
              <Button size="sm" variant="outline" onClick={() => void refresh()}>
                Refresh
              </Button>{" "}
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  void retryFailedEnrichment().then(refresh);
                }}
              >
                Retry failed
              </Button>
              {status.recent_errors.map((e) => (
                <p key={e.id} className="text-muted-foreground">
                  #{e.id} {e.kind} → {e.error}
                </p>
              ))}
            </>
          ) : (
            <p className="text-muted-foreground">Loading…</p>
          )}
        </div>
      </section>

      <section className="mb-8">
        <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-muted-foreground">
          Identity conflicts
        </h2>
        <div className="space-y-2 rounded-xl border border-border bg-card p-4 text-sm">
          {reviews.length === 0 ? <p className="text-muted-foreground">None.</p> : null}
          {reviews.map((r) => (
            <div key={r.id} className="flex items-center justify-between gap-3">
              <span className="min-w-0 break-all">
                {r.kind} ({r.target}): {r.detail}
              </span>
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  void resolveIdentityReview(r.id).then(refresh);
                }}
              >
                Dismiss
              </Button>
            </div>
          ))}
        </div>
      </section>

      <section className="mb-8">
        <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-muted-foreground">
          Album match candidates
        </h2>
        <div className="space-y-3 rounded-xl border border-border bg-card p-4 text-sm">
          <div className="flex gap-2">
            <input
              className="w-32 rounded-md border border-border bg-background px-2 py-1"
              placeholder="album id"
              value={albumId}
              onChange={(e) => setAlbumId(e.target.value)}
            />
            <Button size="sm" variant="outline" onClick={() => void loadMatches()}>
              Load
            </Button>
          </div>
          {matches.map((m) => (
            <div key={m.release_mbid} className="flex items-center justify-between gap-3">
              <span className="min-w-0 break-all">
                {m.release_mbid} · score {m.score.toFixed(2)}
              </span>
              <Button
                size="sm"
                onClick={() => {
                  void applyAlbumMatch(Number(albumId), m.release_mbid)
                    .then(loadMatches)
                    .then(refresh);
                }}
              >
                Apply
              </Button>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
