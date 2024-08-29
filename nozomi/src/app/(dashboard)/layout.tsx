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
    <main className="grid overflow-clip flex-grow grid-areas-main-mobile md:grid-areas-main-desktop h-full gap-0 md:gap-2 p-0 md:p-2 grid-cols-[auto] md:grid-cols-[auto_1fr] grid-rows-[1fr_auto]">
      <div
        style={{
          gridArea: "left-sidebar",
        }}
        className="hidden md:flex relative flex-col area-left-sidebar overflow-y-auto overflow-x-clip md:bg-slate-950 rounded-lg"
      >
        <Menu />
      </div>
      <div
        style={{
          gridArea: "main-view",
        }}
        className="flex relative flex-col area-main-view overflow-y-auto overflow-x-clip md:bg-slate-950 rounded-lg -mr-2 xl:mr-0"
        id="main-view"
      >
        <div className="flex flex-col w-full h-full" id="main">
          <NavControls />
          {children}
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
        <div className="flex relative overflow-y-auto overflow-x-clip lg:hidden flex-col rounded-lg bg-slate-900 mx-1">
          <MobileControls />
        </div>
      </div>
      <div className="flex md:hidden relative flex-col h-16 pt-2 area-nav">
        <Menu />
      </div>
    </main>
  );
}
