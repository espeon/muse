import { UiNode, UiNodeAnchorAttributes, UiNodeImageAttributes, UiNodeInputAttributes, UiNodeScriptAttributes, UiNodeTextAttributes } from "@ory/client";

import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Icons } from "@/components/icons";
import { isUiNodeInputAttributes, getNodeLabel } from "@ory/integrations/ui";

import classNames from "classnames";

interface NodeProps extends React.HTMLAttributes<HTMLDivElement> {
  node: UiNode;
  loading?: boolean;
}

const placeholder: { [key: string]: string } = {
  "traits.email": "hunter2@infosec.guru",
  password: "correct horse battery staple",
};

export function Node({ node, ...props }: NodeProps) {
  if (node.attributes.node_type === "input") {
    const attrs = node.attributes as UiNodeInputAttributes;
    const nodeType = attrs.type;
    const errors = node.messages?.filter((message) => message.type === "error");
    switch (nodeType) {
      case "button":
      case "submit":
        if (node.meta.label?.text.includes("Sign")) {
          return (
            <div {...props}>
              <div className="flex-col py-2">
                <div className="text-sm flex">
                  forgot your password?{" "}
                  <a
                    className="ml-1 text-blue-500 hover:text-blue-600"
                    href="#"
                  >
                    click here
                  </a>
                </div>
                <div className="text-sm flex">
                  or{" "}
                  {node.meta.label?.text.includes("in") ? (
                    <a
                      className="ml-1 text-blue-500 hover:text-blue-600"
                      href="#"
                    >
                      sign up
                    </a>
                  ) : (
                    <a
                      className="ml-1 text-blue-500 hover:text-blue-600"
                      href="#"
                    >
                      sign in
                    </a>
                  )}
                </div>
              </div>
                <Button
                  className="w-full dark:bg-slate-700 dark:hover:bg-slate-800 bg-slate-800"
                  type="submit"
                  id={attrs.name}
                  name={attrs.name}
                  value={attrs.value}
                >
                  {node.meta.label?.text ?? "Sign in"}
                </Button>
            </div>
          );
        } else {
          return (
            <div {...props}>
            <Button
              className="w-full dark:bg-slate-700 dark:hover:bg-slate-800 bg-slate-800"
              type="submit"
              id={attrs.name}
              name={attrs.name}
              value={attrs.value}
              
            >
              {node.meta.label?.text ?? "Sign in"}
            </Button>
            </div>
          );
        }
      default:
        return (
          <div {...props}>
            <div className="flex">
              <div className="text-xl font-normal my-1">
                {node.meta.label?.text ?? ""}
              </div>
            </div>
            <Input
              className={classNames(
                "w-full bg-slate-100",
                {
                  "border-2 border-destructive": errors?.length > 0,
                }
              )}
              placeholder={placeholder[attrs.name] ?? ""}
              id={attrs.name}
              name={attrs.name}
              type={attrs.type}
              value={attrs.type !== "hidden" ? undefined : attrs.value}
            />
          {errors?.map((message, index) => (
            <p key={index} className={"text-[0.8rem] font-medium text-destructive"}>
              {message.text}
            </p>
          ))}
          </div>
        );
    }
  } else if (node.attributes.node_type === "img") {
    const attrs = node.attributes as UiNodeImageAttributes;
    return <img src={attrs.src} id={props.id} />;
  } else if (node.attributes.node_type === "text") {
    const attrs = node.attributes as UiNodeTextAttributes;
    return <div>{attrs.text.text}</div>;
  } else if (node.attributes.node_type === "a") {
    const attrs = node.attributes as UiNodeAnchorAttributes;
    return (
      <a id={props.id} href={attrs.href}>
        {attrs.title.text}
      </a>
    );
  } else if (node.attributes.node_type === "script") {
    const attrs = node.attributes as UiNodeScriptAttributes;
    return <script id={props.id} src={attrs.src} />;
  }

  return null;
}