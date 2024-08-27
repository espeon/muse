import type { Metadata } from "next";
import { Figtree, IBM_Plex_Mono } from "next/font/google";
import "./globals.css";
import Title from "@/components/helpers/title";
import Queue from "@/components/queue";
import Controls from "@/components/controls";
import Menu from "@/components/menu";
import NavControls from "@/components/navControls";
import { RouteChangeListener } from "@/components/routeChangeListener";
import MobileControls, {
  MobileSheetControls,
} from "@/components/mobileControls";
import { ConfigFetcher } from "@/components/configFetcher";
import dynamic from "next/dynamic";

const fig = Figtree({ subsets: ["latin"], variable: "--font-fig" });
const mono = IBM_Plex_Mono({
  subsets: ["latin"],
  weight: "400",
  variable: "--font-mono",
});

export const metadata: Metadata = {
  title: "nozomi audio player",
  description: "muse player",
};

const Player = dynamic(() => import("../components/player"), { ssr: false });

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <head>
        <Title />
      </head>
      <body
        className={`${fig.className} ${mono.variable}`}
        style={{ height: "100vh", width: "100vw", overflow: "hidden" }}
      >
        <main
          className="md:grid hidden overflow-clip"
          style={{
            gridTemplateColumns: "auto 1fr",
            gridTemplateRows: "1fr auto",
            gridTemplateAreas: `
              "left-sidebar    main-view         right-sidebar"
              "now-playing-bar now-playing-bar now-playing-bar"`,
            gap: "0.5rem",
            position: "relative",
            height: "100%",
            padding: "0.5rem",
          }}
        >
          <div
            style={{
              gridArea: "left-sidebar",
            }}
            className="flex relative flex-col overflow-y-auto overflow-x-clip bg-slate-950 rounded-lg"
          >
            <Menu />
          </div>
          <div
            style={{
              gridArea: "main-view",
            }}
            className="flex relative flex-col overflow-y-auto overflow-x-clip bg-slate-950 rounded-lg -mr-2 xl:mr-0"
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
            className="hidden xl:flex relative flex-col overflow-y-auto overflow-x-clip bg-slate-950 rounded-lg"
          >
            <Queue />
          </div>
          <div
            className="hidden lg:flex relative flex-col h-20 overflow-y-auto rounded-lg"
            style={{ gridArea: "now-playing-bar" }}
          >
            <Controls />
          </div>
          <div
            style={{
              gridArea: "now-playing-bar",
            }}
            className="flex lg:hidden relative flex-col h-16 overflow-y-auto overflow-x-clip bg-slate-900 md:bg-slate-950 rounded-lg"
          >
            <MobileControls />
          </div>
        </main>
        <main
          className="grid md:hidden overflow-clip"
          style={{
            gridTemplateColumns: "auto",
            gridTemplateRows: "1fr auto",
            gridTemplateAreas: `
              "main-view"
              "now-playing-bar"
              "nav"`,
            gap: "",
            position: "relative",
            height: "100%",
          }}
        >
          <div
            style={{
              gridArea: "main-view",
            }}
            className="flex relative flex-col overflow-y-auto overflow-x-clip rounded-lg"
            id="main-view"
          >
            <div className="flex flex-col w-full" id="main">
              <NavControls />
              {children}
            </div>
          </div>
          <div
            style={{
              gridArea: "now-playing-bar",
            }}
            className="flex relative flex-col h-16 mx-2 overflow-y-auto overflow-x-clip bg-slate-900 rounded-lg"
          >
            <MobileControls />
          </div>
          <div
            style={{
              gridArea: "nav",
            }}
            className="flex relative flex-col overflow-y-clip pb-2 overflow-x-clip rounded-lg mt-2"
          >
            <Menu />
          </div>
        </main>
        <MobileSheetControls />
        <Player />
        <RouteChangeListener />
        <ConfigFetcher />
      </body>
    </html>
  );
}
