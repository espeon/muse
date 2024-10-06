import RedirectToHomeIfLoggedIn from "@/components/auth/redirectToHomeIfLoggedIn";
import SignIn from "@/components/auth/signIn";
import SetNavTitle from "@/components/helpers/setNavTitle";
import { LuMusic4 } from "react-icons/lu";

export default async function AlbumPage() {
  return (
    <>
      <div className="flex flex-col justify-center items-center h-full min-w-32 mt-16 lg:mt-0 mx-4 lg:mx-12">
        <LuMusic4 className="h-16 w-16 text-pink-400 transition-all duration-700 mb-2" />
        <RedirectToHomeIfLoggedIn />
        <SignIn />
        <SetNavTitle title="Home" />
      </div>
    </>
  );
}
