import { Button } from "@/components/ui/button";
import Link from "next/link";

export function GoBack() {
  return (
    <Link href="/">
      <Button variant="secondary">Go Back</Button>
    </Link>
  );
}
