"use server";
import AuthMenu from "./auth/authMenu";
import NavControlsClient from "./navControlsClient";
import { auth } from "@/auth";

export default async function NavControls() {
  return (
    <NavControlsClient>
      <AuthMenu />
    </NavControlsClient>
  );
}
