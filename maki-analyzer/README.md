# maki-analyzer

`maki-analyzer` is the audio-analysis worker used by Maki.

```sh
maki-analyzer /music/example.flac
```

It writes exactly one JSON object to standard output:

```json
{"version":2,"vector":[0.0]}
```
