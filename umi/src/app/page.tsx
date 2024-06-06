import Image from "next/image";

import { Banner } from "@/app/components/banner";
import Link from "next/link";

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <div className="flex flex-col z-10 w-full max-w-5xl items-start justify-between text-lg lg:flex">
        <h1 className="text-6xl font-bold">
          Umi
        </h1>
        <p className="mt-3 text-2xl">
          Authentication, safe and easy.
        </p>
      </div>
    </main>
  );
}
