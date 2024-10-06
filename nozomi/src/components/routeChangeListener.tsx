"use client";

import { useRouteStore } from "@/stores/routeStore";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";

// legitimately horrible code i would not use this if i had a choice

export function RouteChangeListener() {
  const pathname = usePathname();
  const [changes, setChanges] = useState(0);

  const { push, getNextLast, pop, history, future } = useRouteStore();

  useEffect(() => {
    //console.log(`Route changed to: ${pathname}`);
    // if route is same as future
    if (future[0] === pathname && pathname != null) {
      push(pathname);
    }
    // if route is the same as previous, pop the last one
    let nl = getNextLast();
    //console.log("state", window.history.state);
    //console.log("comparing", nl, pathname);
    //console.log(history);
    if (nl === pathname && nl != null) {
      pop();
      return;
    }
    setChanges((prev) => prev + 1);
    // push changes to route store
    pathname != null && push(pathname);
    //console.log(history);
    //console.log(future);
  }, [pathname]);

  return <></>;
}
