"use client"; // Error components must be Client Components

import { useEffect } from "react";
import { usePathname } from "next/navigation";

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    // Log the error to an error reporting service
    console.error(error);
  }, [error]);
  // pathname
  const pathname = usePathname();
  return (
    <div className="flex-1 flex flex-col justify-center items-center">
      <div className="text-center">
        <div className="text-5xl text-gray-400 font-mono">404</div>
        <div className="text-sm md:text-xl text-gray-400">This {pathname.split("/")[1]} left the band and is touring solo.</div>
        <div className="text-xs md:text-md text-gray-400 mt-4 font-mono flex-wrap">
          <div className="flex flex-row">Error Stack:</div>
          <div>
            {error.stack &&
              error.stack.split("\n").map((line, index) => (
                <div className="group flex flex-row text-left first:rounded-t-md last:rounded-b-md bg-black/75 hover:bg-black/25 transition-colors duration-250">
                  <div className="mr-2 p-2 bg-slate-800 group-hover:bg-slate-800/50 group-first:rounded-tl-md group-last:rounded-bl-md transition-colors duration-250">
                    {index + 1}{" "}
                  </div>
                  <div className="px-3 pl-1 py-2">{line}</div>
                </div>
              ))}
          </div>
        </div>
      </div>
    </div>
  );
}
