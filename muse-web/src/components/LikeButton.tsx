import { Heart } from "lucide-react";
import { useState, useCallback } from "react";
import { toggleLike } from "@/lib/api";
import { cn } from "@/lib/utils";

export function LikeButton({
  trackId,
  initialLiked = false,
  size = 16,
  className,
}: {
  trackId: number;
  initialLiked?: boolean;
  size?: number;
  className?: string;
}) {
  const [liked, setLiked] = useState(initialLiked);
  const [pending, setPending] = useState(false);

  const handleClick = useCallback(
    async (e: React.MouseEvent) => {
      e.stopPropagation();
      if (pending) return;
      const prev = liked;
      setLiked(!prev); // optimistic
      setPending(true);
      try {
        const result = await toggleLike(trackId);
        setLiked(result);
      } catch {
        setLiked(prev); // revert
      } finally {
        setPending(false);
      }
    },
    [trackId, liked, pending],
  );

  return (
    <button
      type="button"
      onClick={handleClick}
      disabled={pending}
      className={cn(
        "inline-flex items-center justify-center transition-colors",
        "text-muted-foreground hover:text-foreground",
        liked && "text-pink-500 hover:text-pink-400",
        className,
      )}
      aria-label={liked ? "Unlike" : "Like"}
      aria-pressed={liked}
    >
      <Heart size={size} fill={liked ? "currentColor" : "none"} />
    </button>
  );
}
