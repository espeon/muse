# maki
## music server

note: this is absolutely not done by any means. anything can and will change.
### API
#### System
|Done|Path|Description|Queries/Notes
|-|-|-|-|
[ ]|`GET /system/info`|Get stats about the server|N/A|

#### Track
|Done|Path|Description|Queries/Notes
|-|-|-|-|
[ ]|`GET /track`|Get information about a track|`?skip=:int&limit:int` - used for paging|
[ ]|`GET /track/:id`|Get information about a track|N/A|
[ ]|`GET /track/:id/stream`|Stream a track|`?transcode=:bool` - transcode the song or not|
[ ]|`GET /track/:scrobble`|Scrobble a track|`?finished=:bool` - mark the track as finished|
[ ]|`GET /track/:id/like`|Get whether you liked the track|`?liked=:bool` - toggle song likage|

#### Album
|Done|Path|Description|Queries/Notes
|-|-|-|-|
[ ]|`GET /album/`|Get all albums|`?skip=:int`/`?index=:int` - which index to start at (used for paging)|
||||`?limit=:int` - the max albums the endpoint will return, defaults to 10
||||`?artist=:string` - self-explanatory
[x]|`GET /album/:id`|Get information about an album|N/A|
[ ]|`GET /album/:id/art`|Get an album's art|N/A|

#### Artist
|Done|Path|Description|Queries/Notes
|-|-|-|-|
[ ]|`GET /artist/`|Get all artists|`?skip=:int`/`?index=:int` - which index to start at (used for paging)|
||||`?limit=:int` - the max artists the endpoint will return, defaults to 10
[ ]|`GET /artist/:id`|Get information about an artist|N/A|

#### Search
|Done|Path|Description|Queries/Notes
|-|-|-|-|
[ ]|`GET /search`|Get information about a track|`?q=:string` - search for your query!|
||||Note: returns all categories

### Licence
This software is released under the MIT licence.
