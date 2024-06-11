import { useTitleStore } from "@/stores/titleStore";
import { useEffect } from "react";

export default function SetNavTitleConstant({ title = "" }: { title: string }) {
    const { setPageTitleVisible, setPageTitle } = useTitleStore();

    useEffect(() => {
    setPageTitleVisible(false);
    setPageTitle(title);
    }, []);

    return <></>;
}