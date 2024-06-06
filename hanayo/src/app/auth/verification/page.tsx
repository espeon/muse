import Link from "next/link";
import { UserVerificationForm } from "@/app/auth/verification/components/user-verification-form";

export default function Login() {
  return (
    <div className="mx-auto flex w-full flex-col justify-center space-y-6 sm:w-[350px]">
      <div className="flex flex-col space-y-2 text-center">
        <h1 className="text-2xl font-semibold tracking-tight">
          Verify your account
        </h1>
      </div>
      <UserVerificationForm />
      <p className="px-8 text-center text-sm text-muted-foreground">
        By clicking continue, you will receive an email according to our{" "}
        <Link
          href="/privacy"
          className="underline underline-offset-4 hover:text-primary"
        >
          Privacy Policy
        </Link>
        .
      </p>
    </div>
  );
}
