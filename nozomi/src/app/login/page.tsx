import RedirectToHomeIfLoggedIn from "@/components/auth/redirectToHomeIfLoggedIn";
import SignIn from "@/components/auth/signIn";
import SetNavTitle from "@/components/helpers/setNavTitle";
import { LuMusic4 } from "react-icons/lu";

export default async function AlbumPage() {
  return (
    <>
      <div className="flex flex-col justify-center items-center h-screen min-w-32 mx-4 md:mx-12">
        <div className="flex flex-col justify-center items-center text-center h-full w-full max-w-xs">
          <LuMusic4 className="h-16 w-16 text-wisteria-300 transition-all duration-700 mb-2" />
          <RedirectToHomeIfLoggedIn />
          <div className="text-3xl font-semibold transition-all duration-700 mb-2">
            Welcome!
          </div>
          <SignIn />
          <SetNavTitle title="Home" />
        </div>
      </div>
    </>
  );
}
