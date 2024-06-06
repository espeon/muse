"use client";

import * as React from "react";

import { cn } from "@/lib/utils";
import { Icons } from "@/components/icons";
import { Node } from "@/components/auth/node";
import { Message } from "@/components/auth/message";

import { frontend } from "@/lib/ory";
import { UpdateVerificationFlowBody, VerificationFlow } from "@ory/client";
import { filterNodesByGroups } from "@ory/integrations/ui";
import { isAxiosError } from "axios";
import { useSession } from "@/contexts/Session";
import { useRouter, useSearchParams } from "next/navigation";

interface UserVerificationFormProps
  extends React.HTMLAttributes<HTMLDivElement> {}

export function UserVerificationForm({
  className,
  ...props
}: UserVerificationFormProps) {
  const router = useRouter();
  const session = useSession();
  const searchParams = useSearchParams();

  const [flow, setFlow] = React.useState<VerificationFlow | undefined>(
    undefined
  );
  const [isLoading, setIsLoading] = React.useState<boolean>(false);
  const formRef = React.useRef<HTMLFormElement>(null);

  async function onSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsLoading(true);

    const form = event.currentTarget;
    const formData = new FormData(form);

    // map the entire form data to JSON for the request body
    let body = Object.fromEntries(
      formData
    ) as unknown as UpdateVerificationFlowBody;

    // extract the method from the submit button
    if ("submitter" in event.nativeEvent) {
      const method = (
        event.nativeEvent as unknown as { submitter: HTMLInputElement }
      ).submitter;
      body = {
        ...body,
        ...{ [method.name]: method.value },
      };
    }

    try {
      const { data } = await frontend.updateVerificationFlow({
        flow: flow!.id,
        updateVerificationFlowBody: body,
      });

      // step after user enters their email
      if (data.state === "sent_email") {
        formRef.current?.reset();
        setFlow(data);
      }

      // step after user enters their code
      if (data.state === "passed_challenge") {
        if (session.data) {
          session.setEmailVerified(true);
          router.push("/");
        } else {
          setFlow(data);
        }
      }
    } catch (err: unknown) {
      if (!isAxiosError(err)) {
        throw err;
      }

      // handle the error
      if (err.response?.status === 400) {
        // user input error
        // show the error messages in the UI
        setFlow(err.response.data as VerificationFlow);
      }
    } finally {
      setIsLoading(false);
    }
  }

  React.useEffect(() => {
    const flowId = searchParams.get("flow");

    if (flowId) {
      frontend
        .getVerificationFlow({ id: flowId })
        .then(({ data: flow }) => {
          setFlow(flow);
        })
        .catch((err: unknown) => {
          if (!isAxiosError(err)) {
            throw err;
          }

          if (err.response?.status === 404 || err.response?.status === 403) {
            router.push("/auth/verification");
          }
        });
    } else {
      frontend.createBrowserVerificationFlow().then(({ data: flow }) => {
        setFlow(flow);
      });
    }
  }, [router, searchParams]);

  if (!flow) {
    return (
      <div className="flex justify-center">
        <Icons.spinner className="mr-2 h-4 w-4 animate-spin" />
      </div>
    );
  }

  return (
    <div className={cn("grid gap-6", className)} {...props}>
      {flow.ui.messages?.map((message, index) => (
        <Message key={index} text={message} />
      ))}

      <form onSubmit={onSubmit} ref={formRef}>
        <div className="grid gap-2">
          {filterNodesByGroups({
            nodes: flow.ui.nodes,
            groups: ["default", "code"],
          }).map((node, index) => (
            <Node key={index} node={node} loading={isLoading} />
          ))}
        </div>
      </form>
    </div>
  );
}
