document.addEventListener("DOMContentLoaded", function () {
  const chkShowHiddenPlayers = document.querySelector(
    "#chk-show-hidden-players",
  );
  const hiddenPlayersTr = document.querySelectorAll("tr.hidden");

  function toggleHiddenPlayers() {
    const isChecked = chkShowHiddenPlayers.checked;
    hiddenPlayersTr.forEach((element) => {
      element.style.display = isChecked ? "" : "none";
    });
  }

  chkShowHiddenPlayers.addEventListener("change", toggleHiddenPlayers);
  toggleHiddenPlayers(); // Initial state
});
