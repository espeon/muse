"use server";
import { auth } from "@/auth";
import SignIn from "./signIn";
import * as Avatar from "@radix-ui/react-avatar";

export default async function AuthMenu() {
  const session = await auth();

  if (!session || !session.id) {
    return <SignIn />;
  } else {
    return (
      <Avatar.Root>
        <Avatar.Fallback
          className="p-2 rounded-full bg-slate-500/50"
          delayMs={800}
        >
          {session.name &&
            session.name
              .split(" ")
              .map((s) => s[0])
              .join("")}
        </Avatar.Fallback>
        <Avatar.Image
          src={session.image ?? "https://i.imgur.com/moGByde.jpeg"}
          alt={session.name ?? "Photo of user"}
          className="w-10 h-10 rounded-full"
        />
      </Avatar.Root>
    );
  }
}
