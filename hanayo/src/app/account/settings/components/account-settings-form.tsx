"use client";

import * as React from "react";

import { cn } from "@/lib/utils";
import { Icons } from "@/components/icons";
import { Node } from "@/components/auth/node";
import { ChangePassword } from "@/app/account/settings/components/change-password";

import { frontend } from "@/lib/ory";
import { SettingsFlow, UpdateSettingsFlowBody } from "@ory/client";
import { filterNodesByGroups } from "@ory/integrations/ui";
import { isAxiosError } from "axios";
import { Message } from "@/components/auth/message";
import { useSession } from "@/contexts/Session";
import { useRouter } from "next/navigation";
import { ChangeProfile } from "./change-profile";

interface AccountSettingsFormProps
  extends React.HTMLAttributes<HTMLDivElement> {}

export function AccountSettingsForm({
  className,
  ...props
}: AccountSettingsFormProps) {
  const router = useRouter();
  const session = useSession();

  const [flow, setFlow] = React.useState<SettingsFlow | undefined>(undefined);
  const [isLoading, setIsLoading] = React.useState<boolean>(false);

  async function onSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsLoading(true);

    // clear ui messages
    setFlow((flow) => {
      if (flow) {
        flow.ui.messages = [];
      }
      return flow;
    });

    const form = event.currentTarget;
    const formData = new FormData(form);

    // map the entire form data to JSON for the request body
    let body = Object.fromEntries(
      formData
    ) as unknown as UpdateSettingsFlowBody;

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
      const { data } = await frontend.updateSettingsFlow({
        flow: flow!.id,
        updateSettingsFlowBody: body,
      });

      if (data.ui) {
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
        setFlow(err.response.data as SettingsFlow);
      }

      // handle redirect browser to
      if (err.response?.data?.redirect_browser_to) {
        // redirect the user to the next step
        router.push(err.response.data.redirect_browser_to);
      }
    } finally {
      setIsLoading(false);
    }
  }

  React.useEffect(() => {
    frontend.createBrowserSettingsFlow().then(({ data: flow }) => {
      setFlow(flow);
    });
  }, []);

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
            groups: ["default"],
          }).map((node, index) => (
            <Node key={index} node={node} loading={isLoading} />
          ))}

          <ChangeProfile
            nodes={filterNodesByGroups({
              nodes: flow.ui.nodes,
              groups: ["profile"],
            })}
            isLoading={isLoading}
          />

          <ChangePassword
            nodes={filterNodesByGroups({
              nodes: flow.ui.nodes,
              groups: ["password"],
            })}
            isLoading={isLoading}
          />
        </div>
      </form>
    </div>
  );
}
