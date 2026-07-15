# maki
## music server

### Licence
This software is released under the MIT licence.

### Audio similarity

Maki keeps analysis results by BLAKE3-256 source-file hash, rather than by
path or track ID. Moving an unchanged file or importing it again reuses its
existing profile. Exact bytes are required: MBID and ISRC never select audio
analysis because a different master can share either identifier.

Maki keeps its MIT server binary separate from the GPL-3.0-only
`maki-analyzer` worker. Similarity analysis is optional and runs after tag
scanning, without shell interpolation. The container image includes the
worker at `/usr/local/bin/maki-analyzer`.

`config/config.maki.json` controls the queue:

```json
{
  "audio_analysis_enabled": false,
  "audio_analysis_threads": 1,
  "audio_analysis_command": ["maki-analyzer"],
  "mix_analysis_enabled": false,
  "mix_analysis_url": "http://mix-analysis:5030"
}
```

Enable the optional AGPL Essentia sidecar with
`docker compose --profile mix-analysis up -d`. It has no host port and no music
volume. Maki decodes full tracks to 44.1 kHz mono PCM and streams them to the
sidecar over the Compose network. The sidecar source is maintained separately
at [espeon/muse-analysis](https://github.com/espeon/muse-analysis).

`POST /api/v1/admin/analyze?kind=similarity|mix|all&track_id=42` starts a
targeted pass (repeat `track_id` for more tracks).
`GET /api/v1/track/:id/similar?limit=10` returns local nearest neighbours using
library-wide z-score normalization and Euclidean distance.
`GET /api/v1/track/:id/mix-profile` returns 204 until an optional mix profile
exists, then returns the complete versioned sidecar payload and typed fields.

For a bounded maintenance or verification pass without starting the server,
run `kyoku analyze --kind all --limit 10` (add `--track-id 42` to target a
track and `--retry-failures` to retry prior errors). `kyoku analyze
--prune-orphaned-assets` explicitly removes cached
profiles that no song references; they are otherwise retained for re-imports.
