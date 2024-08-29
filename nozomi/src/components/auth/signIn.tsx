import { signIn } from "@/auth";

export default function SignIn() {
  return (
    <form
      action={async () => {
        "use server";
        await signIn("zitadel");
      }}
    >
      <button
        className="bg-pink-900 hover:bg-pink-700 text-white py-2 px-4 rounded"
        type="submit"
      >
        Sign in
      </button>
    </form>
  );
}
