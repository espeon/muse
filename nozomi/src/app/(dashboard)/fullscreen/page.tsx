import Lyrics from "./lyrics";

export default function Fullscreen() {
  return (
    <div
      className="hide-scrollbar w-full h-screen md:max-w-[80vw] overflow-y-auto"
      style={{
        maskImage:
          "linear-gradient(to bottom, transparent 0%, black 20% , black 50%, transparent 100%)",
        WebkitMaskImage:
          "linear-gradient(to bottom, transparent 0%, black 20% , black 50%, transparent 100%)", // For WebKit browsers
      }}
    >
      <Lyrics />
    </div>
  );
}
