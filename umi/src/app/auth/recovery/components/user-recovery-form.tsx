"use client";

import * as React from "react";

import { cn } from "@/lib/utils";
import { Icons } from "@/components/icons";
import { Node } from "@/components/auth/node";
import { Message } from "@/components/auth/message";

import { frontend } from "@/lib/ory";
import { RecoveryFlow, UpdateRecoveryFlowBody } from "@ory/client";
import { filterNodesByGroups } from "@ory/integrations/ui";
import { isAxiosError } from "axios";
import { useSession } from "@/contexts/Session";
import { useRouter, useSearchParams } from "next/navigation";

interface UserRecoveryFormProps extends React.HTMLAttributes<HTMLDivElement> {}

export function UserRecoveryForm({
  className,
  ...props
}: UserRecoveryFormProps) {
  const router = useRouter();
  const session = useSession();
  const searchParams = useSearchParams();

  const [flow, setFlow] = React.useState<RecoveryFlow | undefined>(undefined);
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
    ) as unknown as UpdateRecoveryFlowBody;

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
      const { data } = await frontend.updateRecoveryFlow({
        flow: flow!.id,
        updateRecoveryFlowBody: body,
      });

      // step after user enters their email
      if (data.state === "sent_email") {
        formRef.current?.reset();
        setFlow(data);
      }
    } catch (err: unknown) {
      if (!isAxiosError(err)) {
        throw err;
      }

      // handle the error
      if (err.response?.status === 400) {
        // user input error
        // show the error messages in the UI
        setFlow(err.response.data as RecoveryFlow);
      }

      // handle unprocessable entity
      if (err.response?.status === 422) {
        if (err.response?.data?.redirect_browser_to) {
          // redirect the user to the next step
          router.push(err.response.data.redirect_browser_to);
        }
      }
    } finally {
      setIsLoading(false);
    }
  }

  React.useEffect(() => {
    const flowId = searchParams.get("flow");

    if (flowId) {
      frontend
        .getRecoveryFlow({ id: flowId })
        .then(({ data: flow }) => {
          setFlow(flow);
        })
        .catch((err: unknown) => {
          if (!isAxiosError(err)) {
            throw err;
          }

          if (err.response?.status === 404 || err.response?.status === 403) {
            router.push("/auth/recovery");
          }
        });
    } else {
      frontend.createBrowserRecoveryFlow().then(({ data: flow }) => {
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

  console.log(flow.ui.nodes);

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
