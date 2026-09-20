import { useState } from "react";
import {
    ArrowDown,
    ArrowUp,
    CaseSensitive,
    ChevronDown,
    Copy,
    LucideIcon,
    Mic,
    PencilLine,
    Trash2,
} from "lucide-react";
import { Flashcard, VocabularyFlashcard, PhoneticFlashcard, OrthographicFlashcard } from "../types/Flashcard";
import { RichInputField } from "./RichInputField";
import { PlainInputField } from "./PlainInputField";
import { AudioField } from "./AudioField";
import { IconButton } from "./IconButton";

const TYPE_STYLES: Record<
    Flashcard["type"],
    { icon: LucideIcon; label: string; accent: string; badgeBg: string; badgeText: string }
> = {
    vocabulary: {
        icon: CaseSensitive,
        label: "Vocabulary",
        accent: "bg-vocabulary",
        badgeBg: "bg-vocabulary/15",
        badgeText: "text-vocabulary",
    },
    phonetic: {
        icon: Mic,
        label: "Phonetic",
        accent: "bg-phonetic",
        badgeBg: "bg-phonetic/15",
        badgeText: "text-phonetic",
    },
    orthographic: {
        icon: PencilLine,
        label: "Orthographic",
        accent: "bg-orthographic",
        badgeBg: "bg-orthographic/15",
        badgeText: "text-orthographic",
    },
};

function ControlBar({ type }: { type: Flashcard["type"] }) {
    const { icon: TypeIcon, label, badgeBg, badgeText } = TYPE_STYLES[type];

    return (
        <div className="flex items-center gap-3">
            <div className={`flex items-center gap-1.5 rounded-full px-3 py-1 text-sm font-medium ${badgeBg} ${badgeText}`}>
                <TypeIcon size={16} />
                {label}
            </div>

            <button
                type="button"
                className="flex items-center gap-1 rounded-md px-2 py-1 text-sm text-white/60 transition-colors hover:bg-white/5 hover:text-white"
            >
                US - English
                <ChevronDown size={14} />
            </button>

            <div className="ml-auto flex items-center gap-0.5 rounded-lg bg-white/5 p-1">
                <IconButton label="Move up" icon={ArrowUp} iconSize={15} />
                <IconButton label="Move down" icon={ArrowDown} iconSize={15} />
                <IconButton label="Duplicate" icon={Copy} iconSize={15} />
                <IconButton label="Delete" icon={Trash2} iconSize={15} />
            </div>
        </div>
    );
}

interface VocabularyInputProps {
    flashcard: VocabularyFlashcard;
    onChange: (next: VocabularyFlashcard) => void;
}

function VocabularyInput({ flashcard, onChange }: VocabularyInputProps) {
    return (
        <div className="flex flex-col gap-5">
            <RichInputField
                label="Phrase"
                field={flashcard.front.phrase}
                allowMedia
                onChange={(phrase) => onChange({ ...flashcard, front: { phrase } })}
            />
            <div className="grid grid-cols-1 gap-5 md:grid-cols-2">
                <PlainInputField
                    label="Word or Expression"
                    value={flashcard.back.wordOrExpression.content}
                    onChange={(content) =>
                        onChange({ ...flashcard, back: { ...flashcard.back, wordOrExpression: { ...flashcard.back.wordOrExpression, content } } })
                    }
                />
                <PlainInputField
                    label="Meaning"
                    value={flashcard.back.meaning.content}
                    onChange={(content) =>
                        onChange({ ...flashcard, back: { ...flashcard.back, meaning: { ...flashcard.back.meaning, content } } })
                    }
                />
            </div>
        </div>
    );
}

interface PhoneticInputProps {
    flashcard: PhoneticFlashcard;
    onChange: (next: PhoneticFlashcard) => void;
}

function PhoneticInput({ flashcard, onChange }: PhoneticInputProps) {
    return (
        <div className="flex flex-col gap-5">
            <PlainInputField
                label="Phrase"
                value={flashcard.front.phrase.content}
                onChange={(content) => onChange({ ...flashcard, front: { phrase: { ...flashcard.front.phrase, content } } })}
            />
            <RichInputField
                label="IPA Phonetics"
                field={flashcard.back.ipaPhonetics}
                onChange={(ipaPhonetics) => onChange({ ...flashcard, back: { ipaPhonetics } })}
            />
        </div>
    );
}

interface OrthographicInputProps {
    flashcard: OrthographicFlashcard;
    onChange: (next: OrthographicFlashcard) => void;
}

function OrthographicInput({ flashcard, onChange }: OrthographicInputProps) {
    return (
        <div className="flex flex-col gap-5">
            <PlainInputField
                label="Phrase"
                value={flashcard.front.phrase.content}
                onChange={(content) => onChange({ ...flashcard, front: { phrase: { ...flashcard.front.phrase, content } } })}
            />
            <AudioField label="Audio" media={flashcard.front.phrase.media} />
        </div>
    );
}

export interface FlashcardInputProps {
    flashcard: Flashcard;
}

// O estado interno usa `flashcard` só como valor inicial (useState). Para editar
// um flashcard diferente no mesmo componente já montado, quem usa este componente
// deve remontá-lo passando `key={flashcard.id}`.
export function FlashcardInput({ flashcard }: FlashcardInputProps) {
    const [current, setCurrent] = useState<Flashcard>(flashcard);

    let content: React.ReactNode;
    switch (current.type) {
        case "vocabulary":
            content = <VocabularyInput flashcard={current} onChange={setCurrent} />;
            break;
        case "phonetic":
            content = <PhoneticInput flashcard={current} onChange={setCurrent} />;
            break;
        case "orthographic":
            content = <OrthographicInput flashcard={current} onChange={setCurrent} />;
            break;
    }

    return (
        <div className="overflow-hidden rounded-2xl border border-white/10 bg-panel shadow-lg shadow-black/20">
            <div className={`h-1 w-full ${TYPE_STYLES[current.type].accent}`} />
            <div className="flex flex-col gap-5 p-5">
                <ControlBar type={current.type} />
                {content}
            </div>
        </div>
    );
}
