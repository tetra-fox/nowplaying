const request = require("request");
const io = require("socket.io")();
const isEqual = require("lodash.isequal");

const lfmAPI = `http://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user=${process.env.LASTFM_USER}&api_key=${process.env.LASTFM_KEY}&format=json`;
let track = {};

const poll = () => {
  request.get({ url: lfmAPI, json: true }, (err, res, body) => {
    let newTrack = {};

    const trackJSON = body.recenttracks.track[0];
    newTrack.artist = trackJSON.artist["#text"];
    newTrack.name = trackJSON.name;
    trackJSON["@attr"]
      ? (newTrack.nowPlaying = true)
      : (newTrack.nowPlaying = false);

    if (!isEqual(track, newTrack)) {
      io.emit("track", newTrack);
      track = newTrack;
    }
  });
};

io.on("connection", function(socket) {
  socket.emit("track", track);
});

io.listen(process.env.PORT);

setInterval(poll, 1000);
