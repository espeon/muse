import { useTitleStore } from "@/stores/titleStore";
import { useEffect } from "react";

export default function HideNavTitle() {
    const { setPageTitleVisible, setPageTitle } = useTitleStore();

    useEffect(() => {
    setPageTitleVisible(true);
    }, []);

    return <></>;
}