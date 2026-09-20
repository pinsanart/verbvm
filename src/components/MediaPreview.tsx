import { Media } from "../types/Media";

export interface MediaPreviewProps {
    media?: Media[];
}

export function MediaPreview({ media }: MediaPreviewProps) {
    if (!media || media.length === 0) return null;

    return (
        <div className="flex flex-wrap gap-2 rounded-xl border border-white/10 bg-white/5 p-2">
            {media.map((item) => {
                if (item.mimeType.startsWith("image/")) {
                    return <img key={item.id} src={item.localPath} alt="" className="h-16 w-16 rounded-lg object-cover" />;
                }
                if (item.mimeType.startsWith("audio/")) {
                    return <audio key={item.id} controls src={item.localPath} className="h-9" />;
                }
                return null;
            })}
        </div>
    );
}
