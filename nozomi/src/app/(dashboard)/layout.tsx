import Queue from "@/components/queue";
import Controls from "@/components/controls";
import Menu from "@/components/menu";
import NavControls from "@/components/navControls";

import dynamic from "next/dynamic";
import MobileControls from "@/components/mobileControls";
dynamic(() => import("@/components/player"), { ssr: false });

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <main className="flex md:overflow-clip md:grid grid-areas-main-desktop h-full gap-0 md:gap-2 p-0 md:p-2 grid-cols-[auto] md:grid-cols-[auto_1fr] grid-rows-[1fr_auto]">
      <div
        style={{
          gridArea: "left-sidebar",
        }}
        className="hidden md:flex relative flex-col area-left-sidebar overflow-y-clip overflow-x-hidden md:bg-slate-950 rounded-lg"
      >
        <Menu />
      </div>
      <div
        className="flex flex-col area-main-view overflow-auto md:bg-slate-950 rounded-lg lg:-mr-2 xl:mr-0"
        id="main-view"
      >
        <div
          className="flex flex-col w-full h-max overflow-y-auto overflow-x-clip"
          id="main"
        >
          <NavControls />
          {children}
          <div className="md:hidden h-28"></div>
          <div className="flex md:hidden fixed bottom-0 left-0 right-0 z-10 flex-col h-32 area-nav bg-gradient-to-b from-black/60 to-black">
            <div className="flex flex-1 rounded-lg bg-slate-900 mx-1 shadow-md overflow-x-clip">
              <MobileControls />
            </div>
            <div className="flex-1 rounded-lg bg-black pt-1">
              <Menu />
            </div>
          </div>
        </div>
      </div>
      <div
        style={{
          gridArea: "right-sidebar",
        }}
        className="hidden xl:flex area-right-sidebar relative flex-col overflow-y-auto overflow-x-clip bg-slate-950 rounded-lg"
      >
        <Queue />
      </div>
      <div
        style={{
          gridArea: "now-playing-bar",
        }}
        className="h-16 lg:h-20 flex-col overflow-y-auto md:bg-transparent"
      >
        <div className="hidden lg:flex flex-1 rounded-lg bg-slate-950">
          <Controls />
        </div>
      </div>
    </main>
  );
}
