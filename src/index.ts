import { Server } from "socket.io";
import { isEqual } from "lodash";
import fetch from "cross-fetch";

const io = new Server(+process.env.PORT || 3000, {
  cors: {
    origin: "*",
  },
});

const lfmAPI = `http://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user=${process.env.LASTFM_USER}&api_key=${process.env.LASTFM_KEY}&format=json`;
let currentTrack = {};

const poll = () => {
  fetch(lfmAPI)
    .then((res) => res.json())
    .then((data) => {
      let newTrack = {
        artist: "",
        name: "",
        nowPlaying: false,
      };

      const trackJSON = data.recenttracks.track[0];
      newTrack.artist = trackJSON.artist["#text"];
      newTrack.name = trackJSON.name;
      trackJSON["@attr"]
        ? (newTrack.nowPlaying = true)
        : (newTrack.nowPlaying = false);

      if (!isEqual(currentTrack, newTrack)) {
        io.emit("track", newTrack);
        currentTrack = newTrack;
      }
    });
};

io.on("connection", (socket) => {
  socket.emit("track", currentTrack);
});

setInterval(poll, 1000);
