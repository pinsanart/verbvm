import { FormEvent, useState } from "react";
import { useLanguagesViewModel } from "../viewmodels/useLanguagesViewModel";

const inputClass =
    "w-full rounded-md border border-panel-border bg-transparent px-3 py-2 text-sm outline-none focus:border-vocabulary";
const primaryButton =
    "rounded-md bg-vocabulary px-3 py-2 text-sm font-medium disabled:opacity-50";
const dangerButton = "text-sm text-stack hover:underline";

const emptyToNull = (value: string) => (value.trim() === "" ? null : value.trim());

export default function LanguageTests() {
    const vm = useLanguagesViewModel();
    const { languages, variants, selected, error } = vm;

    const [iso6391, setIso6391] = useState("");
    const [iso6393, setIso6393] = useState("");
    const [name, setName] = useState("");

    const [bcp47Code, setBcp47Code] = useState("");
    const [variantName, setVariantName] = useState("");
    const [countryCode, setCountryCode] = useState("");

    const onCreateLanguage = async (e: FormEvent) => {
        e.preventDefault();
        const ok = await vm.addLanguage({ iso6391: emptyToNull(iso6391), iso6393: iso6393.trim(), name });
        if (ok) {
            setIso6391("");
            setIso6393("");
            setName("");
        }
    };

    const onCreateVariant = async (e: FormEvent) => {
        e.preventDefault();
        const ok = await vm.addVariant({
            bcp47Code: bcp47Code.trim(),
            variantName,
            countryCode: emptyToNull(countryCode),
        });
        if (ok) {
            setBcp47Code("");
            setVariantName("");
            setCountryCode("");
        }
    };

    return (
        <div className="space-y-6 p-6">
            <h1 className="text-2xl font-semibold">Language tests</h1>

            {error && (
                <div role="alert" className="rounded-md border border-stack px-3 py-2 text-sm text-stack">
                    {error}
                </div>
            )}

            <div className="grid gap-6 lg:grid-cols-2">
                <section className="space-y-4">
                    <h2 className="text-lg font-medium">Languages</h2>

                    <form onSubmit={onCreateLanguage} className="space-y-2 rounded-md border border-panel-border p-4">
                        <input className={inputClass} placeholder="Name (Japanese)" value={name}
                            onChange={(e) => setName(e.target.value)} required />
                        <div className="grid grid-cols-2 gap-2">
                            <input className={inputClass} placeholder="ISO 639-3 (jpn)" maxLength={3}
                                value={iso6393} onChange={(e) => setIso6393(e.target.value)} required />
                            <input className={inputClass} placeholder="ISO 639-1 (ja), optional" maxLength={2}
                                value={iso6391} onChange={(e) => setIso6391(e.target.value)} />
                        </div>
                        <button type="submit" className={primaryButton}>Create language</button>
                    </form>

                    <ul className="space-y-2">
                        {languages.length === 0 && <li className="text-sm opacity-60">No languages yet.</li>}
                        {languages.map((language) => (
                            <li key={language.id}
                                className={`flex items-center justify-between rounded-md border px-3 py-2 ${
                                    language.id === selected?.id ? "border-vocabulary" : "border-panel-border"
                                }`}>
                                <button type="button" className="flex-1 text-left"
                                    onClick={() => vm.selectLanguage(language.id)}>
                                    <span className="font-medium">{language.name}</span>
                                    <span className="ml-2 text-sm opacity-60">
                                        {language.iso6393}{language.iso6391 ? ` / ${language.iso6391}` : ""}
                                    </span>
                                </button>
                                <button type="button" className={dangerButton}
                                    onClick={() => vm.removeLanguage(language.id)}>Delete</button>
                            </li>
                        ))}
                    </ul>
                </section>

                <section className="space-y-4">
                    <h2 className="text-lg font-medium">
                        {selected ? `Variants of ${selected.name}` : "Variants"}
                    </h2>

                    {!selected && <p className="text-sm opacity-60">Select a language to manage its variants.</p>}

                    {selected && (
                        <>
                            <form onSubmit={onCreateVariant} className="space-y-2 rounded-md border border-panel-border p-4">
                                <div className="grid grid-cols-2 gap-2">
                                    <input className={inputClass} placeholder="BCP 47 (ja-JP)" maxLength={10}
                                        value={bcp47Code} onChange={(e) => setBcp47Code(e.target.value)} required />
                                    <input className={inputClass} placeholder="Country (JP), optional" maxLength={2}
                                        value={countryCode} onChange={(e) => setCountryCode(e.target.value)} />
                                </div>
                                <input className={inputClass} placeholder="Variant name (Japanese, Japan)"
                                    value={variantName} onChange={(e) => setVariantName(e.target.value)} required />
                                <button type="submit" className={primaryButton}>Add variant</button>
                            </form>

                            <ul className="space-y-2">
                                {variants.length === 0 && <li className="text-sm opacity-60">No variants yet.</li>}
                                {variants.map((variant) => {
                                    const isDefault = selected.defaultVariantId === variant.id;
                                    return (
                                        <li key={variant.id}
                                            className="flex items-center justify-between rounded-md border border-panel-border px-3 py-2">
                                            <div>
                                                <span className="font-medium">{variant.variantName}</span>
                                                <span className="ml-2 text-sm opacity-60">
                                                    {variant.bcp47Code}{variant.countryCode ? ` · ${variant.countryCode}` : ""}
                                                </span>
                                                {isDefault && (
                                                    <span className="ml-2 rounded bg-orthographic px-1.5 py-0.5 text-xs font-medium text-black">
                                                        default
                                                    </span>
                                                )}
                                            </div>
                                            <div className="flex gap-3">
                                                {!isDefault && (
                                                    <button type="button" className="text-sm text-vocabulary hover:underline"
                                                        onClick={() => vm.makeDefaultVariant(variant.id)}>Set default</button>
                                                )}
                                                <button type="button" className={dangerButton}
                                                    onClick={() => vm.removeVariant(variant.id)}>Delete</button>
                                            </div>
                                        </li>
                                    );
                                })}
                            </ul>
                        </>
                    )}
                </section>
            </div>
        </div>
    );
}
