document.addEventListener("DOMContentLoaded", function () {
  const statusElem = document.getElementById("status");
  const formElem = document.querySelector("form");
  const guessElem = document.querySelector("input#time-guess");
  const dayToken = document.querySelector('input[name="day-token"]');

  formElem.addEventListener("submit", function (event) {
    event.preventDefault();
    const guessValue = guessElem.value;
    const data = {
      day: parseInt(dayToken.value),
      guess: guessValue,
    };

    fetch("/guess", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(data),
    })
      .then((response) => {
        if (response.status === 200) {
          return response.json();
        } else {
          throw new Error("got an error");
        }
      })
      .then((data) => {
        const time = getTimeLabel();
        const points = data.points;
        alert(`you got ${points} points`);
        globalThis.location.reload();
      })
      .catch((error) => {
        alert(`Invalid input dawg: ${error}`);
        // statusElem.innerHTML =
        //   `[${time}] ERROR: Failed to upload data to server: ` + error.message;
      });
  });
});

function getTimeLabel() {
  const now = new Date();
  const h = String(now.getHours()).padStart(2, "0");
  const m = String(now.getMinutes()).padStart(2, "0");
  const s = String(now.getSeconds()).padStart(2, "0");
  return `${h}:${m}:${s}`;
}
