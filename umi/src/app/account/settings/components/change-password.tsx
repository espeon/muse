import { Node } from "@/components/auth/node";

import { UiNode } from "@ory/client";

interface ChangePasswordProps extends React.HTMLAttributes<HTMLDivElement> {
  nodes: UiNode[];
  isLoading: boolean;
}

export function ChangePassword({ nodes }: ChangePasswordProps) {
  return (
    <>
      <div className="space-y-2">
        <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
          Change your password
        </label>
        {nodes.map((node, index) => (
          <Node key={index} node={node} loading={false} />
        ))}
      </div>
    </>
  );
}
