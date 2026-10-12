/**
 * Plays the first review audio under `root` from the start (the whole page by default). Review
 * audio is marked with `data-review-audio`. Returns whether there was one.
 */
export function replayAudio(root: ParentNode = document): boolean {
    const audio = root.querySelector<HTMLAudioElement>("audio[data-review-audio]");
    if (!audio) return false;
    audio.currentTime = 0;
    audio.play().catch(() => undefined);
    return true;
}
