"use server";
import { auth } from "@/auth";
import SignIn from "./signIn";

export default async function AuthMenu() {
  const session = await auth();

  if (!session || !session.user) {
    return <SignIn />;
  } else {
    console.log(session.user);
    return <>Hello, {session.user.name}!</>;
  }
}
