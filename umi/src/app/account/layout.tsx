import { Separator } from "@/components/ui/separator";
import { Metadata } from "next";
import { SidebarNav } from "@/app/account/components/sidebar-nav";
import { GoBack } from "./components/go-back";

const sidebarNavItems = [
  {
    title: "Settings",
    href: "/account/settings",
  },
];

export const metadata: Metadata = {
  title: "Account",
  description: "Account forms built using the components.",
};

export default function AccountPage({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <>
      <div className="container flex flex-col items-start justify-between space-y-2 py-4 sm:flex-row sm:items-center sm:space-y-0 md:h-16">
        <div className="relative z-20 flex items-center text-lg font-medium">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="mr-2 h-6 w-6"
          >
            <path d="M15 6v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3V6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3" />
          </svg>
          Acme Inc
        </div>
        <div className="ml-auto flex space-x-2 sm:justify-end">
          <GoBack />
        </div>
      </div>
      <Separator />
      <div className="container">
        <div className="hidden space-y-6 p-10 pb-16 md:block">
          <div className="space-y-0.5">
            <h2 className="text-2xl font-bold tracking-tight">Account</h2>
            <p className="text-muted-foreground">
              Manage your account settings and set preferences.
            </p>
          </div>
          <Separator className="my-6" />
          <div className="flex flex-col space-y-8 lg:flex-row lg:space-x-12 lg:space-y-0">
            <aside className="-mx-4 lg:w-1/5">
              <SidebarNav items={sidebarNavItems} />
            </aside>
            <div className="flex-1 lg:max-w-2xl">{children}</div>
          </div>
        </div>
      </div>
    </>
  );
}
