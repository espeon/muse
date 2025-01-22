"use client";
import dynamic from "next/dynamic";

const Player = dynamic(() => import("../components/player"), { ssr: false });

export default function PlayerWrapper() {
  return <Player />;
}
