export default function getLyricStatus(
  currentTime: number,
  lyricStart: number,
  lyricEnd: number,
  offset: number = 0,
) {
  // default offset (animations look weird without this)
  offset = offset + 0.1;

  // add the offset to the current time
  currentTime = Number((currentTime + offset).toFixed(3));

  // Check if the lyric is active
  let isActive = currentTime > lyricStart && currentTime < lyricEnd;
  // Initialize variables for percentage and elapsed seconds
  let percentage = 0;
  let secondsAfterActive = 0;

  if (isActive) {
    let duration = lyricEnd - lyricStart;
    secondsAfterActive = currentTime - lyricStart;
    percentage = (secondsAfterActive / duration) * 100;
  } else if (currentTime > lyricEnd) {
    secondsAfterActive = currentTime - lyricEnd;
  }

  return {
    isActive: isActive,
    percentage: Number(percentage.toFixed(2)),
    secondsAfterActive: secondsAfterActive,
    secondsBeforeActive: lyricStart - currentTime,
  };
}
