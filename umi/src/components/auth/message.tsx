import {
  ExclamationTriangleIcon,
  InfoCircledIcon,
  CheckCircledIcon,
} from "@radix-ui/react-icons";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { UiText } from "@ory/client";

interface MessageProps extends React.HTMLAttributes<HTMLDivElement> {
  text: UiText;
}

export function Message({ text }: MessageProps) {
  if (text.type === "info") {
    return <Info text={text} />;
  }

  if (text.type === "success") {
    return <Success text={text} />;
  }

  if (text.type === "error") {
    return <Error text={text} />;
  }

  return null;
}

const Info = ({ text }: MessageProps) => {
  return (
    <Alert>
      <InfoCircledIcon className="h-4 w-4" />
      <AlertTitle>Info</AlertTitle>
      <AlertDescription>{text.text}</AlertDescription>
    </Alert>
  );
};

const Success = ({ text }: MessageProps) => {
  return (
    <Alert variant="success">
      <CheckCircledIcon className="h-4 w-4" />
      <AlertTitle>Success</AlertTitle>
      <AlertDescription>{text.text}</AlertDescription>
    </Alert>
  );
};

const Error = ({ text }: MessageProps) => {
  return (
    <Alert variant="destructive">
      <ExclamationTriangleIcon className="h-4 w-4" />
      <AlertTitle>Error</AlertTitle>
      <AlertDescription>{text.text}</AlertDescription>
    </Alert>
  );
};
