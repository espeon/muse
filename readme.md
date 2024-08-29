# muse music platform

Muse is your personal music platform: a self-hosted server for *your* music collection, with a simple, intuitive interface that's easy to navigate.
It gives you the freedom to listen to your music however you want, from anywhere, on any device with a web browser.

## Features
- Free and Open Source (MIT License)
- Fine with large collections!
- Works on Linux, Windows, and MacOS
- Supports MP3, WAV, FLAC, and AIFF files
- Multi-user support via OIDC
- Low resource usage

## Screenshots
<p align="left">
    <img height="550" src="/assets/album_desktop.png">
    <img height="550" src="/assets/lyrics_desktop.png">
    <img height="550" src="/assets/album_mobile.png">
</p>

## Deployment instructions

- docker compose is the only supported deployment method currently.

- Use the docker-compose.prod.yaml file for production deployments. Rename the .env.example file to .env and fill in the values, then run `docker compose -f docker-compose.prod.yaml up -d`.

## Dev notes

### includes:
- honoka (sōzu control plane) (eventually)
- maki (music server)
- nozomi (frontend)
- umi (lyrics) - (not public)
- eli (remote control server) for "muse remote" feature

### what's next? (hopefully)

- music recommendation engine
- music-specific analytics

### wanna contribute? here are some things to consider!

- [GitMoji](https://gitmoji.dev) is heavily recommended. It helps with autogenerating things like release notes.

- Umi shouldn't be necessary to run anything, it's just an add-on. The frontend should fail silently if it can't connect to Umi.

- If you want to just develop a certain part, you should be able to run 'production' versions of everything via the docker-compose file.
