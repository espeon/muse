import { cn } from "@/lib/utils";

/** Shimmering skeleton placeholder for loading states. */
export function Skeleton({ className }: { className?: string }) {
  return (
    <div
      className={cn(
        "animate-pulse rounded-md bg-foreground/10",
        className,
      )}
    />
  );
}

/** Album card skeleton — matches AlbumCard dimensions. */
export function AlbumCardSkeleton({ className }: { className?: string }) {
  return (
    <div className={cn("flex flex-col gap-2", className)}>
      <Skeleton className="aspect-square w-full rounded-xl" />
      <Skeleton className="h-3 w-3/4 rounded" />
      <Skeleton className="h-2.5 w-1/2 rounded" />
    </div>
  );
}

/** A row of album card skeletons for loading states. */
export function AlbumCardGrid({ count = 6 }: { count?: number }) {
  return (
    <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
      {Array.from({ length: count }).map((_, i) => (
        <AlbumCardSkeleton key={i} />
      ))}
    </div>
  );
}
