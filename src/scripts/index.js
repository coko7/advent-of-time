document.addEventListener("DOMContentLoaded", function () {
  const disableJsBlockerMsg = document.getElementById("js-block-detect");
  disableJsBlockerMsg.remove();

  updateTimeLeftSpan();
  setInterval(() => updateTimeLeftSpan(), 1000);
});

function updateTimeLeftSpan() {
  const timeLeftSpan = document.querySelector("span#time-left");

  const timeLeft = timeLeftUntilNext6CET();
  const timeLeftText = `${timeLeft.hours}:${timeLeft.minutes}:${timeLeft.seconds}`;
  timeLeftSpan.textContent = timeLeftText;
}

function timeLeftUntilNext6CET() {
  const now = new Date();
  const nowUtc = new Date(now.toISOString());

  const year = nowUtc.getUTCFullYear();
  const month = nowUtc.getUTCMonth();
  const day = nowUtc.getUTCDate();

  // Calculate today's 6:00 AM CET in UTC (6:00 AM CET = 5:00 AM UTC)
  let targetUtc = new Date(Date.UTC(year, month, day, 5, 0, 0));

  // If current time is past today's 6:00 AM CET, target is tomorrow 6:00 AM CET
  if (nowUtc >= targetUtc) {
    targetUtc = new Date(Date.UTC(year, month, day + 1, 5, 0, 0));
  }

  const diffMs = targetUtc - nowUtc;
  const diffSeconds = Math.floor(diffMs / 1000);

  const hours = Math.floor(diffSeconds / 3600)
    .toString()
    .padStart(2, "0");
  const minutes = Math.floor((diffSeconds % 3600) / 60)
    .toString()
    .padStart(2, "0");
  const seconds = (diffSeconds % 60).toString().padStart(2, "0");

  return { milliseconds: diffMs, hours, minutes, seconds };
}
