document.addEventListener("DOMContentLoaded", function() {
  const checkboxShowHidden = document.querySelector("#chk-show-hidden-players");
  const playerRows = document.querySelectorAll("tr");

  function toggleHiddenPlayers() {
    const showHidden = checkboxShowHidden.checked;
    let index = 1;

    playerRows.forEach((playerRow) => {
      removeClassesStartingWith(playerRow, "rank-");
      const isHiddenPlayer = playerRow.classList.contains("hidden");
      if (!isHiddenPlayer || showHidden) {
        playerRow.style.display = "";
        const rankCell = playerRow.querySelector("td.rank");
        playerRow.classList.add(`rank-${index}`);
        if (rankCell) rankCell.textContent = index++;
      } else {
        playerRow.style.display = "none";
      }
    });
  }

  checkboxShowHidden.addEventListener("change", toggleHiddenPlayers);
  toggleHiddenPlayers(); // Initial state
});

function removeClassesStartingWith(element, prefix) {
  const classes = element.className.split(" ");
  const filteredClasses = classes.filter(
    (className) => !className.startsWith(prefix),
  );
  element.className = filteredClasses.join(" ");
}
