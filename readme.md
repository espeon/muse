# μ's music platform

## includes:
- honoka (sōzu control plane) (eventually)
- maki (music server)
- nozomi (frontend)
- umi (lyrics) - (not public)
- eli (remote control server) for "muse remote" feature

## what's next? (hopefully)

- music recommendation engine
- music-specific analytics

## Deployment instructions

- docker compose is the only supported deployment method currently.

- Use the docker-compose.prod.yaml file for production deployments. Rename the .env.example file to .env and fill in the values, then run `docker compose -f docker-compose.prod.yaml up -d`.

## wanna contribute? here are some things to consider!

- [GitMoji](https://gitmoji.dev) is heavily recommended! please use it! it actually helps and is not just a gimmick!

- Dev Ports: Maki is 3033, Umi (if you have access) is 3032, Nozomi is 3031. These should already be in the .env files.

- Umi shouldn't be necessary to run anything, it's just an add-on. The frontend should fail silently if it can't connect to Umi.

- If you want to just develop a certain part, you should be able to run 'production' versions of everything via the docker-compose file.
