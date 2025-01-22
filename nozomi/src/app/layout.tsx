import type { Metadata } from "next";
import { Figtree, IBM_Plex_Mono } from "next/font/google";
import "./globals.css";
import Title from "@/components/helpers/title";
import { RouteChangeListener } from "@/components/routeChangeListener";
import { ConfigFetcher } from "@/components/configFetcher";
import PlayerWrapper from "@/components/playerWrapper";

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

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <head>
        <Title />
        <meta name="application-name" content="muse player" />
        <meta name="apple-mobile-web-app-capable" content="yes" />
        <meta name="apple-mobile-web-app-status-bar-style" content="black" />
        <meta name="apple-mobile-web-app-title" content="muse player" />
        <meta name="description" content="muse player" />
        <meta name="format-detection" content="telephone=no" />
        <meta name="mobile-web-app-capable" content="yes" />
        <meta name="msapplication-config" content="/icons/browserconfig.xml" />
        <meta name="msapplication-TileColor" content="#a3b8e2" />
        <meta name="msapplication-tap-highlight" content="no" />
        <meta name="theme-color" content="#325383" />

        <link rel="apple-touch-icon" href="/icons/touch-icon-iphone.png" />
        <link
          rel="apple-touch-icon"
          sizes="128x128"
          href="/icons/boom-box.128.png"
        />
        <link
          rel="apple-touch-icon"
          sizes="192x192"
          href="/icons/boom-box.192.png"
        />

        <link
          rel="icon"
          type="image/png"
          sizes="32x32"
          href="/icons/boom-box.png"
        />
        <link
          rel="icon"
          type="image/png"
          sizes="16x16"
          href="/icons/boom-box.png"
        />
        <link rel="manifest" href="/manifest.json" />
        <link rel="mask-icon" href="/icons/boom-box.svg" color="#5bbad5" />
        <link rel="shortcut icon" href="/boom-box.png" />

        <meta name="twitter:card" content="muse app" />
        <meta name="twitter:url" content="https://muse.lut.li" />
        <meta name="twitter:title" content="Muse Player" />
        <meta name="twitter:description" content="muse player" />
        <meta
          name="twitter:image"
          content="https://muse.lut.li/icons/boom-box.192.png"
        />
        <meta name="twitter:creator" content="@ameiwi" />
        <meta property="og:type" content="website" />
        <meta property="og:title" content="PWA App" />
        <meta property="og:description" content="Best PWA App in the world" />
        <meta property="og:site_name" content="PWA App" />
        <meta property="og:url" content="https://yourdomain.com" />
        <meta
          property="og:image"
          content="https://muse.lut.li/icons/boom-box.192.png"
        />
        <meta
          name="viewport"
          content="minimum-scale=1, initial-scale=1, width=device-width, shrink-to-fit=no, user-scalable=no, viewport-fit=cover"
        />
      </head>
      <body
        className={`${fig.className} ${mono.variable} lg:overflow-hidden h-full lg:h-screen min-h-lvh w-screen overflow-x-clip`}
      >
        {children}
        <PlayerWrapper />
        <RouteChangeListener />
        <ConfigFetcher />
      </body>
    </html>
  );
}
