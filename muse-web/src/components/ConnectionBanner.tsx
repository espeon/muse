import { Loader2, WifiOff } from "lucide-react";
import { useRemote } from "@/remote/remote-client";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

/** Global connection indicator — shows a small banner when the remote
 *  WebSocket is reconnecting or has failed. Hidden when connected or
 *  disconnected (user not using remote features). */
export function ConnectionBanner() {
  const remote = useRemote();

  if (
    remote.connectionState === "connected" ||
    remote.connectionState === "disconnected" ||
    remote.connectionState === "connecting"
  ) {
    return null;
  }

  const isReconnecting = remote.connectionState === "reconnecting";

  return (
    <div
      className={cn(
        "fixed left-1/2 top-2 z-[55] flex -translate-x-1/2 items-center gap-2 rounded-full border px-4 py-1.5 text-xs font-medium shadow-lg backdrop-blur-md transition-all",
        isReconnecting
          ? "border-yellow-500/20 bg-yellow-500/10 text-yellow-500"
          : "border-red-500/20 bg-red-500/10 text-red-500",
      )}
    >
      {isReconnecting ? (
        <Loader2 size={12} className="animate-spin" />
      ) : (
        <WifiOff size={12} />
      )}
      <span>
        {isReconnecting
          ? `Reconnecting${remote.reconnectAttempt > 0 ? ` (${remote.reconnectAttempt})` : ""}…`
          : "Connection lost"}
      </span>
      {!isReconnecting && (
        <Button
          size="sm"
          variant="ghost"
          className="h-5 px-2 text-[10px]"
          onClick={() => void remote.retry()}
        >
          Retry
        </Button>
      )}
    </div>
  );
}
