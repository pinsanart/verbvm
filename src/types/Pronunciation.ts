/** An accent of the pronunciation dictionary and how many words it has. */
export interface DictionaryLanguage {
    /** BCP 47 code of the accent, such as `en-GB`. */
    variantCode: string;
    words: number;
}

/** What is installed, and the accents the dictionary can hold. */
export interface DictionaryStatus {
    installed: DictionaryLanguage[];
    supported: string[];
}
