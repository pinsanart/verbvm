/** The text of an HTML fragment, for previews and search. Rich fields hold sanitized HTML. */
export function plainText(html: string): string {
    if (!html.includes("<")) return html.trim();
    return (new DOMParser().parseFromString(html, "text/html").body.textContent ?? "").replace(/\s+/g, " ").trim();
}
