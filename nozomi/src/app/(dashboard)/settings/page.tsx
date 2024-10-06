import { redirect } from "next/navigation";
import LastFmConnect from "./lastfm";
import Logout from "./logout";

export default function SettingsPage() {
  return (
    <div className="flex flex-col justify-center items-center h-full">
      <div className="flex flex-col min-w-32 mx-4 md:mx-8 mt-16 max-w-5xl w-full">
        <div className="text-4xl lg:text-4xl xl:text-6xl font-semibold transition-all duration-700 mb-4">
          Settings
        </div>
        <div className="my-4">
          <div className="text-xl lg:text-xl xl:text-2xl transition-all duration-700">
            Link Last.fm
          </div>
          <LastFmConnect />
        </div>

        <div className="my-4">
          <div className="text-xl lg:text-xl xl:text-2xl transition-all duration-700">
            Log out
          </div>
          <Logout />
        </div>
      </div>
    </div>
  );
}
