# nowplaying

tiny websocket server that broadcasts your currently playing song

## getting started

0. [apply](https://www.last.fm/api/account/create) for a Last.fm API key
1. set the `LASTFM_USER` and `LASTFM_API_KEY` environment variables ([example](/.env.example))
2. run `nowplaying` however you please ([docker-compose example](/docker-compose.yml))
3. connect to nowplaying's websocket at `ws://{host}:{port}/ws`

## example payload

```json
{
  "mbid": "41b67131-a415-4254-91ab-a5fa6e8155b5",
  "name": "Tell Me (U Want It)",
  "url": "https://www.last.fm/music/underscores/_/Tell+Me+(U+Want+It)",
  "artist": {
    "mbid": "ab1a3f85-e0ea-470a-af5c-175447ae774c",
    "name": "underscores"
  },
  "album": {
    "mbid": "fa198feb-af0b-4d07-a65f-3f53fb8bff46",
    "name": "U"
  },
  "image": {
    "small": "https://lastfm.freetls.fastly.net/i/u/34s/82573426631c6de14959f4753eafe666.jpg",
    "medium": "https://lastfm.freetls.fastly.net/i/u/64s/82573426631c6de14959f4753eafe666.jpg",
    "large": "https://lastfm.freetls.fastly.net/i/u/174s/82573426631c6de14959f4753eafe666.jpg",
    "extralarge": "https://lastfm.freetls.fastly.net/i/u/300x300/82573426631c6de14959f4753eafe666.jpg"
  },
  "streamable": false,
  "nowplaying": true
}
```
