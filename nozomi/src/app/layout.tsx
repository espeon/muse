import type { Metadata } from "next";
import { Figtree, IBM_Plex_Mono } from "next/font/google";
import "./globals.css";
import Title from "@/components/helpers/title";
import { RouteChangeListener } from "@/components/routeChangeListener";
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
        className={`${fig.className} ${mono.variable} lg:overflow-hidden min-h-screen h-max lg:h-screen w-screen overflow-x-clip`}
      >
        {children}
        <Player />
        <RouteChangeListener />
        <ConfigFetcher />
      </body>
    </html>
  );
}
