import { signIn } from "@/auth";
import clsx from "clsx";
import { cookies } from "next/headers";
import { BsGoogle, BsTwitter } from "react-icons/bs";
import { LuCat, LuFlower, LuFlower2 } from "react-icons/lu";

export default async function SignIn() {
  let lastUsedProvider = (await cookies()).get("lastUsedProvider");
  const providers = [{ name: "LuteaID", id: "zitadel", icon: LuFlower2 }];

  // Sort providers to have the last used one on top
  const sortedProviders = lastUsedProvider
    ? providers.sort((a, b) =>
        a.id === lastUsedProvider.value
          ? -1
          : b.id === lastUsedProvider.value
            ? 1
            : 0,
      )
    : providers;
  // pull the first one off if there is a last used provider
  let lastUsed = null;
  if (lastUsedProvider && sortedProviders.length > 0) {
    lastUsed = sortedProviders.shift();
  }
  return (
    <div className={clsx("flex flex-col items-center justify-center w-full")}>
      {lastUsed ? (
        <div className="flex flex-col items-center justify-center w-full">
          <div className="pb-3 text-lg text-gray-300 transition-all duration-700 center">
            Sign in using your last used provider, or continue with a different
            one.
          </div>
          <div className="pb-2 text-sm text-gray-400 transition-all duration-700 center">
            You signed in using {lastUsed.name} last time.
          </div>
          {loginButton(lastUsed)}
          <div className="flex items-center justify-center mt-3 mb-1 w-full gap-3">
            <div className="h-1 w-1 bg-wisteria-400 rounded-full" />
            <div className="h-1 w-1 bg-wisteria-400 rounded-full" />
            <div className="h-1 w-1 bg-wisteria-400 rounded-full" />
          </div>
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center w-full">
          <div className="pb-4 text-lg text-gray-300 transition-all duration-700 center">
            Sign in using your preferred method.
          </div>
        </div>
      )}
      {sortedProviders.map((provider) => (
        <div key={provider.id}>{loginButton(provider)}</div>
      ))}
    </div>
  );
}

function loginButton(provider: any) {
  return (
    <form
      className="w-full"
      action={async () => {
        "use server";
        await signIn(provider.id);
      }}
    >
      <button
        type="submit"
        className="flex bg-black justify-center hover:bg-wisteria-700 border border-wisteria-600 rounded-lg w-full px-4 py-2 text-white transition-all duration-200"
      >
        <provider.icon className="h-6 w-6 mr-2" /> Continue with {provider.name}
      </button>
    </form>
  );
}
