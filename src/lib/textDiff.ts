/** How a run of characters compares: `equal` in both texts, `wrong` only in what was typed, or `missing` from it. */
export type DiffKind = "equal" | "wrong" | "missing";

export interface DiffSegment {
    text: string;
    kind: DiffKind;
}

export interface TextDiff {
    /** What was typed, with the characters that are not in the expected text marked `wrong`. */
    typed: DiffSegment[];
    /** The expected text, with the characters that were not typed marked `missing`. */
    expected: DiffSegment[];
    /** The two texts are the same once trimmed. */
    exact: boolean;
    /** How many typed characters are not in the expected text. */
    extra: number;
    /** How many expected characters were not typed. */
    missing: number;
}

/** Joins neighbours of the same kind, so a word becomes one segment and not one per letter. */
function merge(segments: DiffSegment[]): DiffSegment[] {
    const merged: DiffSegment[] = [];
    for (const segment of segments) {
        const last = merged[merged.length - 1];
        if (last && last.kind === segment.kind) last.text += segment.text;
        else merged.push({ ...segment });
    }
    return merged;
}

/**
 * Compares what was typed with the expected text character by character, using the longest
 * common subsequence so that one missing letter does not mark everything after it as wrong.
 * Both texts are trimmed. The comparison is strict: case, accents and punctuation count.
 */
export function diffText(typedText: string, expectedText: string): TextDiff {
    const typed = Array.from(typedText.trim());
    const expected = Array.from(expectedText.trim());

    // lengths[i][j]: the length of the common subsequence of typed[i..] and expected[j..].
    const lengths = Array.from({ length: typed.length + 1 }, () => new Array<number>(expected.length + 1).fill(0));
    for (let i = typed.length - 1; i >= 0; i--) {
        for (let j = expected.length - 1; j >= 0; j--) {
            lengths[i][j] =
                typed[i] === expected[j] ? lengths[i + 1][j + 1] + 1 : Math.max(lengths[i + 1][j], lengths[i][j + 1]);
        }
    }

    const typedSegments: DiffSegment[] = [];
    const expectedSegments: DiffSegment[] = [];
    let i = 0;
    let j = 0;
    while (i < typed.length || j < expected.length) {
        if (i < typed.length && j < expected.length && typed[i] === expected[j]) {
            typedSegments.push({ text: typed[i], kind: "equal" });
            expectedSegments.push({ text: expected[j], kind: "equal" });
            i++;
            j++;
        } else if (j >= expected.length || (i < typed.length && lengths[i + 1][j] >= lengths[i][j + 1])) {
            typedSegments.push({ text: typed[i], kind: "wrong" });
            i++;
        } else {
            expectedSegments.push({ text: expected[j], kind: "missing" });
            j++;
        }
    }

    const count = (segments: DiffSegment[], kind: DiffKind) =>
        segments.filter((segment) => segment.kind === kind).length;
    return {
        extra: count(typedSegments, "wrong"),
        missing: count(expectedSegments, "missing"),
        typed: merge(typedSegments),
        expected: merge(expectedSegments),
        exact: typed.length === expected.length && typed.every((char, index) => char === expected[index]),
    };
}
