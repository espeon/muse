"use client";

import React from "react";

import { Session } from "@ory/client";
import { frontend } from "@/lib/ory";
import axios from "axios";
import { usePathname, useRouter } from "next/navigation";

interface SessionContextProps {
  data: Session | undefined;
  set: React.Dispatch<React.SetStateAction<Session | undefined>>;
  destroy: () => void;
  emailVerified: boolean;
  setEmailVerified: React.Dispatch<React.SetStateAction<boolean>>;
}

export const SessionContext = React.createContext<SessionContextProps>({
  data: undefined,
  set: () => {},
  destroy: () => {},
  emailVerified: false,
  setEmailVerified: () => {},
});

export const SessionProvider = ({
  children,
}: {
  children: React.ReactNode;
}) => {
  const router = useRouter();
  const pathname = usePathname();

  const [data, setData] = React.useState<Session | undefined>(undefined);
  const [emailVerified, setEmailVerified] = React.useState<boolean>(true);

  // get the session data
  React.useEffect(() => {
    frontend.toSession().then(({ data: session }) => {
      setData(session);
    });
  }, []);

  // check if the user has verified their email
  React.useEffect(() => {
    if (data) {
      if (data.identity?.verifiable_addresses?.length) {
        setEmailVerified(
          data.identity?.verifiable_addresses?.some(
            (address) => address.verified
          )
        );
      }
    }
  }, [data]);

  const destroy = React.useCallback(() => {
    frontend
      .createBrowserLogoutFlow()
      .then(({ data: flow }) => {
        return axios.get(flow.logout_url, {
          withCredentials: true,
        });
      })
      .then(() => {
        setData(undefined);
      })
      .catch((err) => {
        console.error(err);
      });
  }, []);

  return (
    <SessionContext.Provider
      value={{ data, set: setData, destroy, emailVerified, setEmailVerified }}
    >
      {children}
    </SessionContext.Provider>
  );
};

export const useSession = () => React.useContext(SessionContext);
