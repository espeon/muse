"use server";
import { auth } from "@/auth";
import SignIn from "./signIn";
import { LuMusic2 } from "react-icons/lu";
import { redirect } from "next/navigation";

export default async function RedirectToHomeIfLoggedIn() {
  const session = await auth();

  if (!session || !session.id) {
    return <></>;
  } else {
    redirect("/home");

    return <>Redirecting...</>;
  }
}
