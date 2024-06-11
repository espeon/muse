import NavControls from "@/components/navControls";
import Lyrics from "./lyrics";

export default function Fullscreen() {
  return (
    <div className="flex flex-col w-full" id="main">
      <NavControls />
      <Lyrics />
    </div>
  );
}
