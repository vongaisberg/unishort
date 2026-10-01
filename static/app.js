// Loaded with `defer`, so the page is parsed when this runs. Kept out of the
// HTML so the Content-Security-Policy can forbid inline script.

// Timestamps are UTC without a zone suffix, which Date would read as local
// time; mark them as UTC, then show them in the viewer's time zone.
for (const elem of document.querySelectorAll(".timestamp")) {
    const utc = elem.textContent.trim();
    elem.textContent = new Date(/[zZ]|[+-]\d\d:?\d\d$/.test(utc) ? utc : utc + "Z")
        .toLocaleString(navigator.language);
}

for (const input of document.querySelectorAll("input.short_url")) {
    input.addEventListener("click", () => input.select());
}

for (const button of document.querySelectorAll("img.copy")) {
    button.addEventListener("click", () => {
        navigator.clipboard.writeText(button.alt);
        button.src = "/static/check.svg";
    });
}
