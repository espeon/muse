"use client";

import Link from "next/link";

export default function Logout() {
  return (
    <div className="flex flex-row w-full justify-between items-baseline">
      <p className="text-gray-500 mb-2">Sorry to see you go or something.</p>
      <Link href="/settings">
        <button className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 transition-colors">
          Log out
        </button>
      </Link>
    </div>
  );
}
