// Loaded with `defer`, so the page is parsed when this runs. Kept out of the
// HTML so the Content-Security-Policy can forbid inline script.

// Timestamps are UTC without a zone suffix, which Date would read as local
// time; mark them as UTC, then show them relative to now ("3 days ago"),
// with the full date in the viewer's time zone on hover.
const relative = new Intl.RelativeTimeFormat(navigator.language, { numeric: "auto" });
const units = [
    ["year", 365 * 24 * 3600],
    ["month", 30 * 24 * 3600],
    ["week", 7 * 24 * 3600],
    ["day", 24 * 3600],
    ["hour", 3600],
    ["minute", 60],
];
for (const elem of document.querySelectorAll(".timestamp")) {
    const utc = elem.getAttribute("datetime").trim();
    const date = new Date(/[zZ]|[+-]\d\d:?\d\d$/.test(utc) ? utc : utc + "Z");
    if (isNaN(date)) continue;
    const seconds = (date - Date.now()) / 1000;
    const [unit, size] = units.find(([, size]) => Math.abs(seconds) >= size) || ["second", 1];
    elem.textContent = unit === "second" ? relative.format(0, "second") : relative.format(Math.round(seconds / size), unit);
    elem.title = date.toLocaleString(navigator.language);
}

for (const button of document.querySelectorAll("button.copy")) {
    const label = button.querySelector("span");
    button.addEventListener("click", async () => {
        try {
            await navigator.clipboard.writeText(button.dataset.copy);
            label.textContent = "Copied";
            button.classList.add("copied");
            setTimeout(() => {
                label.textContent = "Copy";
                button.classList.remove("copied");
            }, 2000);
        } catch {
            label.textContent = "Copy failed";
        }
    });
}
