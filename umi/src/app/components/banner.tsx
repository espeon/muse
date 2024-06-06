"use client";

import Link from "next/link";

import { useSession } from "@/contexts/Session";

export function Banner() {
  const session = useSession();

  if (!session.data) {
    return (
      <p className="fixed left-0 top-0 flex w-full justify-center border-b border-gray-300 bg-gradient-to-b from-zinc-200 pb-6 pt-8 backdrop-blur-2xl dark:border-neutral-800 dark:bg-zinc-800/30 dark:from-inherit lg:static lg:w-auto  lg:rounded-xl lg:border lg:bg-gray-200 lg:p-4 lg:dark:bg-zinc-800/30">
        Get started by&nbsp;
        <Link href="/auth/signup">
          <code className="font-mono font-bold">Sign Up</code>
        </Link>
        &nbsp;or&nbsp;
        <Link href="/auth/login">
          <code className="font-mono font-bold">Sign In</code>
        </Link>
      </p>
    );
  }

  return (
    <p className="fixed left-0 top-0 flex w-full justify-center border-b border-gray-300 bg-gradient-to-b from-zinc-200 pb-6 pt-8 backdrop-blur-2xl dark:border-neutral-800 dark:bg-zinc-800/30 dark:from-inherit lg:static lg:w-auto  lg:rounded-xl lg:border lg:bg-gray-200 lg:p-4 lg:dark:bg-zinc-800/30">
      You are logged in.&nbsp;
      <code
        className="font-mono font-bold cursor-pointer"
        onClick={() => session.destroy()}
      >
        Click to Logout
      </code>
    </p>
  );
}
