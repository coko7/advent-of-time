document.addEventListener("DOMContentLoaded", function () {
  const checkboxShowHidden = document.querySelector("#chk-show-hidden-players");
  const playerRows = document.querySelectorAll("tr");

  function toggleHiddenPlayers() {
    const showHidden = checkboxShowHidden.checked;
    let index = 1;
    playerRows.forEach((playerRow) => {
      const isHiddenPlayer = playerRow.classList.contains("hidden");
      if (!isHiddenPlayer || showHidden) {
        playerRow.style.display = "";
        const rankCell = playerRow.querySelector("td.rank");
        if (rankCell) rankCell.textContent = index++;
      } else {
        playerRow.style.display = "none";
      }
    });
  }

  checkboxShowHidden.addEventListener("change", toggleHiddenPlayers);
  toggleHiddenPlayers(); // Initial state
});
