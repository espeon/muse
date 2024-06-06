"use client";

import * as React from "react";

import { cn } from "@/lib/utils";
import { Icons } from "@/components/icons";
import { Node } from "@/components/auth/node";

import { frontend } from "@/lib/ory";
import { LoginFlow, UpdateLoginFlowBody } from "@ory/client";
import { filterNodesByGroups } from "@ory/integrations/ui";
import { isAxiosError } from "axios";
import { Message } from "@/components/auth/message";
import { useSession } from "@/contexts/Session";
import { useRouter } from "next/navigation";

interface UserLoginFormProps extends React.HTMLAttributes<HTMLDivElement> {}

export function UserLoginForm({ className, ...props }: UserLoginFormProps) {
  const router = useRouter();
  const session = useSession();

  const [flow, setFlow] = React.useState<LoginFlow | undefined>(undefined);
  const [isLoading, setIsLoading] = React.useState<boolean>(false);

  async function onSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsLoading(true);

    const form = event.currentTarget;
    const formData = new FormData(form);

    // map the entire form data to JSON for the request body
    let body = Object.fromEntries(formData) as unknown as UpdateLoginFlowBody;

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
      const { data } = await frontend.updateLoginFlow({
        flow: flow!.id,
        updateLoginFlowBody: body,
      });

      if (data.session) {
        session.set(data.session);
      }
    } catch (err: unknown) {
      if (!isAxiosError(err)) {
        throw err;
      }

      // handle the error
      if (err.response?.status === 400) {
        // user input error
        // show the error messages in the UI
        setFlow(err.response.data as LoginFlow);
      }
    } finally {
      setIsLoading(false);
    }
  }

  React.useEffect(() => {
    frontend.createBrowserLoginFlow().then(({ data: flow }) => {
      setFlow(flow);
    });
  }, []);

  React.useEffect(() => {
    if (session.data) {
      router.push("/");
    }
  }, [session.data, router]);

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

      <form onSubmit={onSubmit}>
        <div className="grid gap-2">
          {filterNodesByGroups({
            nodes: flow.ui.nodes,
            groups: ["default", "password"],
          }).map((node, index) => (
            <Node key={index} node={node} loading={isLoading} />
          ))}
        </div>
      </form>
    </div>
  );
}
