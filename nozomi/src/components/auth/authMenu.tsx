"use server";
import { auth } from "@/auth";
import SignIn from "./signIn";

export default async function AuthMenu() {
  const session = await auth();

  if (!session || !session.user) {
    return <SignIn />;
  } else {
    console.log(session.user);
    return (
      <img
        className="h-10 w-10 rounded-full"
        src={session.user.image ?? "https://i.imgur.com/moGByde.jpeg"}
        alt={session.user.name ?? "avatar"}
      />
    );
  }
}
