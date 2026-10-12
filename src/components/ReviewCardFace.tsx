import { ReviewMedia } from "./ReviewMedia";
import type { ReviewField } from "../types/Review";

export interface ReviewCardFaceProps {
    fields: ReviewField[];
}

/**
 * The fields of one side of a flashcard, one after the other. Rich fields are sanitized HTML (the
 * backend cleans it before it is stored), plain fields are text. Media shows under its field.
 */
export function ReviewCardFace({ fields }: ReviewCardFaceProps) {
    return (
        <div className="flex flex-col divide-y divide-line">
            {fields.map((field) => (
                <section key={field.key} className="flex flex-col items-center gap-3 py-5 text-center first:pt-0 last:pb-0">
                    {fields.length > 1 && <h3 className="text-meta font-medium text-muted">{field.label}</h3>}
                    {field.content.trim() !== "" &&
                        (field.isRich ? (
                            <div
                                className="rich max-w-prose text-heading md:text-title"
                                dangerouslySetInnerHTML={{ __html: field.content }}
                            />
                        ) : (
                            <p className="max-w-prose whitespace-pre-wrap text-heading md:text-title">{field.content}</p>
                        ))}
                    <ReviewMedia media={field.media} />
                </section>
            ))}
        </div>
    );
}
