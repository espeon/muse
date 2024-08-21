import { signIn } from "@/auth";

export default function SignIn() {
  return (
    <form
      action={async () => {
        "use server";
        await signIn("zitadel");
      }}
    >
      <button className="btn btn-primary" type="submit">
        Sign in with Zitadel
      </button>
    </form>
  );
}
