import { easeInOutQuad, easeOutExpo, mapRange } from "@/helpers/animath";

export default function Ellipsis({
  start,
  end,
  currentTime}: {
  start: number;
  end: number;
  currentTime: number;
}
) {
  let ellipsis = start + 5 < end;
  let ellipsisEnd = end;
  let isEllActive = currentTime >= start - 2 && currentTime <= ellipsisEnd;
  let ellPercent = ((currentTime - start) / (ellipsisEnd - start)) * 100;
  if (ellipsis)
    return (
      <div
        className={`h-0 opacity-0 transition-all duration-1000 flex justify-start origin-top-left pl-2 ${
          isEllActive ? `h-auto text-3xl mb-4 mt-2 ${ellPercent > 5 && ellPercent < 99 ? "py-4 opacity-100 gap-10" : "py-0 opacity-25 scale-75 gap-6"}` : "gap-0"
        }`}
      >
        <div
          className={`size-4 transition-all duration-500 rounded-full ${
            ellPercent > 10 ? "bg-blue-300 " : "bg-blue-500"
          }`}
          style={{
            scale:
              mapRange(ellPercent, 0, 20, 1, 1.5) *
              mapRange(ellPercent, 95, 99, 1, 0.1, easeOutExpo),
          }}
        ></div>
        <div
          className={`size-4 transition-all duration-500 rounded-full ${
            ellPercent > 45 ? "bg-blue-300 " : "bg-blue-500 "
          }`}
          style={{
            scale:
              mapRange(ellPercent, 20, 55, 1, 1.5) *
              mapRange(ellPercent, 95, 98, 1, 0.1, easeOutExpo),
          }}
        ></div>
        <div
          className={`size-4 transition-all duration-500 rounded-full ${
            ellPercent > 90 ? "bg-blue-300 " : "bg-blue-500 "
          }`}
          style={{
            scale:
              mapRange(ellPercent, 45, 85, 1, 1.5) *
              mapRange(ellPercent, 95, 99, 1, 0.1, easeOutExpo),
          }}
        ></div>
      </div>
    );

  return <></>;
}
