"use server";
import { auth } from "@/auth";
import SignIn from "./signIn";
import { LuMusic2 } from "react-icons/lu";

export default async function AuthMenu() {
  const session = await auth();

  if (!session || !session.user) {
    return (
      <div className="text-2xl font-light transition-all duration-700 mb-4">
        To continue, you'll need to sign in.
      </div>
    );
  } else {
    console.log(session.user);
    return <>Hello, {session.user.name}!</>;
  }
}
