import { ArrowLeft } from "lucide-react";
import { Link, useParams } from "react-router";
import { CountryFlag } from "../components/CountryFlag";
import { styleForType } from "../data/FlashcardTypeStyles";
import { useDecksViewModel } from "../viewmodels/useDecksViewModel";

// Stack is not a flashcard type but a way to link flashcards, so it has no editor yet.
const TILES = [
  { slug: "stack", enabled: false },
  { slug: "vocabulary", enabled: true },
  { slug: "phonetic", enabled: false },
  { slug: "orthographic", enabled: false },
];

/** The second step of creating flashcards: pick what to create in the chosen deck. */
export default function CreateTypes() {
  const { deckName = "" } = useParams();
  const vm = useDecksViewModel();
  const deck = vm.decks.find((d) => d.name === deckName);

  return (
    <div className="mx-auto flex w-full flex-col gap-8 p-6 md:max-w-[60%]">
      <div className="flex flex-col items-center gap-2">
        <Link to="/create" className="flex items-center gap-1 self-start text-sm text-white/50 hover:text-white">
          <ArrowLeft size={14} />
          Decks
        </Link>
        <h1 className="text-center text-3xl font-light uppercase tracking-wide">Create flashcards</h1>
        <div className="flex items-center gap-2 text-sm text-white/60">
          {deck && <CountryFlag code={deck.variant.countryCode} className="h-3.5 w-5" />}
          <span className="font-medium text-white/80">{deckName}</span>
          {deck && <span>· {deck.variant.variantName}</span>}
        </div>
      </div>

      {vm.error && (
        <div role="alert" className="rounded-lg border border-stack/70 px-3 py-2 text-sm text-stack">
          {vm.error}
        </div>
      )}

      {vm.status === "ready" && !deck && (
        <p className="text-center text-sm text-white/50">This deck does not exist.</p>
      )}

      {deck && (
        <div className="flex flex-wrap justify-center gap-4">
          {TILES.map(({ slug, enabled }) => {
            const { icon: Icon, label, tile } = styleForType(slug);
            const classes = `relative flex h-36 w-36 flex-col items-center justify-center gap-3 rounded-lg border-2 border-dotted transition-colors ${tile}`;
            const content = (
              <>
                <span className="text-base font-medium text-white">{label}</span>
                <Icon size={48} strokeWidth={1} />
                {!enabled && (
                  <span className="absolute right-2 top-2 rounded-full bg-white/10 px-2 py-0.5 text-[10px] font-medium uppercase tracking-wide text-white/60">
                    Soon
                  </span>
                )}
              </>
            );
            return enabled ? (
              <Link key={slug} to={`/create/${encodeURIComponent(deckName)}/${slug}`} className={classes}>
                {content}
              </Link>
            ) : (
              <div key={slug} aria-disabled="true" className={`${classes} cursor-not-allowed opacity-40`}>
                {content}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
